//! Shared CRC-8 implementation used across Sensirion sensors (SFA30, SCD40,
//! SHT, SGP, etc). All of them use the same polynomial (0x31) and initial
//! value (0xFF) per the Sensirion I2C command protocol.

pub fn crc8(data: [u8; 2]) -> u8 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_vector() {
        // From the Sensirion CRC-8 application note example.
        assert_eq!(crc8([0xbe, 0xef]), 0x92);
    }
}
