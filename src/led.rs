use crate::constants::{LED_FLASH_CYCLE_TIME_MS, LED_FLASH_OFF_TIME_MS, LED_FLASH_ON_TIME_MS};
use crate::state::State;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, Timer};
use esp_hal::gpio::Output;

pub type LedStateSignal = Signal<NoopRawMutex, State>;

#[embassy_executor::task]
pub async fn led_task(mut led: Output<'static>, led_state_signal: &'static LedStateSignal) {
    led.set_low();
    let mut current_state = State::new();

    loop {
        if let Some(new_state) = led_state_signal.try_take() {
            current_state = new_state;
        }

        let now = Instant::now().as_millis() as u32;
        let last_uid_at = current_state.last_marker_color_updated_at;
        if now - last_uid_at < LED_FLASH_ON_TIME_MS
            || (now - last_uid_at > (LED_FLASH_ON_TIME_MS + LED_FLASH_OFF_TIME_MS)
                && now - last_uid_at < LED_FLASH_CYCLE_TIME_MS)
        {
            led.set_high();
        } else {
            led.set_low();
        }
        Timer::after(Duration::from_millis(10)).await;
    }
}
