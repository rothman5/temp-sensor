use crate::registers::{
    register::{Read, Register},
    resolution::{TempRes, precision_factor},
};

const SIGN_BIT: u8 = 0x10;

pub trait Temperature: Read {
    fn get_raw_temp(&self) -> u16;
    fn get_celsius(&self, res: TempRes) -> f32;
}

impl Temperature for Register {
    fn get_raw_temp(&self) -> u16 {
        self.as_u16() & 0x1FFF
    }

    fn get_celsius(&self, res: TempRes) -> f32 {
        let hi = self.get_msb();
        let lo = self.get_lsb();

        let part_dec = get_decimal_part(hi, lo);
        let part_frac = get_fractional_part(lo, res);

        (part_dec as f32) + part_frac
    }
}

fn get_decimal_part(msb: u8, lsb: u8) -> i16 {
    let mut hi = msb & 0x1F;

    if hi & SIGN_BIT == SIGN_BIT {
        hi &= 0x0F;
        256 - (((hi as i16) << 4) | ((lsb as i16) >> 4))
    } else {
        ((hi as i16) << 4) | ((lsb as i16) >> 4)
    }
}

fn get_fractional_part(lsb: u8, res: TempRes) -> f32 {
    let frac = (lsb & 0x0F) >> (3 - (res as u8));
    (frac as f32) * precision_factor(res)
}
