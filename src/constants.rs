pub const SSID: &str = "magic-markers";
pub const PASSWORD: &str = "magic-markers";
pub const BULB_IP_ADDRESS: &str = "192.168.2.2";
pub const GATEWAY_IP_ADDRESS: &str = "192.168.2.1";
pub const RFID_I2C_ADDRESS: u8 = 0x28;

pub const HEAP_SIZE: usize = 72 * 1024;
pub const HTTP_BUFFER_SIZE: usize = 4096;
pub const I2C_FREQUENCY_KHZ: u32 = 100;
pub const HTTP_TIMEOUT_SECS: u64 = 5;
pub const COMMAND_DELAY_MS: u64 = 500;

pub const LED_FLASH_ON_TIME_MS: u32 = 100;
pub const LED_FLASH_OFF_TIME_MS: u32 = 100;
pub const LED_FLASH_CYCLE_TIME_MS: u32 = 300;
pub const LED_SLOW_BLINK_ON_TIME_MS: u32 = 500;
pub const LED_SLOW_BLINK_OFF_TIME_MS: u32 = 1500;
pub const LED_BUTTON_FLASH_TIME_MS: u32 = 150;

pub const PERIODIC_SYNC_INTERVAL_SECS: u64 = 10;
