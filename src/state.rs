use core::sync::atomic::AtomicU32;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Instant;
use mfrc522::GenericUid;

pub static LAST_UID_AT: AtomicU32 = AtomicU32::new(Instant::MIN.as_millis() as u32);
pub static LAST_UID: Mutex<CriticalSectionRawMutex, Option<GenericUid<7>>> = Mutex::new(None);
