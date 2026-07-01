#![no_std]
#![no_main]

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_nrf::twim::Twim;
use embassy_nrf::{bind_interrupts, twim};
use embassy_time::{Duration, Instant, Timer};
use static_cell::StaticCell;

use defmt_rtt as _;
use panic_probe as _;

defmt::timestamp!("{=u64:ms}", { Instant::now().as_millis() as u64 });

const SFA30_ADDR: u8 = 0x5D;

// Correct SFA30 commands
const CMD_START_CONTINUOUS: [u8; 2] = [0x00, 0x06];
const CMD_READ_VALUES: [u8; 2] = [0x03, 0x27];

bind_interrupts!(struct Irqs {
    TWISPI0 => twim::InterruptHandler<embassy_nrf::peripherals::TWISPI0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    defmt::info!("Starting firmware");

    let peripherals = embassy_nrf::init(Default::default());
    let config = twim::Config::default();

    static TX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    let tx_buf = TX_BUF.init([0; 16]);

    // -----------------------------
    // Create TWIM driver
    // -----------------------------
    let mut i2c = Twim::new(
        peripherals.TWISPI0,
        Irqs,
        peripherals.P0_04, // SDA
        peripherals.P0_05, // SCL
        config,
        tx_buf,
    );

    // Let sensor boot
    Timer::after(Duration::from_millis(500)).await;

    // -----------------------------
    // Start continuous measurement
    // -----------------------------
    match i2c.write(SFA30_ADDR, &CMD_START_CONTINUOUS).await {
        Ok(_) => info!("SFA30 measurement started"),
        Err(_) => error!("I2C transaction failed"),
    }

    let mut buffer = [0u8; 9]; // adjust once datasheet confirmed

    loop {
        Timer::after(Duration::from_secs(1)).await;

        // Step 1: send read command
        if let Err(_) = i2c.write(SFA30_ADDR, &CMD_READ_VALUES).await {
            error!("I2C write failed");
            continue;
        }

        Timer::after(Duration::from_millis(50)).await;

        // Step 2: read response
        match i2c.read(SFA30_ADDR, &mut buffer).await {
            Ok(_) => {
                info!("raw bytes: {}", buffer);
            }
            Err(_) => {
                error!("I2C read failed");
            }
        }
    }
}
