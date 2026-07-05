//! Driver/decoder for the Sensirion SFA30 formaldehyde sensor.

use crate::domain::sensor_data::HCOHSensorData;

pub const SFA30_ADDR: u8 = 0x5D;

pub const CMD_START_CONTINUOUS: [u8; 2] = [0x00, 0x06];
pub const CMD_READ_VALUES: [u8; 2] = [0x03, 0x27];

fn crc8(data: [u8; 2]) -> u8 {
    let mut crc: u8 = 0xFF;
    for &byte in data.iter() {
        crc ^= byte;
        for _ in 0..8 {
            crc = if crc & 0x80 != 0 {
                (crc << 1) ^ 0x31
            } else {
                crc << 1
            };
        }
    }
    crc
}

pub fn decode(buffer: &[u8; 9]) -> Result<HCOHSensorData, &'static str> {
    let hcho_bytes = [buffer[0], buffer[1]];
    let rh_bytes = [buffer[3], buffer[4]];
    let temp_bytes = [buffer[6], buffer[7]];

    if crc8(hcho_bytes) != buffer[2] {
        return Err("HCHO CRC mismatch");
    }
    if crc8(rh_bytes) != buffer[5] {
        return Err("Humidity CRC mismatch");
    }
    if crc8(temp_bytes) != buffer[8] {
        return Err("Temperature CRC mismatch");
    }

    let hcho_raw = i16::from_be_bytes(hcho_bytes);
    let rh_raw = i16::from_be_bytes(rh_bytes);
    let temp_raw = i16::from_be_bytes(temp_bytes);

    Ok(HCOHSensorData {
        hcho_ppb: hcho_raw as f32 / 5.0,
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
        assert!((reading.hcho_ppb - 11.0).abs() < 0.01);
        assert!((reading.humidity_percent - 54.75).abs() < 0.01);
        assert!((reading.temp_celsius - 32.63).abs() < 0.01);
    }
}
