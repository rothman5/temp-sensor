use crate::registers::device_id::DeviceId;
use crate::registers::manuf_id::ManufId;
use crate::registers::register::{Read, Register, RegisterPointer, Write};
use crate::registers::resolution::{Resolution, TempRes};
use crate::registers::temperature::Temperature;
use embedded_hal::i2c::{I2c, SevenBitAddress};

const DEFAULT_ADDRESS: u8 = 0x18;

pub enum Address {
    Default,
    Alternate { bit2: bool, bit1: bool, bit0: bool },
}

impl From<Address> for u8 {
    fn from(address: Address) -> Self {
        match address {
            Address::Default => DEFAULT_ADDRESS,
            Address::Alternate { bit2, bit1, bit0 } => {
                DEFAULT_ADDRESS | ((bit2 as u8) << 2) | ((bit1 as u8) << 1) | (bit0 as u8)
            }
        }
    }
}

pub enum Error<E> {
    I2c(E),
    ReadOnlyRegister,
    InvalidRegisterLength,
}

impl<E> From<E> for Error<E> {
    fn from(error: E) -> Self {
        Error::I2c(error)
    }
}

pub struct MCP9808<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C, E> MCP9808<I2C>
where
    I2C: I2c<SevenBitAddress, Error = E>,
{
    pub fn new(i2c: I2C, address: Address) -> Self {
        Self {
            i2c,
            address: address.into(),
        }
    }

    pub fn get_address(&self) -> u8 {
        self.address
    }

    pub fn set_address(&mut self, address: Address) {
        self.address = address.into();
    }

    pub fn get_device_info(&mut self) -> Result<impl DeviceId, Error<E>> {
        let register = Register::new(RegisterPointer::DeviceId, 2);
        self.read_register(register)
    }

    pub fn get_manuf_info(&mut self) -> Result<impl ManufId, Error<E>> {
        let register = Register::new(RegisterPointer::ManufId, 2);
        self.read_register(register)
    }

    pub fn get_temperature(&mut self) -> Result<impl Temperature, Error<E>> {
        let register = Register::new(RegisterPointer::TempAmbient, 2);
        self.read_register(register)
    }

    pub fn set_resolution(&mut self, res: TempRes) -> Result<impl Resolution, Error<E>> {
        let mut register = Register::new(RegisterPointer::Resolution, 2);
        register.set_resolution(res);
        self.write_register(register)
    }

    fn read_register<R: Read>(&mut self, mut register: R) -> Result<R, Error<E>> {
        register.read(&mut self.i2c, self.address)?;
        Ok(register)
    }

    fn write_register<R: Write>(&mut self, register: R) -> Result<R, Error<E>> {
        register.write(&mut self.i2c, self.address)?;
        Ok(register)
    }
}
