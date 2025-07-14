use crate::bulb::{BulbChannelSender, TasmotaCommand};
use crate::led::LedStateSignal;
use crate::marker_color::MarkerColor;
use crate::state::{State, StateCommand, StateSignal};
use defmt::{debug, info};
use embassy_time::{Duration, Timer};
use esp_hal::gpio::Input;
use esp_hal::{i2c::master::I2c, Blocking};
use mfrc522::{comm::blocking::i2c::I2cInterface, Initialized, Mfrc522, Uid};

#[embassy_executor::task]
pub async fn state_manager_task(
    state_signal: &'static StateSignal,
    bulb_channel_sender: BulbChannelSender,
    led_state_signal: &'static LedStateSignal,
) {
    let mut state = State::new();

    loop {
        let command = state_signal.wait().await;
        match command {
            StateCommand::SetMarkerColor(color) => {
                state.update_marker_color(color.clone());
                let (h, s, b) = color.hsb();
                bulb_channel_sender
                    .send(TasmotaCommand::HSBColor(h, s, b))
                    .await;
                led_state_signal.signal(state.clone());
            }
            StateCommand::ClearMarkerColor => {
                state.clear_marker_color();
                bulb_channel_sender.send(TasmotaCommand::White(100)).await;
                led_state_signal.signal(state.clone());
            }
        }
    }
}

#[embassy_executor::task]
pub async fn button_task(button: Input<'static>, state_signal: &'static StateSignal) {
    let mut was_pressed = false;
    loop {
        let is_pressed = button.is_low();
        if is_pressed && !was_pressed {
            state_signal.signal(StateCommand::ClearMarkerColor);
        }
        was_pressed = is_pressed;
        Timer::after(Duration::from_millis(100)).await;
    }
}

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
