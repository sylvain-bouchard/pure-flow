use crate::domain::sensor_data::SensorData;

#[derive(Debug, defmt::Format)]
pub enum TransportError {
    Ble,
    Encode,
}

pub trait TelemetryTransport {
    async fn send(&mut self, data: SensorData) -> Result<(), TransportError>;
}