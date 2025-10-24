use crate::registers::{
    register::{Register, RegisterPointer},
    resolution::{TempRes, precision_factor},
};

const SIGN_BIT: u8 = 0x10;

#[derive(Debug, Clone, Copy)]
pub struct Temperature {
    pub reg: Register,
}

impl Temperature {
    pub fn new() -> Self {
        Self {
            reg: Register::new(RegisterPointer::TempAmbient, 2),
        }
    }

    pub fn get_raw_temp(&self) -> u16 {
        self.reg.as_u16() & 0x1FFF
    }

    pub fn get_celsius(&self, res: TempRes) -> f32 {
        let hi = self.reg.get_msb();
        let lo = self.reg.get_lsb();

        let part_dec = self.get_decimal_part(hi, lo);
        let part_frac = self.get_fractional_part(lo, res);

        (part_dec as f32) + part_frac
    }

    fn get_decimal_part(&self, msb: u8, lsb: u8) -> i16 {
        let mut hi = msb & 0x1F;

        if hi & SIGN_BIT == SIGN_BIT {
            hi &= 0x0F;
            256 - (((hi as i16) << 4) | ((lsb as i16) >> 4))
        } else {
            ((hi as i16) << 4) | ((lsb as i16) >> 4)
        }
    }

    fn get_fractional_part(&self, lsb: u8, res: TempRes) -> f32 {
        let frac = (lsb & 0x0F) >> (3 - (res as u8));
        (frac as f32) * precision_factor(res)
    }
}
