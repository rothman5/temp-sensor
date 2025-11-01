extern crate alloc;

use alloc::format;
use alloc::string::String;
use embassy_time::Instant;

#[derive(Copy, Clone)]
pub struct Measurement {
    pub timestamp: Instant,
    pub in_temp_c: f32,
    pub out_temp_c: f32,
}

impl Measurement {
    pub fn to_bytes(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[0..8].copy_from_slice(&self.timestamp.as_ticks().to_le_bytes());
        bytes[8..12].copy_from_slice(&self.in_temp_c.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.out_temp_c.to_le_bytes());
        bytes
    }

    pub fn to_string(&self) -> String {
        format!(
            "Timestamp: {} sec, Inside: {:.2} °C, Outside: {:.2} °C",
            self.timestamp.as_secs(),
            self.in_temp_c,
            self.out_temp_c
        )
    }
}

impl Default for Measurement {
    fn default() -> Self {
        Measurement {
            timestamp: Instant::from_ticks(0),
            in_temp_c: 0.0,
            out_temp_c: 0.0,
        }
    }
}
