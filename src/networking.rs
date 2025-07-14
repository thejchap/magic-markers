use crate::constants::{PASSWORD, SSID};
use defmt::info;
use embassy_net::Runner;
use embassy_time::{Duration, Timer};
use esp_wifi::wifi::{
    AccessPointConfiguration, AuthMethod, Configuration, WifiController, WifiDevice, WifiEvent,
    WifiState,
};

#[embassy_executor::task]
pub async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    info!("starting net task...");
    runner.run().await
}

#[embassy_executor::task]
pub async fn connection_task(mut controller: WifiController<'static>) {
    loop {
        if let WifiState::ApStarted = esp_wifi::wifi::wifi_state() {
            controller.wait_for_event(WifiEvent::ApStop).await;
            Timer::after(Duration::from_millis(5000)).await
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::AccessPoint(AccessPointConfiguration {
                ssid: SSID.try_into().unwrap(),
                password: PASSWORD.try_into().unwrap(),
                auth_method: AuthMethod::WPA2Personal,
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            info!("starting wifi...");
            controller.start_async().await.unwrap();
            info!("wifi started");
        }
    }
}
