//! Driver/decoder for the Sensirion SCD40 carbon dioxide sensor.
pub const SCD40_ADDR: u8 = 0x62;

pub const CMD_START_PERIODIC_MEASUREMENT: [u8; 2] = [0x21, 0xB1];
pub const CMD_READ_MEASUREMENT: [u8; 2] = [0xEC, 0x05];