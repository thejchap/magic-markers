use crate::constants::{
    LED_BUTTON_FLASH_TIME_MS, LED_FLASH_CYCLE_TIME_MS, LED_FLASH_OFF_TIME_MS, LED_FLASH_ON_TIME_MS,
    LED_SLOW_BLINK_OFF_TIME_MS, LED_SLOW_BLINK_ON_TIME_MS,
};
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
    let mut slow_blink_start = Instant::now().as_millis() as u32;

    loop {
        if let Some(new_state) = led_state_signal.try_take() {
            current_state = new_state;
            // Reset slow blink timing when state changes
            slow_blink_start = Instant::now().as_millis() as u32;
        }

        let now = Instant::now().as_millis() as u32;

        if !current_state.is_connected {
            // Slow blink while disconnected
            let slow_blink_time =
                (now - slow_blink_start) % (LED_SLOW_BLINK_ON_TIME_MS + LED_SLOW_BLINK_OFF_TIME_MS);
            if slow_blink_time < LED_SLOW_BLINK_ON_TIME_MS {
                led.set_high();
            } else {
                led.set_low();
            }
        } else {
            // Connected - check for button press flash, then marker flash, then off
            let last_button_at = current_state.last_button_press_at;
            let time_since_button = now - last_button_at;

            if time_since_button < LED_BUTTON_FLASH_TIME_MS {
                // Single flash on button press
                led.set_high();
            } else {
                let last_marker_at = current_state.last_marker_color_updated_at;
                let time_since_marker = now - last_marker_at;

                if current_state.last_marker_color.is_some()
                    && time_since_marker < LED_FLASH_CYCLE_TIME_MS
                {
                    // Flash pattern on marker tap
                    if time_since_marker < LED_FLASH_ON_TIME_MS
                        || (time_since_marker > (LED_FLASH_ON_TIME_MS + LED_FLASH_OFF_TIME_MS)
                            && time_since_marker < LED_FLASH_CYCLE_TIME_MS)
                    {
                        led.set_high();
                    } else {
                        led.set_low();
                    }
                } else {
                    // Off when connected but no recent activity
                    led.set_low();
                }
            }
        }

        Timer::after(Duration::from_millis(10)).await;
    }
}
