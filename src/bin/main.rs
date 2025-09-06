#![no_std]
#![no_main]

use embassy_executor::Spawner;
use esp_hal::clock::CpuClock;
use magic_markers::bulb::{bulb_commands_task, BulbChannel};
use magic_markers::button::button_task;
use magic_markers::constants::{BULB_IP_ADDRESS, HEAP_SIZE};
use magic_markers::led::{led_task, LedStateSignal};
use magic_markers::mk_static;
use magic_markers::networking::{connection_task, net_task};
use magic_markers::peripherals::Peripherals;
use magic_markers::rfid::rfid_task;
use magic_markers::state::{periodic_sync_task, state_manager_task, StateSignal};

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
    spawner.spawn(periodic_sync_task(state_signal)).unwrap();
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
            state_signal,
        ))
        .unwrap();
}
