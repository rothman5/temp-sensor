use crate::registers::register::{Register, RegisterPointer};

const DEVICE_ID: u8 = 0x04;

#[derive(Debug, Clone, Copy)]
pub struct DevInfoReg {
    pub reg: Register,
}

impl DevInfoReg {
    pub fn new() -> Self {
        Self {
            reg: Register::new(RegisterPointer::DeviceId, 2),
        }
    }

    pub fn get_device_id(&self) -> u8 {
        self.reg.get_msb()
    }

    pub fn get_device_rev(&self) -> u8 {
        self.reg.get_lsb()
    }

    pub fn is_valid_device(&self) -> bool {
        self.get_device_id() == DEVICE_ID
    }
}
