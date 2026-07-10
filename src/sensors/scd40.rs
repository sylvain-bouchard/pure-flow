//! Driver/decoder for the Sensirion SCD40 carbon dioxide sensor.

use crate::domain::sensor_data::CO2SensorData;
use crate::sensors::sensirion::crc8;

use embassy_time::{Duration, Timer};
use embedded_hal_async::i2c::I2c;

const SCD40_ADDR: u8 = 0x62;

const CMD_START_PERIODIC_MEASUREMENT: [u8; 2] = [0x21, 0xB1];
const CMD_GET_DATA_READY_STATUS: [u8; 2] = [0xE4, 0xB8];
const CMD_READ_MEASUREMENT: [u8; 2] = [0xEC, 0x05];
const CMD_WAKE_UP: [u8; 2] = [0x36, 0xF6];
const CMD_STOP_PERIODIC_MEASUREMENT: [u8; 2] = [0x3F, 0x86];
const CMD_REINIT: [u8; 2] = [0x36, 0x46];

#[derive(Debug, Copy, Clone, defmt::Format)]
pub enum DecodeError {
    Co2CrcMismatch,
    TemperatureCrcMismatch,
    HumidityCrcMismatch,
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

pub struct Scd40<I2C> {
    i2c: I2C,
}

impl<I2C> Scd40<I2C>
where
    I2C: I2c,
{
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    pub async fn start(&mut self) -> Result<(), Error<I2C::Error>> {
        // Stop previous measurement if it was running
        self.send_command(&CMD_STOP_PERIODIC_MEASUREMENT).await?;

        // This command needs ~500ms execution time
        Timer::after(Duration::from_millis(500)).await;

        // Reset internal state
        self.send_command(&CMD_REINIT).await?;

        Timer::after(Duration::from_millis(30)).await;

        // Start measurements
        self.send_command(&CMD_START_PERIODIC_MEASUREMENT).await?;

        Ok(())
    }

    pub async fn data_ready(&mut self) -> Result<bool, Error<I2C::Error>> {
        let mut buffer = [0u8; 3];

        self.send_command(&CMD_GET_DATA_READY_STATUS).await?;

        self.i2c
            .read(SCD40_ADDR, &mut buffer)
            .await
            .map_err(Error::I2c)?;

        // Decode status word here
        Ok(is_data_ready(&buffer).map_err(|_| Error::Decode(DecodeError::DataReadyCrcMismatch))?)
    }

    pub async fn read(&mut self) -> Result<CO2SensorData, Error<I2C::Error>> {
        let mut buffer = [0u8; 9];

        self.send_command(&CMD_READ_MEASUREMENT).await?;

        Timer::after(Duration::from_millis(1)).await;

        self.i2c
            .read(SCD40_ADDR, &mut buffer)
            .await
            .map_err(Error::I2c)?;

        decode(&buffer).map_err(Error::Decode)
    }

    async fn send_command(&mut self, command: &[u8; 2]) -> Result<(), Error<I2C::Error>> {
        self.i2c
            .write(SCD40_ADDR, command)
            .await
            .map_err(Error::I2c)
    }
}

/// Parses the 3-byte "get data ready status" response.
/// The low 11 bits of the word are non-zero when a fresh sample is available.
pub fn is_data_ready(buffer: &[u8; 3]) -> Result<bool, DecodeError> {
    let word_bytes = [buffer[0], buffer[1]];

    if crc8(word_bytes) != buffer[2] {
        return Err(DecodeError::DataReadyCrcMismatch);
    }

    let word = u16::from_be_bytes(word_bytes);
    Ok(word & 0x07ff != 0)
}

/// Parses the 9-byte "read measurement" response: CO2, temperature, humidity,
/// each as a 2-byte big-endian value followed by a CRC byte.
pub fn decode(buffer: &[u8; 9]) -> Result<CO2SensorData, DecodeError> {
    let co2_bytes = [buffer[0], buffer[1]];
    let temp_bytes = [buffer[3], buffer[4]];
    let rh_bytes = [buffer[6], buffer[7]];

    if crc8(co2_bytes) != buffer[2] {
        return Err(DecodeError::Co2CrcMismatch);
    }
    if crc8(temp_bytes) != buffer[5] {
        return Err(DecodeError::TemperatureCrcMismatch);
    }
    if crc8(rh_bytes) != buffer[8] {
        return Err(DecodeError::HumidityCrcMismatch);
    }

    let co2_raw = u16::from_be_bytes(co2_bytes);
    let temp_raw = u16::from_be_bytes(temp_bytes);
    let rh_raw = u16::from_be_bytes(rh_bytes);

    Ok(CO2SensorData {
        co2_ppm: co2_raw,
        temp_celsius: -45.0 + 175.0 * (temp_raw as f32) / 65535.0,
        humidity_percent: 100.0 * (rh_raw as f32) / 65535.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_known_sample() {
        // CO2 = 500 ppm, temp raw = 27067 (~27.28C), RH raw = 26214 (40%)
        let buffer = [0x01, 0xf4, 0x33, 0x69, 0xbb, 0x42, 0x66, 0x66, 0x93];
        let reading = decode(&buffer).unwrap();
        assert_eq!(reading.co2_ppm, 500);
        assert!((reading.temp_celsius - 27.28).abs() < 0.05);
        assert!((reading.humidity_percent - 40.0).abs() < 0.05);
    }
}
