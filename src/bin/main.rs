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
//!
//! # tasmota commands
//!
//! ```bash
//! # change color
//! curl -X POST "http://192.168.0.102/cm?cmnd=HSBColor%20255,255,255"
//! ```

/// This module makes it easy.
use core::net::{Ipv4Addr, SocketAddrV4};
use core::{
    str::FromStr,
    sync::atomic::{AtomicU32, AtomicU8, Ordering},
};
use defmt::{debug, error, info, warn, Format};
use edge_dhcp::{
    io::{self, DEFAULT_SERVER_PORT},
    server::{Server, ServerOptions},
};
use edge_nal::UdpBind;
use edge_nal_embassy::{Udp, UdpBuffers};
use embassy_executor::Spawner;
use embassy_net::{tcp::TcpSocket, IpListenEndpoint, Ipv4Cidr, Runner, Stack, StaticConfigV4};
use embassy_time::{Duration, Instant, Timer};
use embedded_io_async::Write;
use esp_hal::{
    clock::CpuClock,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    i2c::master::{Config, I2c},
    time::Rate,
    timer::{systimer::SystemTimer, timg::TimerGroup},
    Blocking,
};
use esp_wifi::wifi::{
    AccessPointConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiState,
};
use mfrc522::{comm::blocking::i2c::I2cInterface, GenericUid, Initialized, Mfrc522, Uid};
use static_cell::StaticCell;
extern crate alloc;

/// marker colors, with map to and from their rfid uid values
/// explicitly map to/from u8 values (which is what is persisted in global state via AtomicU8)
#[repr(u8)]
#[derive(Format, PartialEq, Clone)]
enum MarkerColor {
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
    fn uid(&self) -> [u8; 7] {
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
    fn from_uid(generic_uid: &GenericUid<7>) -> Option<Self> {
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
static LAST_MARKER_COLOR_UPDATED_AT: AtomicU32 = AtomicU32::new(Instant::MIN.as_millis() as u32);
/// last detected marker color
static LAST_MARKER_COLOR: AtomicU8 = AtomicU8::new(0);

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

    // initialize wifi
    // https://github.com/esp-rs/esp-hal/blob/main/examples/src/bin/wifi_embassy_access_point.rs
    // https://tasmota.github.io/docs/Commands/#with-web-requests
    static WIFI_INIT: StaticCell<esp_wifi::EspWifiController> = StaticCell::new();
    static RESOURCES: StaticCell<embassy_net::StackResources<3>> = StaticCell::new();
    let timer1 = TimerGroup::new(peripherals.TIMG0);
    let mut rng = esp_hal::rng::Rng::new(peripherals.RNG);
    let seed = rng.random().into();
    let init = WIFI_INIT.init(esp_wifi::init(timer1.timer0, rng, peripherals.RADIO_CLK).unwrap());
    let (ctrl, interfaces) = esp_wifi::wifi::new(init, peripherals.WIFI).unwrap();
    let device = interfaces.ap;
    let gw_ip_addr_str = "192.168.2.1";
    let gw_ip_addr = Ipv4Addr::from_str(gw_ip_addr_str).expect("failed to parse gateway ip");
    let config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(gw_ip_addr, 24),
        gateway: Some(gw_ip_addr),
        dns_servers: Default::default(),
    });
    let (stack, runner) = embassy_net::new(
        device,
        config,
        RESOURCES.init(embassy_net::StackResources::new()),
        seed,
    );
    info!("wifi controller initialized");

    // initialize led
    let mut led: Output<'_> = Output::new(peripherals.GPIO7, Level::Low, OutputConfig::default());
    led.set_low();

    // initialize button
    let button = Input::new(
        peripherals.GPIO9,
        InputConfig::default().with_pull(Pull::Up),
    );

    // rfid reader
    let sda = peripherals.GPIO2;
    let scl = peripherals.GPIO1;

    // communicates via i2c protocol
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

    // start up tasks
    spawner.spawn(rfid_task(mfrc522)).unwrap();
    spawner.spawn(led_task(led)).unwrap();
    spawner.spawn(button_task(button)).unwrap();
    spawner.spawn(connection_task(ctrl)).unwrap();
    spawner.spawn(net_task(runner)).unwrap();
    spawner.spawn(run_dhcp(stack, gw_ip_addr_str)).unwrap();
    spawner.spawn(listener_task(stack)).unwrap();
}

