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
