use defmt::Format;
use mfrc522::GenericUid;

#[repr(u8)]
#[derive(Format, PartialEq, Clone)]
pub enum MarkerColor {
    Unknown = 0,
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
            MarkerColor::Unknown => [0, 0, 0, 0, 0, 0, 0],
        }
    }
    pub fn from_uid(generic_uid: &GenericUid<7>) -> Self {
        let uid = generic_uid.as_bytes();
        if uid == MarkerColor::Red.uid() {
            MarkerColor::Red
        } else if uid == MarkerColor::Brown.uid() {
            MarkerColor::Brown
        } else if uid == MarkerColor::BlueLagoon.uid() {
            MarkerColor::BlueLagoon
        } else if uid == MarkerColor::Green.uid() {
            MarkerColor::Green
        } else if uid == MarkerColor::Black.uid() {
            MarkerColor::Black
        } else if uid == MarkerColor::SandyTan.uid() {
            MarkerColor::SandyTan
        } else if uid == MarkerColor::Gray.uid() {
            MarkerColor::Gray
        } else if uid == MarkerColor::Pink.uid() {
            MarkerColor::Pink
        } else if uid == MarkerColor::Blue.uid() {
            MarkerColor::Blue
        } else if uid == MarkerColor::Yellow.uid() {
            MarkerColor::Yellow
        } else if uid == MarkerColor::Orange.uid() {
            MarkerColor::Orange
        } else if uid == MarkerColor::Violet.uid() {
            MarkerColor::Violet
        } else {
            MarkerColor::Unknown
        }
    }
}
impl From<u8> for MarkerColor {
    fn from(value: u8) -> Self {
        match value {
            1 => MarkerColor::Red,
            2 => MarkerColor::Brown,
            3 => MarkerColor::BlueLagoon,
            4 => MarkerColor::Green,
            5 => MarkerColor::Black,
            6 => MarkerColor::SandyTan,
            7 => MarkerColor::Gray,
            8 => MarkerColor::Pink,
            9 => MarkerColor::Blue,
            10 => MarkerColor::Yellow,
            11 => MarkerColor::Orange,
            12 => MarkerColor::Violet,
            _ => MarkerColor::Unknown,
        }
    }
}
