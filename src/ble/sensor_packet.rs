use crate::domain::sensor_data::SensorData;

pub enum SensorPacket {
    Hcho(HchoPacket),
    Co2(Co2Packet),
    Aqi(AqiPacket),
}

pub struct HchoPacket {
    pub hcho_ppb: u16,
    pub relative_humidity: u16,
    pub temperature: i16,
}

pub struct Co2Packet {
    pub co2_ppm: u16,
    pub relative_humidity: u16,
    pub temperature: i16,
}

pub struct AqiPacket {
    pub pm1_0: u16,
    pub pm2_5: u16,
    pub pm4_0: u16,
    pub pm10: u16,
    pub relative_humidity: u16,
    pub temperature: i16,
    pub voc_index: u16,
    pub nox_index: u16,
}

impl From<SensorData> for SensorPacket {
    fn from(data: SensorData) -> Self {
        match data {
            SensorData::Hcho(data) => SensorPacket::Hcho(HchoPacket {
                hcho_ppb: data.hcho_ppb as u16,
                relative_humidity: data.humidity_percent as u16,
                temperature: data.temp_celsius as i16,
            }),

            SensorData::Co2(data) => SensorPacket::Co2(Co2Packet {
                co2_ppm: data.co2_ppm,
                relative_humidity: data.humidity_percent as u16,
                temperature: data.temp_celsius as i16,
            }),

            SensorData::Aqi(data) => SensorPacket::Aqi(AqiPacket {
                pm1_0: data.pm1_0 as u16,
                pm2_5: data.pm2_5 as u16,
                pm4_0: data.pm4_0 as u16,
                pm10: data.pm10 as u16,
                relative_humidity: data.humidity_percent as u16,
                temperature: data.temperature_celsius as i16,
                voc_index: data.voc_index as u16,
                nox_index: data.nox_index as u16,
            }),
        }
    }
}

impl SensorPacket {
    pub fn encode(&self) -> ([u8; 24], usize) {
        let mut buffer = [0u8; 24];

        let size = match self {
            SensorPacket::Hcho(packet) => packet.encode(&mut buffer),

            SensorPacket::Co2(packet) => packet.encode(&mut buffer),

            SensorPacket::Aqi(packet) => packet.encode(&mut buffer),
        };

        (buffer, size)
    }
}

impl HchoPacket {
    fn encode(&self, buffer: &mut [u8; 24]) -> usize {
        buffer[0..2].copy_from_slice(&self.hcho_ppb.to_le_bytes());
        buffer[2..4].copy_from_slice(&self.relative_humidity.to_le_bytes());
        buffer[4..6].copy_from_slice(&self.temperature.to_le_bytes());

        6
    }
}

impl Co2Packet {
    fn encode(&self, buffer: &mut [u8; 24]) -> usize {
        buffer[0..2].copy_from_slice(&self.co2_ppm.to_le_bytes());
        buffer[2..4].copy_from_slice(&self.relative_humidity.to_le_bytes());
        buffer[4..6].copy_from_slice(&self.temperature.to_le_bytes());

        6
    }
}

impl AqiPacket {
    fn encode(&self, buffer: &mut [u8; 24]) -> usize {
        buffer[0..2].copy_from_slice(&self.pm1_0.to_le_bytes());
        buffer[2..4].copy_from_slice(&self.pm2_5.to_le_bytes());
        buffer[4..6].copy_from_slice(&self.pm4_0.to_le_bytes());
        buffer[6..8].copy_from_slice(&self.pm10.to_le_bytes());

        buffer[8..10].copy_from_slice(&self.relative_humidity.to_le_bytes());
        buffer[10..12].copy_from_slice(&self.temperature.to_le_bytes());

        buffer[12..14].copy_from_slice(&self.voc_index.to_le_bytes());
        buffer[14..16].copy_from_slice(&self.nox_index.to_le_bytes());

        16
    }
}
