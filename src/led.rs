use core::sync::atomic::Ordering;

use embassy_time::{Duration, Instant, Timer};
use esp_hal::gpio::Output;

use crate::state::LAST_UID_AT;

#[embassy_executor::task]
pub async fn led_task(mut led: Output<'static>) {
    loop {
        let now = Instant::now().as_millis() as u32;
        let last_uid_at = LAST_UID_AT.load(Ordering::Relaxed);
        if now - last_uid_at > 200 {
            led.set_low();
        } else {
            led.set_high();
        }
        Timer::after(Duration::from_millis(10)).await;
    }
}
