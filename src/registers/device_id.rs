use crate::registers::register::{Read, Register};

const DEVICE_ID: u8 = 0x04;

pub trait DeviceId: Read {
    fn get_device_id(&self) -> u8;
    fn get_device_rev(&self) -> u8;
    fn is_valid_device(&self) -> bool;
}

impl DeviceId for Register {
    fn get_device_id(&self) -> u8 {
        self.get_msb()
    }

    fn get_device_rev(&self) -> u8 {
        self.get_lsb()
    }

    fn is_valid_device(&self) -> bool {
        self.get_device_id() == DEVICE_ID
    }
}
