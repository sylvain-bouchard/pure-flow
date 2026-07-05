//! Driver/decoder for the Sensirion SCD40 carbon dioxide sensor.
use crate::sensors::sensirion::crc8;

pub const SCD40_ADDR: u8 = 0x62;

pub const CMD_START_PERIODIC_MEASUREMENT: [u8; 2] = [0x21, 0xB1];
pub const CMD_READ_MEASUREMENT: [u8; 2] = [0xEC, 0x05];
pub const CMD_GET_DATA_READY_STATUS: [u8; 2] = [0xE4, 0xB8];

/// Decoded, unscaled SCD40 reading. Caller maps this into `SensorData::Co2`.
pub struct Scd40Reading {
    pub co2_ppm: u16,
    pub humidity_percent: f32,
    pub temp_celsius: f32,
}

/// Parses the 3-byte "get data ready status" response.
/// The low 11 bits of the word are non-zero when a fresh sample is available.
pub fn is_data_ready(buffer: &[u8; 3]) -> Result<bool, &'static str> {
    let word_bytes = [buffer[0], buffer[1]];

    if crc8(word_bytes) != buffer[2] {
        return Err("Data-ready status CRC mismatch");
    }

    let word = u16::from_be_bytes(word_bytes);
    Ok(word & 0x07ff != 0)
}

/// Parses the 9-byte "read measurement" response: CO2, temperature, humidity,
/// each as a 2-byte big-endian value followed by a CRC byte.
pub fn decode(buffer: &[u8; 9]) -> Result<Scd40Reading, &'static str> {
    let co2_bytes = [buffer[0], buffer[1]];
    let temp_bytes = [buffer[3], buffer[4]];
    let rh_bytes = [buffer[6], buffer[7]];

    if crc8(co2_bytes) != buffer[2] {
        return Err("CO2 CRC mismatch");
    }
    if crc8(temp_bytes) != buffer[5] {
        return Err("Temperature CRC mismatch");
    }
    if crc8(rh_bytes) != buffer[8] {
        return Err("Humidity CRC mismatch");
    }

    let co2_raw = u16::from_be_bytes(co2_bytes);
    let temp_raw = u16::from_be_bytes(temp_bytes);
    let rh_raw = u16::from_be_bytes(rh_bytes);

    Ok(Scd40Reading {
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
