use embassy_time::{Duration, Timer};
use embedded_hal_async::i2c::I2c;

use crate::sensors::sensirion::crc8;

const SEN55_ADDR: u8 = 0x69;

const CMD_START_MEASUREMENT: [u8; 2] = [0x00, 0x21];

const CMD_STOP_MEASUREMENT: [u8; 2] = [0x01, 0x04];

const CMD_READ_DATA_READY: [u8; 2] = [0x02, 0x02];

const CMD_READ_MEASURED_VALUES: [u8; 2] = [0x03, 0xC4];

pub struct AQISensorData {
    pub pm1_0: f32,
    pub pm2_5: f32,
    pub pm4_0: f32,
    pub pm10: f32,

    pub humidity_percent: f32,
    pub temperature_celsius: f32,

    pub voc_index: f32,
    pub nox_index: f32,
}

#[derive(Debug, Copy, Clone, defmt::Format)]
pub enum DecodeError {
    Pm1CrcMismatch,
    Pm25CrcMismatch,
    Pm4CrcMismatch,
    Pm10CrcMismatch,
    HumidityCrcMismatch,
    TemperatureCrcMismatch,
    VocCrcMismatch,
    NoxCrcMismatch,
    DataReadyCrcMismatch,
}

#[derive(Debug)]
pub enum Error<E> {
    I2c(E),
    Decode(DecodeError),
}

impl<E> defmt::Format for Error<E> {
    fn format(&self, f: defmt::Formatter) {
        match self {
            Error::I2c(_) => {
                defmt::write!(f, "I2C error");
            }

            Error::Decode(e) => {
                defmt::write!(f, "Decode error: {:?}", e);
            }
        }
    }
}

pub struct Sen55<I2C> {
    i2c: I2C,
}

impl<I2C> Sen55<I2C>
where
    I2C: I2c,
{
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    /// Starts continuous measurement.
    ///
    /// The first measurement will become available after approximately
    /// one second.
    pub async fn start(&mut self) -> Result<(), Error<I2C::Error>> {
        // Stop first in case measurements are already running.
        let _ = self.stop().await;

        Timer::after(Duration::from_millis(200)).await;

        self.send_command(&CMD_START_MEASUREMENT).await?;

        // Give the internal fan/laser time to start.
        Timer::after(Duration::from_secs(1)).await;

        Ok(())
    }

    /// Stops continuous measurement.
    pub async fn stop(&mut self) -> Result<(), Error<I2C::Error>> {
        self.send_command(&CMD_STOP_MEASUREMENT).await?;

        // Datasheet specifies < 200 ms execution time.
        Timer::after(Duration::from_millis(200)).await;

        Ok(())
    }

    /// Returns true when a new sample is available.
    pub async fn data_ready(&mut self) -> Result<bool, Error<I2C::Error>> {
        let mut buffer = [0u8; 3];

        self.send_command(&CMD_READ_DATA_READY).await?;

        self.i2c
            .read(SEN55_ADDR, &mut buffer)
            .await
            .map_err(Error::I2c)?;

        is_data_ready(&buffer).map_err(Error::Decode)
    }

    /// Reads one measurement frame.
    ///
    /// Assumes data_ready() returned true.
    pub async fn read(&mut self) -> Result<AQISensorData, Error<I2C::Error>> {
        // 8 values × (2 data bytes + CRC)
        let mut buffer = [0u8; 24];

        self.send_command(&CMD_READ_MEASURED_VALUES).await?;

        Timer::after(Duration::from_millis(1)).await;

        self.i2c
            .read(SEN55_ADDR, &mut buffer)
            .await
            .map_err(Error::I2c)?;

        decode(&buffer).map_err(Error::Decode)
    }

    async fn send_command(&mut self, command: &[u8; 2]) -> Result<(), Error<I2C::Error>> {
        self.i2c
            .write(SEN55_ADDR, command)
            .await
            .map_err(Error::I2c)
    }
}

/// Parses the 3-byte "get data ready status" response.
/// The least significant bit of the status word is set when a fresh sample is available.
pub fn is_data_ready(buffer: &[u8; 3]) -> Result<bool, DecodeError> {
    let word_bytes = [buffer[0], buffer[1]];

    if crc8(word_bytes) != buffer[2] {
        return Err(DecodeError::DataReadyCrcMismatch);
    }

    let status = u16::from_be_bytes(word_bytes);

    Ok(status & 0x0001 != 0)
}

/// Parses the 24-byte "read measured values" response: PM1.0, PM2.5, PM4.0,
/// PM10, humidity, temperature, VOC index, and NOx index, each as a
/// 2-byte big-endian value followed by a CRC byte.
pub fn decode(buffer: &[u8; 24]) -> Result<AQISensorData, DecodeError> {
    let pm1 = decode_word(buffer[0], buffer[1], buffer[2], DecodeError::Pm1CrcMismatch)?;

    let pm25 = decode_word(
        buffer[3],
        buffer[4],
        buffer[5],
        DecodeError::Pm25CrcMismatch,
    )?;

    let pm4 = decode_word(buffer[6], buffer[7], buffer[8], DecodeError::Pm4CrcMismatch)?;

    let pm10 = decode_word(
        buffer[9],
        buffer[10],
        buffer[11],
        DecodeError::Pm10CrcMismatch,
    )?;

    let humidity = decode_word(
        buffer[12],
        buffer[13],
        buffer[14],
        DecodeError::HumidityCrcMismatch,
    )?;

    let temperature = decode_word(
        buffer[15],
        buffer[16],
        buffer[17],
        DecodeError::TemperatureCrcMismatch,
    )?;

    let voc = decode_word(
        buffer[18],
        buffer[19],
        buffer[20],
        DecodeError::VocCrcMismatch,
    )?;

    let nox = decode_word(
        buffer[21],
        buffer[22],
        buffer[23],
        DecodeError::NoxCrcMismatch,
    )?;

    Ok(AQISensorData {
        pm1_0: pm1 as f32 / 10.0,
        pm2_5: pm25 as f32 / 10.0,
        pm4_0: pm4 as f32 / 10.0,
        pm10: pm10 as f32 / 10.0,

        humidity_percent: humidity as f32 / 100.0,

        temperature_celsius: temperature as f32 / 200.0,

        voc_index: voc as f32 / 10.0,
        nox_index: nox as f32 / 10.0,
    })
}

fn decode_word(msb: u8, lsb: u8, crc: u8, err: DecodeError) -> Result<u16, DecodeError> {
    let bytes = [msb, lsb];

    if crc8(bytes) != crc {
        return Err(err);
    }

    Ok(u16::from_be_bytes(bytes))
}
