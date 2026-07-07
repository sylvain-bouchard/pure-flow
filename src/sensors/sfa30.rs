//! Driver/decoder for the Sensirion SFA30 formaldehyde sensor.

use crate::domain::sensor_data::HCOHSensorData;
use crate::sensors::sensirion::crc8;

use embassy_time::{Duration, Timer};
use embedded_hal_async::i2c::I2c;

pub const SFA30_ADDR: u8 = 0x5D;

const CMD_START_CONTINUOUS: [u8; 2] = [0x00, 0x06];
const CMD_READ_VALUES: [u8; 2] = [0x03, 0x27];

#[derive(Debug, Copy, Clone, defmt::Format)]
pub enum DecodeError {
    HchoCrcMismatch,
    HumidityCrcMismatch,
    TemperatureCrcMismatch,
}

#[derive(Debug, defmt::Format)]
pub enum Error {
    I2c,
    Decode(DecodeError),
}

pub struct Sfa30<I2C> {
    i2c: I2C,
}

impl<I2C> Sfa30<I2C>
where
    I2C: I2c,
{
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    /// Starts continuous measurement.
    pub async fn start(&mut self) -> Result<(), Error> {
        self.i2c
            .write(SFA30_ADDR, &CMD_START_CONTINUOUS)
            .await
            .map_err(|_| Error::I2c)?;

        Ok(())
    }

    /// Reads one HCHO measurement.
    pub async fn read(&mut self) -> Result<HCOHSensorData, Error> {
        self.i2c
            .write(SFA30_ADDR, &CMD_READ_VALUES)
            .await
            .map_err(|_| Error::I2c)?;

        // Sensor requires ~50 ms before data is ready.
        Timer::after(Duration::from_millis(50)).await;

        let mut buffer = [0u8; 9];

        self.i2c
            .read(SFA30_ADDR, &mut buffer)
            .await
            .map_err(|_| Error::I2c)?;

        decode(&buffer).map_err(Error::Decode)
    }
}

fn decode(buffer: &[u8; 9]) -> Result<HCOHSensorData, DecodeError> {
    let hcho_bytes = [buffer[0], buffer[1]];
    let rh_bytes = [buffer[3], buffer[4]];
    let temp_bytes = [buffer[6], buffer[7]];

    if crc8(hcho_bytes) != buffer[2] {
        return Err(DecodeError::HchoCrcMismatch);
    }
    if crc8(rh_bytes) != buffer[5] {
        return Err(DecodeError::HumidityCrcMismatch);
    }
    if crc8(temp_bytes) != buffer[8] {
        return Err(DecodeError::TemperatureCrcMismatch);
    }

    let hcho_raw = i16::from_be_bytes(hcho_bytes);
    let rh_raw = i16::from_be_bytes(rh_bytes);
    let temp_raw = i16::from_be_bytes(temp_bytes);

    Ok(HCOHSensorData {
        hcho_ppb: hcho_raw as f32,
        humidity_percent: rh_raw as f32 / 100.0,
        temp_celsius: temp_raw as f32 / 200.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_known_sample() {
        let buffer = [0, 55, 211, 21, 99, 112, 25, 126, 203];
        let reading = decode(&buffer).unwrap();
        assert!((reading.hcho_ppb - 55.0).abs() < 0.01);
        assert!((reading.humidity_percent - 54.75).abs() < 0.01);
        assert!((reading.temp_celsius - 32.63).abs() < 0.01);
    }
}
