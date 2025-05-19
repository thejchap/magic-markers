use defmt::{error, info};
use embassy_time::{Duration, Timer};
use esp_hal::{i2c::master::I2c, Blocking};
use mfrc522::{comm::blocking::i2c::I2cInterface, Initialized, Mfrc522, Uid};

#[embassy_executor::task]
pub async fn rfid_task(mut mfrc522: Mfrc522<I2cInterface<I2c<'static, Blocking>>, Initialized>) {
    loop {
        if let Ok(atqa) = mfrc522.wupa() {
            match mfrc522.select(&atqa) {
                Ok(ref _uid @ Uid::Single(ref inner)) => {
                    info!("card uid {=[?]}", inner.as_bytes());
                }
                Ok(ref _uid @ Uid::Double(ref inner)) => {
                    info!("card double uid {=[?]}", inner.as_bytes());
                }
                Ok(_) => defmt::info!("got other uid size"),
                Err(_e) => {
                    error!("Select error");
                }
            }
        }
        Timer::after(Duration::from_millis(100)).await;
    }
}
