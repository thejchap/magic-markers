use defmt::Format;
use mfrc522::GenericUid;

#[repr(u8)]
#[derive(Debug, Format, PartialEq, Clone)]
pub enum MarkerColor {
    Red = 1,
    Brown = 2,
    BlueLagoon = 3,
    Green = 4,
    Black = 5,
    SandyTan = 6,
    Gray = 7,
    Pink = 8,
    Blue = 9,
    Yellow = 10,
    Orange = 11,
    Violet = 12,
}

impl MarkerColor {
    pub fn hsb(&self) -> (u8, u8, u8) {
        match self {
            MarkerColor::Red => (0, 255, 255),
            MarkerColor::Brown => (30, 255, 255),
            MarkerColor::BlueLagoon => (180, 255, 255),
            MarkerColor::Green => (120, 255, 255),
            MarkerColor::Black => (0, 0, 0),
            MarkerColor::SandyTan => (30, 100, 200),
            MarkerColor::Gray => (0, 0, 128),
            MarkerColor::Pink => (0, 255, 255),
            MarkerColor::Blue => (240, 255, 255),
            MarkerColor::Yellow => (60, 255, 255),
            MarkerColor::Orange => (30, 255, 200),
            MarkerColor::Violet => (0, 255, 255),
        }
    }

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

impl TryFrom<u8> for MarkerColor {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(MarkerColor::Red),
            2 => Ok(MarkerColor::Brown),
            3 => Ok(MarkerColor::BlueLagoon),
            4 => Ok(MarkerColor::Green),
            5 => Ok(MarkerColor::Black),
            6 => Ok(MarkerColor::SandyTan),
            7 => Ok(MarkerColor::Gray),
            8 => Ok(MarkerColor::Pink),
            9 => Ok(MarkerColor::Blue),
            10 => Ok(MarkerColor::Yellow),
            11 => Ok(MarkerColor::Orange),
            12 => Ok(MarkerColor::Violet),
            _ => Err(()),
        }
    }
}
