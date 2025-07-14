#![no_std]
#![no_main]

//! # control an led smart bulb using a set of crayola markers!
//! inspired by [this reel](https://www.instagram.com/reel/DIE2O59Svcz/?igsh=MXNnbmJsZWRmcHhlNA%3D%3D)
//!
//! # how it works
//! the markers have a unique rfid tag, which is read by an rfid reader
//! color changes post a command directly to the led smart bulb via an http request to the tasmota command endpoint
//!
//! # resources
//! - [tasmota light docs](https://tasmota.github.io/docs/Lights/#3-channels-rgb-lights)
//! - [tasmota commands](https://tasmota.github.io/docs/Lights/#3-channels-rgb-lights)
//! - [tasmota firmware](https://github.com/arendst/Tasmota-firmware/tree/firmware/release-firmware/tasmota)
//! - [markers](https://www.amazon.com/dp/B003HGGPLW)
//! - [rfid reader](https://shop.m5stack.com/products/rfid-unit-2-ws1850s?srsltid=AfmBOop6K8L69siyTW5ufYZakI-9S1a9My58NNKoWxzAvqqJq6W6jRW3)
//! - [nanoc6 examples](https://www.amazon.com/dp/B0B3XQ5Z6F)
//! - [nanoc6 docs](https://docs.m5stack.com/en/core/M5NanoC6)
//! - [esp hal wifi embassy access point example](https://github.com/esp-rs/esp-hal/blob/main/examples/src/bin/wifi_embassy_access_point.rs)
//!
//! # tasmota
//!
//! ## template
//!
//! ```json
//! {"NAME":"Kauf Bulb", "GPIO":[0,0,0,0,416,419,0,0,417,420,418,0,0,0], "FLAG":0, "BASE":18, "CMND":"SO105 1|RGBWWTable 204,204,122,153,153"}
//! ```
//!
//! ## commands
//!
//! configures the bulb and restarts to connect to the esp32 access point
//!
//! ```bash
//! backlog template {"NAME":"Kauf Bulb", "GPIO":[0,0,0,0,416,419,0,0,417,420,418,0,0,0], "FLAG":0, "BASE":18, "CMND":"SO105 1|RGBWWTable 204,204,122,153,153"}; module 0; fade 1; devicename magic-markers-bulb; friendlyname1 magic-markers-bulb; ipaddress1 192.168.2.2; ipaddress2 192.168.2.1; ipaddress3 255.255.255.0; ssid1 magic-markers; password1 magic-markers; wificonfig 0
//! ```
//!
//! # networking
//! the rfid reader is connected to an esp32 dev kit, which starts a wifi access point
//! it expects the bulb to be connected to its network with a static ip:
//! - ip: 192.168.2.2
//! - gateway: 192.168.2.1
//! - subnet: 255.255.255.0

use embassy_executor::Spawner;
use esp_hal::clock::CpuClock;

use magic_markers::bulb::{bulb_commands_task, BulbChannel};
use magic_markers::constants::{BULB_IP_ADDRESS, HEAP_SIZE};
use magic_markers::led::{led_task, LedStateSignal};
use magic_markers::mk_static;
use magic_markers::networking::{connection_task, net_task};
use magic_markers::peripherals::Peripherals;
use magic_markers::state::StateSignal;
use magic_markers::tasks::{button_task, rfid_task, state_manager_task};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    rtt_target::rtt_init_defmt!();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let esp_peripherals = esp_hal::init(config);
    esp_alloc::heap_allocator!(size: HEAP_SIZE);

    let peripherals = Peripherals::new(esp_peripherals);
    let bulb_channel = mk_static!(BulbChannel, BulbChannel::new());
    let state_signal = mk_static!(StateSignal, StateSignal::new());
    let led_state_signal = mk_static!(LedStateSignal, LedStateSignal::new());

    spawner
        .spawn(state_manager_task(
            state_signal,
            bulb_channel.sender(),
            led_state_signal,
        ))
        .unwrap();
    spawner
        .spawn(rfid_task(peripherals.mfrc522, state_signal))
        .unwrap();
    spawner
        .spawn(led_task(peripherals.led, led_state_signal))
        .unwrap();
    spawner
        .spawn(button_task(peripherals.button, state_signal))
        .unwrap();
    spawner
        .spawn(connection_task(peripherals.wifi_controller))
        .unwrap();
    spawner.spawn(net_task(peripherals.network_runner)).unwrap();
    spawner
        .spawn(bulb_commands_task(
            peripherals.network_stack,
            BULB_IP_ADDRESS,
            bulb_channel.receiver(),
        ))
        .unwrap();
}
