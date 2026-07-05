use crate::ble::sensor_packet::SensorPacket;
use crate::transport::TransportError;
use crate::{domain::sensor_data::SensorData, transport::TelemetryTransport};

use defmt::{error, info};

pub struct BleAdvertiser {}

impl TelemetryTransport for BleAdvertiser {
    async fn send(&mut self, data: SensorData) -> Result<(), TransportError> {
        let packet: SensorPacket = data.into();
        let payload = packet.encode();

        self.send_adv(&payload).await
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
