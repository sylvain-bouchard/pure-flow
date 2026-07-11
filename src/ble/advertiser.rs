use crate::ble::sensor_packet::SensorPacket;
use crate::transport::TransportError;
use crate::{domain::sensor_data::SensorData, transport::TelemetryTransport};

use defmt::info;

pub struct BleAdvertiser {}

impl TelemetryTransport for BleAdvertiser {
    async fn send(&mut self, data: SensorData) -> Result<(), TransportError> {
        match &data {
            SensorData::Hcho(reading) => info!(
                "HCHO: {} ppb | Humidity: {}% | Temp: {} C",
                reading.hcho_ppb, reading.humidity_percent, reading.temp_celsius
            ),

            SensorData::Co2(reading) => info!(
                "CO2: {} ppm | Humidity: {}% | Temp: {} C",
                reading.co2_ppm, reading.humidity_percent, reading.temp_celsius
            ),

            SensorData::Aqi(reading) => info!(
                "PM1.0: {} ug/m3 | PM2.5: {} ug/m3 | PM4.0: {} ug/m3 | PM10: {} ug/m3 | \
                 VOC: {} | NOx: {} | Humidity: {}% | Temp: {} C",
                reading.pm1_0,
                reading.pm2_5,
                reading.pm4_0,
                reading.pm10,
                reading.voc_index,
                reading.nox_index,
                reading.humidity_percent,
                reading.temperature_celsius
            ),
        }

        let packet: SensorPacket = data.into();
        let (buffer, length) = packet.encode();

        self.send_adv(&buffer[..length]).await
    }
}

impl BleAdvertiser {
    pub fn new() -> Self {
        Self {}
    }

    async fn send_adv(&mut self, payload: &[u8]) -> Result<(), TransportError> {
        defmt::info!("BLE ADV: {:x}", payload);

        Ok(())
    }
}
