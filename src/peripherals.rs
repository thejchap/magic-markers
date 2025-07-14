use crate::constants::{GATEWAY_IP_ADDRESS, I2C_FREQUENCY_KHZ, RFID_I2C_ADDRESS};
use crate::mk_static;
use core::net::Ipv4Addr;
use core::str::FromStr;
use defmt::{error, info};
use embassy_net::{Ipv4Cidr, Runner, Stack, StaticConfigV4};
use esp_hal::{
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    i2c::master::{Config, I2c},
    time::Rate,
    timer::{systimer::SystemTimer, timg::TimerGroup},
    Blocking,
};
use esp_wifi::wifi::{WifiController, WifiDevice};
use mfrc522::{comm::blocking::i2c::I2cInterface, Initialized, Mfrc522};

pub struct Peripherals {
    pub led: Output<'static>,
    pub button: Input<'static>,
    pub mfrc522: Mfrc522<I2cInterface<I2c<'static, Blocking>>, Initialized>,
    pub wifi_controller: WifiController<'static>,
    pub network_runner: Runner<'static, WifiDevice<'static>>,
    pub network_stack: Stack<'static>,
}

impl Peripherals {
    pub fn new(esp_peripherals: esp_hal::peripherals::Peripherals) -> Self {
        let timer0 = SystemTimer::new(esp_peripherals.SYSTIMER);
        esp_hal_embassy::init(timer0.alarm0);
        info!("embassy initialized");

        let timer1 = TimerGroup::new(esp_peripherals.TIMG0);
        let mut rng = esp_hal::rng::Rng::new(esp_peripherals.RNG);
        let seed = rng.random().into();
        let init = mk_static!(
            esp_wifi::EspWifiController,
            esp_wifi::init(timer1.timer0, rng, esp_peripherals.RADIO_CLK).unwrap()
        );
        let (ctrl, interfaces) = esp_wifi::wifi::new(init, esp_peripherals.WIFI).unwrap();
        let device = interfaces.ap;
        let gw_ip_addr_str = GATEWAY_IP_ADDRESS;
        let gw_ip_addr = Ipv4Addr::from_str(gw_ip_addr_str).expect("failed to parse gateway ip");
        let config = embassy_net::Config::ipv4_static(StaticConfigV4 {
            address: Ipv4Cidr::new(gw_ip_addr, 24),
            gateway: Some(gw_ip_addr),
            dns_servers: Default::default(),
        });
        let (stack, runner) = embassy_net::new(
            device,
            config,
            mk_static!(
                embassy_net::StackResources<3>,
                embassy_net::StackResources::new()
            ),
            seed,
        );
        info!("wifi controller initialized");

        let led: Output<'_> =
            Output::new(esp_peripherals.GPIO7, Level::Low, OutputConfig::default());

        let button = Input::new(
            esp_peripherals.GPIO9,
            InputConfig::default().with_pull(Pull::Up),
        );

        let sda = esp_peripherals.GPIO2;
        let scl = esp_peripherals.GPIO1;

        let mut i2c = match I2c::new(
            esp_peripherals.I2C0,
            Config::default().with_frequency(Rate::from_khz(I2C_FREQUENCY_KHZ)),
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

        let itf = I2cInterface::new(i2c, RFID_I2C_ADDRESS);
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

        match mfrc522.set_antenna_gain(mfrc522::RxGain::DB48) {
            Ok(()) => info!("antenna gain set"),
            Err(_) => {
                error!("failed to set antenna gain");
                panic!();
            }
        }

        Self {
            led,
            button,
            mfrc522,
            wifi_controller: ctrl,
            network_runner: runner,
            network_stack: stack,
        }
    }
}
