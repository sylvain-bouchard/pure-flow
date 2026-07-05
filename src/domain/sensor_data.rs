#[derive(Clone, Copy)]
pub enum SensorData {
    Hcho(HCOHSensorData),
    Co2(CO2SensorData),
}

#[derive(Clone, Copy)]
pub struct HCOHSensorData {
    pub hcho_ppb: f32,
    pub humidity_percent: f32,
    pub temp_celsius: f32,
}

#[derive(Clone, Copy)]
pub struct CO2SensorData {
    pub co2_ppm: u16,
    pub humidity_percent: f32,
    pub temp_celsius: f32,
}
