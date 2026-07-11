#[derive(Clone, Copy)]
pub enum SensorData {
    Hcho(HCOHSensorData),
    Co2(CO2SensorData),
    Aqi(AQISensorData),
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

#[derive(Clone, Copy)]
pub struct AQISensorData {
    pub pm1_0: f32,
    pub pm2_5: f32,
    pub pm4_0: f32,
    pub pm10: f32,

    pub humidity_percent: f32,
    pub temperature_celsius: f32,

    pub voc_index: f32,
    pub nox_index: f32,
}