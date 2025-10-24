use crate::registers::register::{Register, RegisterPointer};

const MANUF_ID: u16 = 0x0054;

#[derive(Debug, Clone, Copy)]
pub struct ManufInfo {
    pub reg: Register,
}

impl ManufInfo {
    pub fn new() -> Self {
        Self {
            reg: Register::new(RegisterPointer::ManufId, 2),
        }
    }

    pub fn get_manuf_id(&self) -> u16 {
        self.reg.as_u16()
    }

    pub fn is_valid_manuf(&self) -> bool {
        self.get_manuf_id() == MANUF_ID
    }
}
