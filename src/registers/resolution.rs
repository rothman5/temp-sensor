use crate::registers::register::{Register, RegisterPointer};

#[derive(Debug, Clone, Copy)]
pub struct Resolution {
    pub reg: Register,
}

impl Resolution {
    pub fn new() -> Self {
        Self {
            reg: Register::new(RegisterPointer::Resolution, 1),
        }
    }

    pub fn get_resolution(&self) -> TempRes {
        TempRes::from(self.reg.get_msb())
    }

    pub fn set_resolution(&mut self, res: TempRes) {
        self.reg.set_msb(res as u8);
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TempRes {
    Deg_0_5C = 0b00,
    Deg_0_25C = 0b01,
    Deg_0_125C = 0b10,
    Deg_0_0625C = 0b11,
}

impl From<u8> for TempRes {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0b00 => TempRes::Deg_0_5C,
            0b01 => TempRes::Deg_0_25C,
            0b10 => TempRes::Deg_0_125C,
            0b11 => TempRes::Deg_0_0625C,
            _ => unreachable!("Temperature resolution can only be 2 bits."),
        }
    }
}

pub fn precision_factor(resolution: TempRes) -> f32 {
    match resolution {
        TempRes::Deg_0_5C => 0.5,
        TempRes::Deg_0_25C => 0.25,
        TempRes::Deg_0_125C => 0.125,
        TempRes::Deg_0_0625C => 0.0625,
    }
}
