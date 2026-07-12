use crate::domain::sensor_data::EnvironmentData;

pub struct EnvironmentPacket {
    pub co2_ppm: u16,
    pub hcho_ppb: u16,

    pub pm1_0: u16,
    pub pm2_5: u16,
    pub pm4_0: u16,
    pub pm10: u16,

    pub voc_index: u16,
    pub nox_index: u16,

    pub relative_humidity: u16,
    pub temperature: i16,
}

impl From<EnvironmentData> for EnvironmentPacket {
    fn from(data: EnvironmentData) -> Self {
        Self {
            co2_ppm: data.co2_ppm.unwrap_or(0),
            hcho_ppb: data.hcho_ppb.unwrap_or(0),

            pm1_0: data.pm1_0.unwrap_or(0),
            pm2_5: data.pm2_5.unwrap_or(0),
            pm4_0: data.pm4_0.unwrap_or(0),
            pm10: data.pm10.unwrap_or(0),

            voc_index: data.voc_index.unwrap_or(0),
            nox_index: data.nox_index.unwrap_or(0),

            relative_humidity: data.humidity_percent.unwrap_or(0.0) as u16,
            temperature: data.temperature_celsius.unwrap_or(0.0) as i16,
        }
    }
}

impl EnvironmentPacket {
    pub fn encode(&self) -> [u8; 20] {
        let mut buffer = [0u8; 20];

        buffer[0..2].copy_from_slice(&self.co2_ppm.to_le_bytes());
        buffer[2..4].copy_from_slice(&self.hcho_ppb.to_le_bytes());

        buffer[4..6].copy_from_slice(&self.pm1_0.to_le_bytes());
        buffer[6..8].copy_from_slice(&self.pm2_5.to_le_bytes());
        buffer[8..10].copy_from_slice(&self.pm4_0.to_le_bytes());
        buffer[10..12].copy_from_slice(&self.pm10.to_le_bytes());

        buffer[12..14].copy_from_slice(&self.voc_index.to_le_bytes());
        buffer[14..16].copy_from_slice(&self.nox_index.to_le_bytes());

        buffer[16..18].copy_from_slice(&self.relative_humidity.to_le_bytes());
        buffer[18..20].copy_from_slice(&self.temperature.to_le_bytes());

        buffer
    }
}