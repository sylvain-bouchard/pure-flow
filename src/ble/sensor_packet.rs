use crate::domain::sensor_data::SensorData;

pub struct SensorPacket {
    pub level: u16,
    pub relative_humidity: u16,
    pub temperature: i16,
}

impl From<SensorData> for SensorPacket {
    fn from(data: SensorData) -> Self {
        match data {
            SensorData::Hcho(data) => Self {
                level: data.hcho_ppb as u16,
                relative_humidity: data.humidity_percent as u16,
                temperature: data.temp_celsius as i16,
            },

            SensorData::Co2(data) => Self {
                level: data.co2_ppm,
                relative_humidity: data.humidity_percent as u16,
                temperature: data.temp_celsius as i16,
            },
        }
    }
}

impl SensorPacket {
    pub fn encode(&self) -> [u8; 6] {
        let mut packet_bytes = [0u8; 6];
        packet_bytes[0..2].copy_from_slice(&self.level.to_le_bytes());
        packet_bytes[2..4].copy_from_slice(&self.relative_humidity.to_le_bytes());
        packet_bytes[4..6].copy_from_slice(&self.temperature.to_le_bytes());
        packet_bytes
    }
}
