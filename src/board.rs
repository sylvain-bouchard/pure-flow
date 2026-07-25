use embassy_nrf::{Peri, peripherals::*};

use crate::ble::nordic::RadioPeripherals;

pub struct BoardPeripherals {
    pub radio: RadioPeripherals,

    pub twim: Peri<'static, TWISPI0>,
    pub sda: Peri<'static, P0_04>,
    pub scl: Peri<'static, P0_05>,
}

pub fn split_peripherals(p: embassy_nrf::Peripherals) -> BoardPeripherals {
    let embassy_nrf::Peripherals {
        // ---------------------------------------------------------------------
        // Radio / MPSL / SDC
        // ---------------------------------------------------------------------
        RNG,
        RTC0,
        TIMER0,
        TEMP,

        PPI_CH17,
        PPI_CH18,
        PPI_CH19,
        PPI_CH20,
        PPI_CH21,
        PPI_CH22,
        PPI_CH23,
        PPI_CH24,
        PPI_CH25,
        PPI_CH26,
        PPI_CH27,
        PPI_CH28,
        PPI_CH29,
        PPI_CH30,
        PPI_CH31,

        // ---------------------------------------------------------------------
        // I2C
        // ---------------------------------------------------------------------
        TWISPI0,
        P0_04,
        P0_05,
        // ---------------------------------------------------------------------
        // Unused peripherals
        // ---------------------------------------------------------------------
        ..
    } = p;

    BoardPeripherals {
        radio: RadioPeripherals {
            rng: RNG,

            rtc0: RTC0,
            timer0: TIMER0,
            temp: TEMP,

            ppi_ch17: PPI_CH17,
            ppi_ch18: PPI_CH18,
            ppi_ch19: PPI_CH19,
            ppi_ch20: PPI_CH20,
            ppi_ch21: PPI_CH21,
            ppi_ch22: PPI_CH22,
            ppi_ch23: PPI_CH23,
            ppi_ch24: PPI_CH24,
            ppi_ch25: PPI_CH25,
            ppi_ch26: PPI_CH26,
            ppi_ch27: PPI_CH27,
            ppi_ch28: PPI_CH28,
            ppi_ch29: PPI_CH29,
            ppi_ch30: PPI_CH30,
            ppi_ch31: PPI_CH31,
        },

        twim: TWISPI0,
        sda: P0_04,
        scl: P0_05,
    }
}
