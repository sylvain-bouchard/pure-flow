use crate::ble::sensor_packet::EnvironmentPacket;
use crate::domain::sensor_data::EnvironmentData;
use crate::transport::{TelemetryTransport, TransportError};

use defmt::info;

pub struct BleAdvertiser {}

impl BleAdvertiser {
    pub fn new() -> Self {
        Self {}
    }

    async fn send_adv(&mut self, payload: &[u8]) -> Result<(), TransportError> {
        info!("BLE ADV: {:x}", payload);

        Ok(())
    }
}

impl TelemetryTransport for BleAdvertiser {
    async fn send(&mut self, environment: EnvironmentData) -> Result<(), TransportError> {
        info!(
            "CO2: {:?} ppm | HCHO: {:?} ppb | \
             PM1.0: {:?} ug/m3 | PM2.5: {:?} ug/m3 | \
             PM4.0: {:?} ug/m3 | PM10: {:?} ug/m3 | \
             VOC: {:?} | NOx: {:?} | \
             Humidity: {:?}% | Temp: {:?} C",
            environment.co2_ppm,
            environment.hcho_ppb,
            environment.pm1_0,
            environment.pm2_5,
            environment.pm4_0,
            environment.pm10,
            environment.voc_index,
            environment.nox_index,
            environment.humidity_percent,
            environment.temperature_celsius
        );

        let packet = EnvironmentPacket::from(environment);

        let payload = packet.encode();

        self.send_adv(&payload).await
    }
}