// tasks

/// task for button press
#[embassy_executor::task]
async fn button_task(button: Input<'static>) {
    let mut was_pressed = false;
    loop {
        let is_pressed = button.is_low();
        if is_pressed && !was_pressed {
            reset_marker_color();
        }
        was_pressed = is_pressed;
        Timer::after(Duration::from_millis(100)).await;
    }
}

/// loop for managing led
#[embassy_executor::task]
async fn led_task(mut led: Output<'static>) {
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
async fn rfid_task(mut mfrc522: Mfrc522<I2cInterface<I2c<'static, Blocking>>, Initialized>) {
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

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    info!("starting net task");
    runner.run().await
}

/// task for managing wifi connection
#[embassy_executor::task]
async fn connection_task(mut controller: WifiController<'static>) {
    loop {
        if let WifiState::ApStarted = esp_wifi::wifi::wifi_state() {
            controller.wait_for_event(WifiEvent::ApStop).await;
            Timer::after(Duration::from_millis(5000)).await
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::AccessPoint(AccessPointConfiguration {
                ssid: "magic-markers".try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            info!("starting wifi...");
            controller.start_async().await.unwrap();
            info!("wifi started");
        }
    }
}

/// task for listening for http requests
#[embassy_executor::task]
async fn listener_task(stack: Stack<'static>) {
    info!("starting listener task");
    let mut rx_buffer = [0; 1536];
    let mut tx_buffer = [0; 1536];
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
    info!("connect to the `magic-markers` network and visit http://192.168.2.1:8080");
    while !stack.is_config_up() {
        Timer::after(Duration::from_millis(100)).await
    }
    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));
    loop {
        let r = socket
            .accept(IpListenEndpoint {
                addr: None,
                port: 8080,
            })
            .await;
        info!("http request");
        if r.is_err() {
            info!("connect error");
            continue;
        }
        let mut buffer = [0u8; 1024];
        let mut pos = 0;
        loop {
            match socket.read(&mut buffer).await {
                Ok(0) => {
                    info!("read eof");
                    break;
                }
                Ok(len) => {
                    let to_print =
                        unsafe { core::str::from_utf8_unchecked(&buffer[..(pos + len)]) };
                    if to_print.contains("\r\n\r\n") {
                        break;
                    }
                    pos += len;
                }
                Err(_) => {
                    info!("read error");
                    break;
                }
            };
        }
        let r = socket
            .write_all(
                b"HTTP/1.0 200 OK\r\n\r\n\
            <html>\
                <body>\
                    <h1>magic-markers!</h1>\
                </body>\
            </html>\r\n\
            ",
            )
            .await;
        if r.is_err() {
            info!("write error");
        }

        let r = socket.flush().await;
        if r.is_err() {
            info!("flush error");
        }
        Timer::after(Duration::from_millis(1000)).await;
        socket.close();
        Timer::after(Duration::from_millis(1000)).await;
        socket.abort();
    }
}

/// task for running dhcp server
#[embassy_executor::task]
async fn run_dhcp(stack: Stack<'static>, gw_ip_addr: &'static str) {
    info!("starting dhcp task");
    let ip = Ipv4Addr::from_str(gw_ip_addr).expect("dhcp task failed to parse gw ip");
    let mut buf = [0u8; 1500];
    let mut gw_buf = [Ipv4Addr::UNSPECIFIED];
    let buffers = UdpBuffers::<3, 1024, 1024, 10>::new();
    let unbound_socket = Udp::new(stack, &buffers);
    let mut bound_socket = unbound_socket
        .bind(core::net::SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::UNSPECIFIED,
            DEFAULT_SERVER_PORT,
        )))
        .await
        .unwrap();
    loop {
        _ = io::server::run(
            &mut Server::<_, 64>::new_with_et(ip),
            &ServerOptions::new(ip, Some(&mut gw_buf)),
            &mut bound_socket,
            &mut buf,
        )
        .await
        .inspect_err(|_| warn!("dhcp server error"));
        Timer::after(Duration::from_millis(500)).await;
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

/// clears the last marker color and resets the updated at timestamp
fn reset_marker_color() {
    LAST_MARKER_COLOR.store(0, Ordering::Relaxed);
    LAST_MARKER_COLOR_UPDATED_AT.store(Instant::now().as_millis() as u32, Ordering::Relaxed);
    info!("marker color reset");
}
