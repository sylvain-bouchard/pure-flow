use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::{Mutex};

use embedded_hal_async::i2c::{
    Error,
    ErrorKind,
    ErrorType,
    I2c,
    Operation,
};

use embassy_nrf::twim::Twim;


pub struct SharedI2cBus {
    bus: &'static Mutex<CriticalSectionRawMutex, Twim<'static>>,
}


impl SharedI2cBus {
    pub fn new(
        bus: &'static Mutex<CriticalSectionRawMutex, Twim<'static>>,
    ) -> Self {
        Self { bus }
    }
}


impl Error for TwimError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}


#[derive(Debug)]
pub struct TwimError;


impl ErrorType for SharedI2cBus {
    type Error = TwimError;
}


impl I2c for SharedI2cBus {

    async fn read(
        &mut self,
        address: u8,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {

        let mut bus = self.bus.lock().await;

        bus.read(address, buffer)
            .await
            .map_err(|_| TwimError)
    }


    async fn write(
        &mut self,
        address: u8,
        bytes: &[u8],
    ) -> Result<(), Self::Error> {

        let mut bus = self.bus.lock().await;

        bus.write(address, bytes)
            .await
            .map_err(|_| TwimError)
    }


    async fn write_read(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {

        let mut bus = self.bus.lock().await;

        bus.write_read(address, bytes, buffer)
            .await
            .map_err(|_| TwimError)
    }


    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {

        let mut bus = self.bus.lock().await;

        bus.transaction(address, operations)
            .await
            .map_err(|_| TwimError)
    }
}