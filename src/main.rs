#![no_std]
#![no_main]

// -----------------------------------------------------------------------------
// Core
// -----------------------------------------------------------------------------

use core::future::pending;

// -----------------------------------------------------------------------------
// Embassy
// -----------------------------------------------------------------------------

use embassy_executor::Spawner;
use embassy_nrf::twim::Twim;
use embassy_nrf::{bind_interrupts, twim};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Instant, Timer};

// -----------------------------------------------------------------------------
// Logging / panic
// -----------------------------------------------------------------------------

use defmt::{error, info};
use defmt_rtt as _;
use panic_probe as _;

// -----------------------------------------------------------------------------
// Utilities
// -----------------------------------------------------------------------------

use static_cell::StaticCell;

// -----------------------------------------------------------------------------
// Local modules
// -----------------------------------------------------------------------------

mod ble;
mod domain;
mod i2c_bus;
mod sensors;
mod transport;

// -----------------------------------------------------------------------------
// Local imports
// -----------------------------------------------------------------------------

use crate::ble::advertiser::BleAdvertiser;
use crate::domain::sensor_data::SensorData;
use crate::i2c_bus::SharedI2cBus;
use crate::sensors::scd40::Scd40;
use crate::sensors::sfa30::Sfa30;
use crate::transport::TelemetryTransport;

// -----------------------------------------------------------------------------
// Global resources
// -----------------------------------------------------------------------------

pub static SENSOR_CHANNEL: Channel<CriticalSectionRawMutex, SensorData, 8> = Channel::new();

static I2C_BUS: StaticCell<Mutex<CriticalSectionRawMutex, Twim<'static>>> = StaticCell::new();

// -----------------------------------------------------------------------------
// Embassy configuration
// -----------------------------------------------------------------------------

defmt::timestamp!("{=u64:ms}", { Instant::now().as_millis() as u64 });

bind_interrupts!(struct Irqs {
    TWISPI0 => twim::InterruptHandler<embassy_nrf::peripherals::TWISPI0>;
});

// -----------------------------------------------------------------------------
// Tasks
// -----------------------------------------------------------------------------

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    defmt::info!("Starting firmware");

    let peripherals = embassy_nrf::init(Default::default());
    let twim_config = twim::Config::default();

    static TX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    let tx_buf = TX_BUF.init([0; 16]);

    // Create TWIM driver
    let i2c = Twim::new(
        peripherals.TWISPI0,
        Irqs,
        peripherals.P0_04, // SDA
        peripherals.P0_05, // SCL
        twim_config,
        tx_buf,
    );

    let mutex = I2C_BUS.init(Mutex::new(i2c));

    let scd40_i2c = SharedI2cBus::new(mutex);
    spawner.spawn(scd40_task(scd40_i2c).unwrap());

    let sfa30_i2c = SharedI2cBus::new(mutex);
    spawner.spawn(sfa30_task(sfa30_i2c).unwrap());

    let advertiser = BleAdvertiser::new();
    spawner.spawn(ble_transmission_task(advertiser).unwrap());

    pending::<()>().await;
}

#[embassy_executor::task]
async fn scd40_task(i2c: SharedI2cBus) {
    let mut sensor = Scd40::new(i2c);

    if let Err(error) = sensor.start().await {
        error!("SCD40 start failed: {:?}", error);
        return;
    }

    loop {
        match sensor.data_ready().await {
            Ok(true) => match sensor.read().await {
                Ok(reading) => {
                    SENSOR_CHANNEL.send(SensorData::Co2(reading)).await;
                }
                Err(error) => error!("SCD40 read failed: {:?}", error),
            },

            Ok(false) => {}

            Err(error) => {
                error!("SCD40 ready check failed: {:?}", error);
            }
        }

        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
async fn sfa30_task(i2c: SharedI2cBus) {
    let mut sensor = Sfa30::new(i2c);

    if let Err(error) = sensor.start().await {
        error!("SFA30 start failed: {}", error);

        loop {
            Timer::after(Duration::from_secs(5)).await;

            match sensor.start().await {
                Ok(_) => {
                    info!("SFA30 started successfully");
                    break;
                }
                Err(error) => {
                    error!("SFA30 retry failed: {}", error);
                }
            }
        }
    }

    Timer::after(Duration::from_secs(1)).await;

    loop {
        match sensor.read().await {
            Ok(reading) => {
                SENSOR_CHANNEL.send(SensorData::Hcho(reading)).await;
            }

            Err(error) => {
                error!("SFA30 read failed: {:?}", error);
            }
        }

        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
pub async fn ble_transmission_task(mut advertiser: BleAdvertiser) {
    let receiver = SENSOR_CHANNEL.receiver();

    loop {
        let data = receiver.receive().await;

        if let Err(error) = advertiser.send(data).await {
            defmt::error!("BLE send failed: {:?}", error);
        }
    }
}
