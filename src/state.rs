use crate::bulb::{BulbChannelSender, TasmotaCommand};
use crate::constants::PERIODIC_SYNC_INTERVAL_SECS;
use crate::led::LedStateSignal;
use crate::marker_color::MarkerColor;
use defmt::{info, Format};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, Timer};

#[derive(Debug, Clone)]
pub struct State {
    pub last_marker_color_updated_at: u32,
    pub last_marker_color: Option<MarkerColor>,
    pub is_connected: bool,
    pub intended_bulb_state: Option<TasmotaCommand>,
    pub current_dimmer_level: u8,
    pub last_button_press_at: u32,
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
            is_connected: false,
            intended_bulb_state: None,
            current_dimmer_level: 0,
            last_button_press_at: Instant::MIN.as_millis() as u32,
        }
    }

    pub fn update_marker_color(&mut self, color: MarkerColor) {
        self.last_marker_color = Some(color);
        self.last_marker_color_updated_at = Instant::now().as_millis() as u32;
        self.current_dimmer_level = 100;
    }

    pub fn clear_marker_color(&mut self) {
        self.last_marker_color = None;
        self.last_marker_color_updated_at = Instant::now().as_millis() as u32;
    }

    pub fn set_connected(&mut self, connected: bool) {
        self.is_connected = connected;
    }

    pub fn toggle_dimmer(&mut self) -> u8 {
        // Toggle between 0 and 100
        self.current_dimmer_level = if self.current_dimmer_level == 0 {
            100
        } else {
            0
        };
        self.current_dimmer_level
    }
}

#[derive(Format, Clone)]
pub enum StateCommand {
    SetMarkerColor(MarkerColor),
    ClearMarkerColor,
    SetConnected(bool),
    SyncState,
    ToggleDimmer,
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
                let color_changed = state.last_marker_color.as_ref() != Some(&color);
                state.update_marker_color(color.clone());
                if color_changed {
                    let (h, s, b) = color.hsb();
                    let command = TasmotaCommand::HSBColor(h, s, b);
                    state.intended_bulb_state = Some(command.clone());
                    bulb_channel_sender.send(command).await;
                    led_state_signal.signal(state.clone());
                }
            }
            StateCommand::ClearMarkerColor => {
                let had_color = state.last_marker_color.is_some();
                state.clear_marker_color();
                if had_color {
                    let command = TasmotaCommand::White(100);
                    state.intended_bulb_state = Some(command.clone());
                    bulb_channel_sender.send(command).await;
                    led_state_signal.signal(state.clone());
                }
            }
            StateCommand::SetConnected(connected) => {
                let connection_changed = state.is_connected != connected;
                let was_disconnected = !state.is_connected;
                state.set_connected(connected);
                if connection_changed {
                    led_state_signal.signal(state.clone());
                    // If we just reconnected and have an intended state, resync it
                    if connected && was_disconnected {
                        if let Some(intended_command) = &state.intended_bulb_state {
                            bulb_channel_sender.send(intended_command.clone()).await;
                        }
                    }
                }
            }
            StateCommand::SyncState => {
                // Manually triggered state sync - resend intended state if we have one
                if state.is_connected {
                    if let Some(intended_command) = &state.intended_bulb_state {
                        info!("syncing bulb state: {:?}", intended_command);
                        bulb_channel_sender.send(intended_command.clone()).await;
                    } else {
                        info!("no intended state to sync");
                    }
                } else {
                    info!("skipping sync - not connected to bulb");
                }
            }
            StateCommand::ToggleDimmer => {
                let dimmer_level = state.toggle_dimmer();
                state.last_button_press_at = Instant::now().as_millis() as u32;
                let command = TasmotaCommand::Dimmer(dimmer_level);
                state.intended_bulb_state = Some(command.clone());
                bulb_channel_sender.send(command).await;
                info!("toggled dimmer to: {}%", dimmer_level);
                led_state_signal.signal(state.clone());
            }
        }
    }
}

#[embassy_executor::task]
pub async fn periodic_sync_task(state_signal: &'static StateSignal) {
    info!(
        "starting periodic sync task with {}s interval",
        PERIODIC_SYNC_INTERVAL_SECS
    );

    loop {
        Timer::after(Duration::from_secs(PERIODIC_SYNC_INTERVAL_SECS)).await;
        info!("triggering periodic state sync");
        state_signal.signal(StateCommand::SyncState);
    }
}
