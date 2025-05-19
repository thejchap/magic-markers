use core::sync::atomic::Ordering;

use defmt::{info, warn};
use embassy_time::{Duration, Instant, Timer};
use esp_hal::{i2c::master::I2c, Blocking};
use mfrc522::{comm::blocking::i2c::I2cInterface, Initialized, Mfrc522, Uid};

use crate::state::{LAST_UID, LAST_UID_AT};

#[embassy_executor::task]
pub async fn rfid_task(mut mfrc522: Mfrc522<I2cInterface<I2c<'static, Blocking>>, Initialized>) {
    loop {
        if let Ok(atqa) = mfrc522.new_card_present() {
            match mfrc522.select(&atqa) {
                Ok(Uid::Double(inner)) => {
                    info!("uid: {:?}", inner.as_bytes());
                    {
                        let mut last_uid = LAST_UID.lock().await;
                        *last_uid = Some(inner);
                    }
                    LAST_UID_AT.store(Instant::now().as_millis() as u32, Ordering::Relaxed);
                }
                Ok(_) => info!("wrong uid size"),
                Err(e) => match e {
                    mfrc522::Error::Collision => warn!("collision"),
                    mfrc522::Error::Timeout => warn!("timeout"),
                    mfrc522::Error::Comm(c) => warn!("invalid response {:?}", c),
                    mfrc522::Error::IncompleteFrame => warn!("incomplete frame"),
                    mfrc522::Error::BufferOverflow => warn!("buffer overflow"),
                    mfrc522::Error::Crc => warn!("crc error"),
                    mfrc522::Error::Protocol => warn!("protocol error"),
                    mfrc522::Error::Bcc => warn!("bcc error"),
                    mfrc522::Error::Nak => warn!("nak"),
                    mfrc522::Error::NoRoom => warn!("no room"),
                    mfrc522::Error::Overheating => warn!("overheating"),
                    mfrc522::Error::Proprietary => warn!("proprietary error"),
                    mfrc522::Error::Parity => warn!("parity error"),
                    mfrc522::Error::Wr => warn!("write error"),
                },
            }
        }
        Timer::after(Duration::from_millis(10)).await;
    }
}
