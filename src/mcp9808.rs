use crate::registers::device_id::DevInfoReg;
use crate::registers::manuf_id::ManInfoReg;
use crate::registers::resolution::{ResReg, TempRes};
use crate::registers::temperature::TempReg;
use embedded_hal_async::i2c::{I2c, SevenBitAddress};

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

#[derive(Debug)]
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

pub struct MCP9808 {
    address: u8,
}

impl MCP9808 {
    pub fn new(address: Address) -> Self {
        Self {
            address: address.into(),
        }
    }

    pub fn get_address(&self) -> u8 {
        self.address
    }

    pub fn set_address(&mut self, address: Address) {
        self.address = address.into();
    }

    pub async fn get_device_info<I2C>(&self, i2c: &mut I2C) -> Result<DevInfoReg, Error<I2C::Error>>
    where
        I2C: I2c<SevenBitAddress>,
    {
        let mut dev_info = DevInfoReg::new();
        dev_info.reg.read(i2c, self.address).await?;
        Ok(dev_info)
    }

    pub async fn get_manuf_info<I2C>(&self, i2c: &mut I2C) -> Result<ManInfoReg, Error<I2C::Error>>
    where
        I2C: I2c<SevenBitAddress>,
    {
        let mut manuf_info = ManInfoReg::new();
        manuf_info.reg.read(i2c, self.address).await?;
        Ok(manuf_info)
    }

    pub async fn get_temp<I2C>(&self, i2c: &mut I2C) -> Result<TempReg, Error<I2C::Error>>
    where
        I2C: I2c<SevenBitAddress>,
    {
        let mut temp = TempReg::new();
        temp.reg.read(i2c, self.address).await?;
        Ok(temp)
    }

    pub async fn set_res<I2C>(&self, i2c: &mut I2C, r: TempRes) -> Result<ResReg, Error<I2C::Error>>
    where
        I2C: I2c<SevenBitAddress>,
    {
        let mut resolution = ResReg::new();
        resolution.set_resolution(r);
        resolution.reg.write(i2c, self.address).await?;
        Ok(resolution)
    }
}
