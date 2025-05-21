#![no_std]
#![no_main]

use defmt::{error, info};
use embassy_executor::Spawner;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::time::Rate;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;
use magic_markers::led::led_task;
use magic_markers::rfid::rfid_task;
use mfrc522::comm::blocking::i2c::I2cInterface;
use mfrc522::Mfrc522;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.3.1

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

    // Turn on LED
    let mut led: Output<'_> = Output::new(peripherals.GPIO7, Level::Low, OutputConfig::default());
    led.set_low();

    // RFID reader
    let sda = peripherals.GPIO2;
    let scl = peripherals.GPIO1;
    let mut i2c = match I2c::new(
        peripherals.I2C0,
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
    i2c = i2c.with_sda(sda).with_scl(scl);
    let itf = I2cInterface::new(i2c, 0x28);
    let mut mfrc522 = match Mfrc522::new(itf).init() {
        Ok(mfrc522) => mfrc522,
        Err(e) => match e {
            mfrc522::Error::Comm(c) => {
                error!("mfrc522 comm error: {:?}", c);
                panic!();
            }
            _ => {
                error!("other mfrc522 init error");
                panic!();
            }
        },
    };
    if let Ok(version) = mfrc522.version() {
        info!("mfrc522 version: {:?}", version);
    } else {
        error!("mfrc522 version error");
        panic!();
    }
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
