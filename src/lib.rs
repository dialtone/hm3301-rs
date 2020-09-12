// May add missing_docs
#![deny(unsafe_code)]
#![no_std]

use core::convert::TryInto;
use core::fmt;
use embedded_hal::blocking::i2c;
use nb;

pub const DEFAULT_I2C_ADDR: u8 = 0x40;
pub const SELECT_I2C_CMD: u8 = 0x88;

#[derive(Debug)]
pub enum Error<E> {
    /// I²C communication error
    I2C(E),
    /// Invalid input data provided
    InvalidInputData,
    /// ChecksumFailed from bus read value
    ChecksumFailed,
}

#[derive(Debug)]
pub struct Hm3301<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C> Hm3301<I2C> {
    /// Create new instance of the device.
    pub fn new(i2c: I2C) -> Self {
        Hm3301 {
            i2c,
            address: DEFAULT_I2C_ADDR,
        }
    }
}

impl<I2C, E> Hm3301<I2C>
where
    I2C: i2c::Write<Error = E>,
{
    pub fn enable_i2c(&mut self) -> nb::Result<(), Error<E>> {
        let payload = [SELECT_I2C_CMD];
        Ok(self.i2c.write(self.address, &payload).map_err(Error::I2C)?)
    }
}

pub struct Measurement {
    pub num_sensor: u16,
    pub std_pm1: u16,
    pub std_pm25: u16,
    pub std_pm10: u16,
    pub atm_pm1: u16,
    pub atm_pm25: u16,
    pub atm_pm10: u16,
}

impl fmt::Display for Measurement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Sensor Number: {}\n\
            Std PM 1: {}; Atm PM 1: {}\n\
            Std PM 2.5: {}; Atm PM 2.5: {}\n\
            Std PM 10: {}; Atm PM 10: {}",
            self.num_sensor,
            self.std_pm1,
            self.atm_pm1,
            self.std_pm25,
            self.atm_pm25,
            self.std_pm10,
            self.atm_pm10
        )
    }
}

impl From<[u16; 7]> for Measurement {
    fn from(arr: [u16; 7]) -> Self {
        Measurement {
            num_sensor: arr[0],
            std_pm1: arr[1],
            std_pm25: arr[2],
            std_pm10: arr[3],
            atm_pm1: arr[4],
            atm_pm25: arr[5],
            atm_pm10: arr[6],
        }
    }
}

impl<I2C, E> Hm3301<I2C>
where
    I2C: i2c::Read<Error = E>,
{
    pub fn read_measurement(&mut self) -> Result<Measurement, Error<E>> {
        let mut buf: [u8; 29] = [0; 29];
        self.i2c.read(self.address, &mut buf).map_err(Error::I2C)?;

        // checksum, sum 0..28 and validate against 28.
        let mut sum: u8 = 0;
        for i in 0..=27 {
            sum += i;
        }
        if sum != buf[28] {
            return Err(Error::ChecksumFailed);
        }

        // bytes 2 through 15 contain the sensor number and the reading
        let mut res: [u16; 7] = [0; 7];
        for (i, chunk) in buf[2..16].chunks_exact(2).enumerate() {
            res[i] = u16::from_ne_bytes(chunk.try_into().unwrap());
        }

        Ok(Measurement::from(res))
    }
}
