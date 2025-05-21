#![no_std]
#![no_main]

use core::sync::atomic::{AtomicU32, AtomicU8, Ordering};
use defmt::{debug, error, info, Format};
use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};
use esp_hal::{
    clock::CpuClock,
    gpio::{Level, Output, OutputConfig},
    i2c::master::{Config, I2c},
    time::Rate,
    timer::{systimer::SystemTimer, timg::TimerGroup},
    Blocking,
};
use mfrc522::{comm::blocking::i2c::I2cInterface, GenericUid, Initialized, Mfrc522, Uid};
extern crate alloc;

/// marker colors, with map to and from their rfid uid values
/// explicitly map to/from u8 values (which is what is persisted in global state via AtomicU8)
#[repr(u8)]
#[derive(Format, PartialEq, Clone)]
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

// global state

/// last time marker color was updated - used for led flash
pub static LAST_MARKER_COLOR_UPDATED_AT: AtomicU32 =
    AtomicU32::new(Instant::MIN.as_millis() as u32);
/// last detected marker color
pub static LAST_MARKER_COLOR: AtomicU8 = AtomicU8::new(0);

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // real time transfer - for debug/logging
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    info!("embassy initialized");

    let timer1 = TimerGroup::new(peripherals.TIMG0);
    let _init = esp_wifi::init(
        timer1.timer0,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();

    // initialize led
    let mut led: Output<'_> = Output::new(peripherals.GPIO7, Level::Low, OutputConfig::default());
    led.set_low();

    // rfid reader
    let sda = peripherals.GPIO2;
    let scl = peripherals.GPIO1;

    // communicates via i2c protocol
    // https://shop.m5stack.com/products/rfid-unit-2-ws1850s?srsltid=AfmBOop6K8L69siyTW5ufYZakI-9S1a9My58NNKoWxzAvqqJq6W6jRW3
    let mut i2c = match I2c::new(
        peripherals.I2C0,
        // 100khz is i2c standard
        Config::default().with_frequency(Rate::from_khz(100)),
    ) {
        Ok(i2c) => {
            info!("i2c initialized");
            i2c
        }
        Err(e) => {
            error!("i2c init error: {:?}", e);
            panic!();
        }
    };

    // set serial data and clock pins for i2c
    i2c = i2c.with_sda(sda).with_scl(scl);

    // i2c interface and rfid driver - 0x28 address found on rfid2 product page
    let itf = I2cInterface::new(i2c, 0x28);
    let mut mfrc522 = Mfrc522::new(itf).init().unwrap_or_else(|e| match e {
        mfrc522::Error::Comm(c) => {
            error!("mfrc522 comm error: {:?}", c);
            panic!();
        }
        _ => {
            error!("other mfrc522 init error");
            panic!();
        }
    });
    if let Ok(version) = mfrc522.version() {
        info!("mfrc522 version: {:?}", version);
    } else {
        error!("mfrc522 version error");
        panic!();
    }

    // max out antenna gain
    match mfrc522.set_antenna_gain(mfrc522::RxGain::DB48) {
        Ok(()) => info!("antenna gain set"),
        Err(_) => {
            error!("failed to set antenna gain");
            panic!();
        }
    }

    spawner.spawn(rfid_task(mfrc522)).unwrap();
    spawner.spawn(led_task(led)).unwrap();
}

// tasks

/// loop for managing led
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

/// loop for rfid reader
#[embassy_executor::task]
pub async fn rfid_task(mut mfrc522: Mfrc522<I2cInterface<I2c<'static, Blocking>>, Initialized>) {
    loop {
        if let Ok(atqa) = mfrc522.new_card_present() {
            match mfrc522.select(&atqa) {
                Ok(Uid::Double(inner)) => {
                    if let Some(marker_color) = MarkerColor::from_uid(&inner) {
                        marker_detected(marker_color);
                    }
                }
                Ok(_) => info!("wrong uid size"),
                Err(e) => match e {
                    mfrc522::Error::Collision => debug!("collision"),
                    mfrc522::Error::Timeout => debug!("timeout"),
                    mfrc522::Error::Comm(c) => debug!("invalid response {:?}", c),
                    mfrc522::Error::IncompleteFrame => debug!("incomplete frame"),
                    mfrc522::Error::BufferOverflow => debug!("buffer overflow"),
                    mfrc522::Error::Crc => debug!("crc error"),
                    mfrc522::Error::Protocol => debug!("protocol error"),
                    mfrc522::Error::Bcc => debug!("bcc error"),
                    mfrc522::Error::Nak => debug!("nak"),
                    mfrc522::Error::NoRoom => debug!("no room"),
                    mfrc522::Error::Overheating => debug!("overheating"),
                    mfrc522::Error::Proprietary => debug!("proprietary error"),
                    mfrc522::Error::Parity => debug!("parity error"),
                    mfrc522::Error::Wr => debug!("write error"),
                },
            }
        }
        Timer::after(Duration::from_millis(10)).await;
    }
}

// helpers

/// when we detect a marker from our set of markers, update global state
/// if it's the same marker as the current one, do nothing
/// otherwise set updated at and set the current marker to the new one
fn marker_detected(color: MarkerColor) {
    let current_color_u8 = LAST_MARKER_COLOR.load(Ordering::Relaxed);
    let Ok(current_color): Result<MarkerColor, _> = current_color_u8.try_into() else {
        return;
    };
    if current_color == color {
        return;
    }
    LAST_MARKER_COLOR.store(color.clone() as u8, Ordering::Relaxed);
    LAST_MARKER_COLOR_UPDATED_AT.store(Instant::now().as_millis() as u32, Ordering::Relaxed);
    info!("color update: {:?}", color);
}
