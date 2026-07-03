#![no_std]
#![no_main]

use core::future::pending;
use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_nrf::twim::Twim;
use embassy_nrf::{bind_interrupts, twim};
use embassy_time::{Duration, Instant, Timer};
use static_cell::StaticCell;

use defmt_rtt as _;
use panic_probe as _;

mod sfa30;

use sfa30::{CMD_READ_VALUES, CMD_START_CONTINUOUS, SFA30_ADDR, SensorData, decode};

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

static SENSOR_CHANNEL: Channel<CriticalSectionRawMutex, SensorData, 4> = Channel::new();

defmt::timestamp!("{=u64:ms}", { Instant::now().as_millis() as u64 });

bind_interrupts!(struct Irqs {
    TWISPI0 => twim::InterruptHandler<embassy_nrf::peripherals::TWISPI0>;
});

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

    spawner.spawn(sensor_reading_task(i2c).unwrap());
    spawner.spawn(ble_transmission_task().unwrap());

    pending::<()>().await;
}

#[embassy_executor::task]
async fn sensor_reading_task(mut i2c: Twim<'static>) {
    Timer::after(Duration::from_millis(500)).await;

    // Start continuous measurement
    match i2c.write(SFA30_ADDR, &CMD_START_CONTINUOUS).await {
        Ok(_) => info!("SFA30 measurement started"),
        Err(_) => error!("I2C transaction failed"),
    }

    let mut buffer = [0u8; 9];

    loop {
        Timer::after(Duration::from_secs(1)).await;

        if i2c.write(SFA30_ADDR, &CMD_READ_VALUES).await.is_err() {
            error!("I2C write failed");
            continue;
        }

        Timer::after(Duration::from_millis(50)).await;

        match i2c.read(SFA30_ADDR, &mut buffer).await {
            Ok(_) => match decode(&buffer) {
                Ok(reading) => {
                    SENSOR_CHANNEL.send(reading).await;
                }
                Err(error) => {
                    error!("Decode failed: {}", error);
                }
            },
            Err(_) => {
                error!("I2C read failed");
            }
        }
    }
}

#[embassy_executor::task]
async fn ble_transmission_task() {
    loop {
        // Wait for the latest sensor value
        let data = SENSOR_CHANNEL.receive().await;

        // Log for now (replace later with BLE advertising)
        defmt::info!(
            "BLE TX -> HCHO: {} ppb | RH: {} % | Temp: {} °C",
            data.hcho_ppb,
            data.humidity_percent,
            data.temp_celsius
        );

        // TODO: replace this with actual BLE update:
        // - advertisement payload update
        // - GATT characteristic notify
        // - or Nordic UART service send

        Timer::after(embassy_time::Duration::from_millis(100)).await;
    }
}
