use crate::domain::sensor_data::EnvironmentData;

#[derive(Debug, defmt::Format)]
pub enum TransportError {
    Ble,
    Encode,
}

pub trait TelemetryTransport {
    async fn send(&mut self, data: EnvironmentData) -> Result<(), TransportError>;
}