use crate::marker_color::MarkerColor;
use crate::state::{StateCommand, StateSignal};
use defmt::{debug, info};
use embassy_time::{Duration, Timer};
use esp_hal::{i2c::master::I2c, Blocking};
use mfrc522::{comm::blocking::i2c::I2cInterface, Initialized, Mfrc522, Uid};

#[embassy_executor::task]
pub async fn rfid_task(
    mut mfrc522: Mfrc522<I2cInterface<I2c<'static, Blocking>>, Initialized>,
    state_signal: &'static StateSignal,
) {
    loop {
        if let Ok(atqa) = mfrc522.new_card_present() {
            match mfrc522.select(&atqa) {
                Ok(Uid::Double(inner)) => {
                    if let Some(marker_color) = MarkerColor::from_uid(&inner) {
                        info!("detected color: {}", marker_color);
                        state_signal.signal(StateCommand::SetMarkerColor(marker_color));
                    } else {
                        info!("unknown marker uid: {}", inner.as_bytes());
                    }
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
