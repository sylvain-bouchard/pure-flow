use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_nrf::{Peri, mode::Async, peripherals::*};
use embassy_nrf::{bind_interrupts, rng};
use nrf_sdc::mpsl::MultiprotocolServiceLayer;
use nrf_sdc::{self as sdc, Error, SoftdeviceController, mpsl};
use static_cell::StaticCell;

const L2CAP_TXQ: u8 = 4;
const L2CAP_RXQ: u8 = 4;
const L2CAP_MTU: usize = 251;

// Bind the hardware interrupts locally to this file instead of main.rs
bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<RNG>;
    EGU0_SWI0 => nrf_sdc::mpsl::LowPrioInterruptHandler;
    CLOCK_POWER => nrf_sdc::mpsl::ClockInterruptHandler;
    RADIO => nrf_sdc::mpsl::HighPrioInterruptHandler;
    TIMER0 => nrf_sdc::mpsl::HighPrioInterruptHandler;
    RTC0 => nrf_sdc::mpsl::HighPrioInterruptHandler;
});

pub struct RadioPeripherals {
    pub rng: Peri<'static, RNG>,

    pub rtc0: Peri<'static, RTC0>,
    pub timer0: Peri<'static, TIMER0>,
    pub temp: Peri<'static, TEMP>,

    pub ppi_ch17: Peri<'static, PPI_CH17>,
    pub ppi_ch18: Peri<'static, PPI_CH18>,
    pub ppi_ch19: Peri<'static, PPI_CH19>,
    pub ppi_ch20: Peri<'static, PPI_CH20>,
    pub ppi_ch21: Peri<'static, PPI_CH21>,
    pub ppi_ch22: Peri<'static, PPI_CH22>,
    pub ppi_ch23: Peri<'static, PPI_CH23>,
    pub ppi_ch24: Peri<'static, PPI_CH24>,
    pub ppi_ch25: Peri<'static, PPI_CH25>,
    pub ppi_ch26: Peri<'static, PPI_CH26>,
    pub ppi_ch27: Peri<'static, PPI_CH27>,
    pub ppi_ch28: Peri<'static, PPI_CH28>,
    pub ppi_ch29: Peri<'static, PPI_CH29>,
    pub ppi_ch30: Peri<'static, PPI_CH30>,
    pub ppi_ch31: Peri<'static, PPI_CH31>,
}

fn build_sdc<'d, const N: usize>(
    p: nrf_sdc::Peripherals<'d>,
    rng: &'d mut rng::Rng<Async>,
    mpsl: &'d MultiprotocolServiceLayer,
    mem: &'d mut sdc::Mem<N>,
) -> Result<SoftdeviceController<'d>, Error> {
    sdc::Builder::new()?
        .support_adv()
        .support_peripheral()
        .peripheral_count(1)?
        .buffer_cfg(L2CAP_MTU as u16, L2CAP_MTU as u16, L2CAP_TXQ, L2CAP_RXQ)?
        .build(p, rng, mpsl, mem)
}

#[embassy_executor::task]
async fn mpsl_task(mpsl: &'static MultiprotocolServiceLayer<'static>) -> ! {
    mpsl.run().await
}

/// Sets up the low-level Nordic radio hardware layers (MPSL & SDC)
/// and returns an initialized SoftdeviceController.
pub async fn setup_radio_hardware(
    peripherals: RadioPeripherals,
    spawner: Spawner,
) -> nrf_sdc::SoftdeviceController<'static> {
    // 1. Setup MPSL
    let mpsl_p = mpsl::Peripherals::new(
        peripherals.rtc0,
        peripherals.timer0,
        peripherals.temp,
        peripherals.ppi_ch19,
        peripherals.ppi_ch30,
        peripherals.ppi_ch31,
    );
    let lfclk_cfg = mpsl::raw::mpsl_clock_lfclk_cfg_t {
        source: mpsl::raw::MPSL_CLOCK_LF_SRC_RC as u8,
        rc_ctiv: mpsl::raw::MPSL_RECOMMENDED_RC_CTIV as u8,
        rc_temp_ctiv: mpsl::raw::MPSL_RECOMMENDED_RC_TEMP_CTIV as u8,
        accuracy_ppm: mpsl::raw::MPSL_DEFAULT_CLOCK_ACCURACY_PPM as u16,
        skip_wait_lfclk_started: mpsl::raw::MPSL_DEFAULT_SKIP_WAIT_LFCLK_STARTED != 0,
    };

    static MPSL: StaticCell<MultiprotocolServiceLayer> = StaticCell::new();
    let mpsl = MPSL.init(defmt::unwrap!(mpsl::MultiprotocolServiceLayer::new(
        mpsl_p, Irqs, lfclk_cfg
    )));
    spawner.spawn(unwrap!(mpsl_task(&*mpsl)));

    // 2. Setup SDC
    let sdc_p = sdc::Peripherals::new(
        peripherals.ppi_ch17,
        peripherals.ppi_ch18,
        peripherals.ppi_ch20,
        peripherals.ppi_ch21,
        peripherals.ppi_ch22,
        peripherals.ppi_ch23,
        peripherals.ppi_ch24,
        peripherals.ppi_ch25,
        peripherals.ppi_ch26,
        peripherals.ppi_ch27,
        peripherals.ppi_ch28,
        peripherals.ppi_ch29,
    );

    static RNG: StaticCell<rng::Rng<Async>> = StaticCell::new();
    let rng = RNG.init(rng::Rng::new(peripherals.rng, Irqs));

    static SDC_MEM: StaticCell<sdc::Mem<16384>> = StaticCell::new();
    let sdc_mem = SDC_MEM.init(sdc::Mem::<16384>::new());

    let sdc = unwrap!(build_sdc(sdc_p, rng, mpsl, sdc_mem));

    sdc
}
