use core::sync::atomic::Ordering;

use embassy_time::{Duration, Instant, Timer};
use esp_hal::gpio::Output;

use crate::state::LAST_MARKER_COLOR_UPDATED_AT;

#[embassy_executor::task]
pub async fn led_task(mut led: Output<'static>) {
    loop {
        let now = Instant::now().as_millis() as u32;
        let last_uid_at = LAST_MARKER_COLOR_UPDATED_AT.load(Ordering::Relaxed);
        if now - last_uid_at < 100 || (now - last_uid_at > 200 && now - last_uid_at < 300) {
            led.set_high();
        } else {
            led.set_low();
        }
        Timer::after(Duration::from_millis(10)).await;
    }
}
