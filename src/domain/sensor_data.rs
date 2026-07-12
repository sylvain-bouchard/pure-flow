#[derive(Clone, Copy)]
pub struct EnvironmentData {
    pub co2_ppm: Option<u16>,
    pub hcho_ppb: Option<u16>,

    pub pm1_0: Option<u16>,
    pub pm2_5: Option<u16>,
    pub pm4_0: Option<u16>,
    pub pm10: Option<u16>,

    pub voc_index: Option<u16>,
    pub nox_index: Option<u16>,

    pub temperature_celsius: Option<f32>,
    pub humidity_percent: Option<f32>,
}

impl EnvironmentData {
    pub const fn new() -> Self {
        Self {
            co2_ppm: None,
            hcho_ppb: None,

            pm1_0: None,
            pm2_5: None,
            pm4_0: None,
            pm10: None,

            voc_index: None,
            nox_index: None,

            humidity_percent: None,
            temperature_celsius: None,
        }
    }
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
