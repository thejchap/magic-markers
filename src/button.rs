use crate::state::{StateCommand, StateSignal};
use embassy_time::{Duration, Timer};
use esp_hal::gpio::Input;

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
