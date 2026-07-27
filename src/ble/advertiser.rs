use crate::ble::sensor_packet::EnvironmentPacket;
use crate::domain::sensor_data::EnvironmentData;
use crate::transport::{TelemetryTransport, TransportError};

use embassy_executor::Spawner;
// -----------------------------------------------------------------------------
// BLE & Trouble
// -----------------------------------------------------------------------------
use nrf_sdc::SoftdeviceController;
use static_cell::StaticCell;
use trouble_host::prelude::*;

// -----------------------------------------------------------------------------
// Logging / panic
// -----------------------------------------------------------------------------
use defmt::info;

const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 1;

static RESOURCES: StaticCell<
    HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX>,
> = StaticCell::new();
static STACK: StaticCell<Stack<'static, SoftdeviceController<'static>, DefaultPacketPool>> =
    StaticCell::new();

#[derive(Debug, defmt::Format)]
pub enum BleAdvertiserError {
    SpawnBackgroundTask,
}

pub struct BleAdvertiser {
    peripheral: Peripheral<'static, SoftdeviceController<'static>, DefaultPacketPool>,
}

impl BleAdvertiser {
    pub fn new(
        controller: SoftdeviceController<'static>,
        spawner: Spawner,
    ) -> Result<Self, BleAdvertiserError> {
        let resources = RESOURCES.init(HostResources::new());
        let address = Address::random([0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa]);

        let stack = trouble_host::new(controller, resources)
            .set_random_address(address)
            .build();

        let stack: &'static mut Stack<'static, SoftdeviceController<'static>, DefaultPacketPool> =
            STACK.init(stack);

        let token = trouble_background_task(stack.runner())
            .map_err(|_| BleAdvertiserError::SpawnBackgroundTask)?;
        spawner.spawn(token);

        Ok(Self {
            peripheral: stack.peripheral(),
        })
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

#[embassy_executor::task]
async fn trouble_background_task(
    mut runner: Runner<'static, SoftdeviceController<'static>, DefaultPacketPool>,
) {
    loop {
        let _ = runner.run().await;
    }
}
