use crate::registers::register::{Read, Register};

const MANUF_ID: u16 = 0x0054;

pub trait ManufId: Read {
    fn get_manuf_id(&self) -> u16;
    fn is_valid_manuf(&self) -> bool;
}

impl ManufId for Register {
    fn get_manuf_id(&self) -> u16 {
        self.as_u16()
    }

    fn is_valid_manuf(&self) -> bool {
        self.get_manuf_id() == MANUF_ID
    }
}
