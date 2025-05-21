use crate::markers::MarkerColor;
use core::sync::atomic::{AtomicU32, AtomicU8};
use embassy_time::Instant;

pub static LAST_MARKER_COLOR_UPDATED_AT: AtomicU32 =
    AtomicU32::new(Instant::MIN.as_millis() as u32);
pub static LAST_MARKER_COLOR: AtomicU8 = AtomicU8::new(MarkerColor::Unknown as u8);
