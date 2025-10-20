use crate::mcp9808::Error;
use core::fmt::Debug;
use embedded_hal::i2c::{I2c, SevenBitAddress};

#[derive(Debug, Clone, Copy)]
pub enum RegisterPointer {
    Config = 0x01,
    TempUpper = 0x02,
    TempLower = 0x03,
    TempCritical = 0x04,
    TempAmbient = 0x05,
    ManufId = 0x06,
    DeviceId = 0x07,
    Resolution = 0x08,
}

impl RegisterPointer {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn is_read_only(self) -> bool {
        matches!(
            self,
            RegisterPointer::TempAmbient | RegisterPointer::ManufId | RegisterPointer::DeviceId
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Register {
    ptr: RegisterPointer,
    buf: [u8; 2],
    len: usize,
}

impl Register {
    pub fn new(ptr: RegisterPointer, len: usize) -> Self {
        Self {
            ptr,
            buf: [0; 2],
            len,
        }
    }

    pub fn get_ptr(&self) -> u8 {
        self.ptr.as_u8()
    }

    pub fn get_buf(&self) -> &[u8] {
        &self.buf[0..self.len]
    }

    pub fn set_buf(&mut self, data: [u8; 2]) {
        self.buf = data;
    }

    pub fn get_buf_mut(&mut self) -> &mut [u8] {
        &mut self.buf[0..self.len]
    }

    pub fn get_len(&self) -> usize {
        self.len
    }

    pub fn is_read_only(&self) -> bool {
        self.ptr.is_read_only()
    }

    pub fn get_lsb(&self) -> u8 {
        self.buf[1]
    }

    pub fn get_msb(&self) -> u8 {
        self.buf[0]
    }

    pub fn set_lsb(&mut self, lsb: u8) {
        self.buf[1] = lsb;
    }

    pub fn set_msb(&mut self, msb: u8) {
        self.buf[0] = msb;
    }

    pub fn get_bit(&self, bit: usize) -> bool {
        if bit >= self.len * 8 {
            panic!(
                "Bit {} out of range for register of length {}",
                bit, self.len
            );
        }

        if bit < 8 {
            (self.get_lsb() & (1 << bit)) != 0
        } else {
            (self.get_msb() & (1 << (bit - 8))) != 0
        }
    }

    pub fn set_bit(&mut self, bit: usize, value: bool) {
        if bit >= self.len * 8 {
            panic!(
                "Bit {} out of range for register of length {}",
                bit, self.len
            );
        }

        if bit < 8 {
            if value {
                self.set_lsb(self.get_lsb() | (1 << bit));
            } else {
                self.set_lsb(self.get_lsb() & !(1 << bit));
            }
        } else {
            if value {
                self.set_msb(self.get_msb() | (1 << (bit - 8)));
            } else {
                self.set_msb(self.get_msb() & !(1 << (bit - 8)));
            }
        }
    }

    pub fn as_u16(&self) -> u16 {
        u16::from_le_bytes(self.buf)
    }

    pub fn from_u16(&mut self, value: u16) {
        self.buf = value.to_le_bytes();
    }
}

pub trait Read: Debug + Copy + Clone {
    fn read<I2C, E>(&mut self, i2c: &mut I2C, address: u8) -> Result<(), Error<E>>
    where
        I2C: I2c<SevenBitAddress, Error = E>;
}

impl Read for Register {
    fn read<I2C, E>(&mut self, i2c: &mut I2C, address: u8) -> Result<(), Error<E>>
    where
        I2C: I2c<SevenBitAddress, Error = E>,
    {
        if self.get_len() == 0 || self.get_len() > 2 {
            return Err(Error::InvalidRegisterLength);
        }

        i2c.write_read(address, &[self.get_ptr()], self.get_buf_mut())?;
        Ok(())
    }
}

pub trait Write: Read {
    fn write<I2C, E>(&self, i2c: &mut I2C, address: u8) -> Result<(), Error<E>>
    where
        I2C: I2c<SevenBitAddress, Error = E>;
}

impl Write for Register {
    fn write<I2C, E>(&self, i2c: &mut I2C, address: u8) -> Result<(), Error<E>>
    where
        I2C: I2c<SevenBitAddress, Error = E>,
    {
        if self.get_len() == 0 || self.get_len() > 2 {
            return Err(Error::InvalidRegisterLength);
        }

        if self.is_read_only() {
            return Err(Error::ReadOnlyRegister);
        }

        let mut buf = [0u8; 3];
        buf[0] = self.get_ptr();
        buf[1..(1 + self.get_len())].copy_from_slice(self.get_buf());

        i2c.write(address, &buf[0..(1 + self.get_len())])?;
        Ok(())
    }
}
