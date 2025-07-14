use crate::bulb::{BulbChannelSender, TasmotaCommand};
use crate::led::LedStateSignal;
use crate::marker_color::MarkerColor;
use defmt::Format;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Instant;

#[derive(Debug, Clone)]
pub struct State {
    pub last_marker_color_updated_at: u32,
    pub last_marker_color: Option<MarkerColor>,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn new() -> Self {
        Self {
            last_marker_color_updated_at: Instant::MIN.as_millis() as u32,
            last_marker_color: None,
        }
    }

    pub fn update_marker_color(&mut self, color: MarkerColor) {
        self.last_marker_color = Some(color);
        self.last_marker_color_updated_at = Instant::now().as_millis() as u32;
    }

    pub fn clear_marker_color(&mut self) {
        self.last_marker_color = None;
        self.last_marker_color_updated_at = Instant::now().as_millis() as u32;
    }
}

#[derive(Format, Clone)]
pub enum StateCommand {
    SetMarkerColor(MarkerColor),
    ClearMarkerColor,
}

pub type StateSignal = Signal<NoopRawMutex, StateCommand>;

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
