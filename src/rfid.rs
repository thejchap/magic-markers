use core::sync::atomic::Ordering;

use defmt::{debug, info};
use embassy_time::{Duration, Instant, Timer};
use esp_hal::{i2c::master::I2c, Blocking};
use mfrc522::{comm::blocking::i2c::I2cInterface, Initialized, Mfrc522, Uid};

use crate::{
    markers::MarkerColor,
    state::{LAST_MARKER_COLOR, LAST_MARKER_COLOR_UPDATED_AT},
};

fn marker_detected(color: MarkerColor) {
    let current_color_u8 = LAST_MARKER_COLOR.load(Ordering::Relaxed);
    let current_color: MarkerColor = current_color_u8.into();
    if current_color == color {
        return;
    }
    LAST_MARKER_COLOR.store(color.clone() as u8, Ordering::Relaxed);
    LAST_MARKER_COLOR_UPDATED_AT.store(Instant::now().as_millis() as u32, Ordering::Relaxed);
    info!("color update: {:?}", color);
}

#[embassy_executor::task]
pub async fn rfid_task(mut mfrc522: Mfrc522<I2cInterface<I2c<'static, Blocking>>, Initialized>) {
    loop {
        if let Ok(atqa) = mfrc522.new_card_present() {
            match mfrc522.select(&atqa) {
                Ok(Uid::Double(inner)) => {
                    marker_detected(MarkerColor::from_uid(&inner));
                }
                Ok(_) => info!("wrong uid size"),
                Err(e) => match e {
                    mfrc522::Error::Collision => debug!("collision"),
                    mfrc522::Error::Timeout => debug!("timeout"),
                    mfrc522::Error::Comm(c) => debug!("invalid response {:?}", c),
                    mfrc522::Error::IncompleteFrame => debug!("incomplete frame"),
                    mfrc522::Error::BufferOverflow => debug!("buffer overflow"),
                    mfrc522::Error::Crc => debug!("crc error"),
                    mfrc522::Error::Protocol => debug!("protocol error"),
                    mfrc522::Error::Bcc => debug!("bcc error"),
                    mfrc522::Error::Nak => debug!("nak"),
                    mfrc522::Error::NoRoom => debug!("no room"),
                    mfrc522::Error::Overheating => debug!("overheating"),
                    mfrc522::Error::Proprietary => debug!("proprietary error"),
                    mfrc522::Error::Parity => debug!("parity error"),
                    mfrc522::Error::Wr => debug!("write error"),
                },
            }
        }
        Timer::after(Duration::from_millis(10)).await;
    }
}
