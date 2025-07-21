use defmt::Format;
use mfrc522::GenericUid;

#[derive(Debug, Format, PartialEq, Clone)]
pub enum MarkerColor {
    Red,
    Brown,
    BlueLagoon,
    Green,
    Black,
    SandyTan,
    Gray,
    Pink,
    Blue,
    Yellow,
    Orange,
    Violet,
}

impl MarkerColor {
    /// returns the hue, saturation, and brightness values for the marker color
    ///
    /// h - hue. 0-360
    /// s - saturation. 0-100
    /// b - brightness. 0-100
    pub fn hsb(&self) -> (u16, u8, u8) {
        match self {
            MarkerColor::Red => (0, 100, 100),
            MarkerColor::Brown => (30, 100, 30),
            MarkerColor::BlueLagoon => (180, 100, 100),
            MarkerColor::Green => (120, 100, 100),
            MarkerColor::Black => (0, 0, 1),
            MarkerColor::SandyTan => (30, 100, 100),
            MarkerColor::Gray => (0, 0, 50),
            MarkerColor::Pink => (340, 100, 100),
            MarkerColor::Blue => (240, 100, 100),
            MarkerColor::Yellow => (60, 100, 100),
            MarkerColor::Orange => (30, 100, 100),
            MarkerColor::Violet => (0, 0, 70),
        }
    }

    /// maps the marker color to its rfid uid value
    pub fn uid(&self) -> [u8; 7] {
        match self {
            MarkerColor::Red => [4, 61, 60, 18, 54, 30, 145],
            MarkerColor::Brown => [4, 61, 59, 18, 54, 30, 145],
            MarkerColor::BlueLagoon => [4, 61, 58, 18, 54, 30, 145],
            MarkerColor::Green => [4, 61, 57, 18, 54, 30, 145],
            MarkerColor::Black => [4, 61, 56, 18, 54, 30, 145],
            MarkerColor::SandyTan => [4, 61, 55, 18, 54, 30, 145],
            MarkerColor::Gray => [4, 61, 54, 18, 54, 30, 145],
            MarkerColor::Pink => [4, 61, 53, 18, 54, 30, 145],
            MarkerColor::Blue => [4, 61, 52, 18, 54, 30, 145],
            MarkerColor::Yellow => [4, 61, 51, 18, 54, 30, 145],
            MarkerColor::Orange => [4, 61, 50, 18, 54, 30, 145],
            MarkerColor::Violet => [4, 61, 49, 18, 54, 30, 145],
        }
    }

    /// maps a rfid uid value to a marker color
    pub fn from_uid(generic_uid: &GenericUid<7>) -> Option<Self> {
        let uid = generic_uid.as_bytes();
        if uid == MarkerColor::Red.uid() {
            Some(MarkerColor::Red)
        } else if uid == MarkerColor::Brown.uid() {
            Some(MarkerColor::Brown)
        } else if uid == MarkerColor::BlueLagoon.uid() {
            Some(MarkerColor::BlueLagoon)
        } else if uid == MarkerColor::Green.uid() {
            Some(MarkerColor::Green)
        } else if uid == MarkerColor::Black.uid() {
            Some(MarkerColor::Black)
        } else if uid == MarkerColor::SandyTan.uid() {
            Some(MarkerColor::SandyTan)
        } else if uid == MarkerColor::Gray.uid() {
            Some(MarkerColor::Gray)
        } else if uid == MarkerColor::Pink.uid() {
            Some(MarkerColor::Pink)
        } else if uid == MarkerColor::Blue.uid() {
            Some(MarkerColor::Blue)
        } else if uid == MarkerColor::Yellow.uid() {
            Some(MarkerColor::Yellow)
        } else if uid == MarkerColor::Orange.uid() {
            Some(MarkerColor::Orange)
        } else if uid == MarkerColor::Violet.uid() {
            Some(MarkerColor::Violet)
        } else {
            None
        }
    }
}
