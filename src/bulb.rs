extern crate alloc;
use crate::constants::{COMMAND_DELAY_MS, HTTP_BUFFER_SIZE, HTTP_TIMEOUT_SECS};
use crate::mk_static;
use alloc::format;
use core::fmt;
use defmt::{info, warn, Format};
use embassy_net::{
    dns::DnsSocket,
    tcp::client::{TcpClient, TcpClientState},
    Stack,
};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_time::{Duration, Timer};
use reqwless::client::HttpClient;
use reqwless::request::Method;

#[derive(Format, Clone)]
pub enum TasmotaCommand {
    HSBColor(u16, u8, u8),
    White(u16),
}

impl fmt::Display for TasmotaCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            TasmotaCommand::HSBColor(h, s, b) => {
                write!(f, "hsbcolor%20{},{},{}", h, s, b)
            }
            TasmotaCommand::White(value) => write!(f, "white%20{}", value),
        }
    }
}

pub type BulbChannel = Channel<NoopRawMutex, TasmotaCommand, 8>;
pub type BulbChannelSender = Sender<'static, NoopRawMutex, TasmotaCommand, 8>;
pub type BulbChannelReceiver = Receiver<'static, NoopRawMutex, TasmotaCommand, 8>;

#[embassy_executor::task]
pub async fn bulb_commands_task(
    stack: Stack<'static>,
    bulb_ip_addr_str: &'static str,
    bulb_channel_receiver: BulbChannelReceiver,
) {
    info!("starting web task...");
    stack.wait_link_up().await;
    stack.wait_config_up().await;
    let state = mk_static!(
        TcpClientState<1, HTTP_BUFFER_SIZE, HTTP_BUFFER_SIZE>,
        TcpClientState::<1, HTTP_BUFFER_SIZE, HTTP_BUFFER_SIZE>::new()
    );
    let tcp_client = mk_static!(
        TcpClient<'static, 1, HTTP_BUFFER_SIZE, HTTP_BUFFER_SIZE>,
        TcpClient::new(stack, state)
    );
    let dns_client = mk_static!(embassy_net::dns::DnsSocket<'static>, DnsSocket::new(stack));
    tcp_client.set_timeout(Some(Duration::from_secs(HTTP_TIMEOUT_SECS)));
    let client = mk_static!(
        HttpClient<
            'static,
            TcpClient<'static, 1, HTTP_BUFFER_SIZE, HTTP_BUFFER_SIZE>,
            embassy_net::dns::DnsSocket<'static>,
        >,
        HttpClient::new(tcp_client, dns_client)
    );
    let mut buffer = [0u8; HTTP_BUFFER_SIZE];
    loop {
        let command = bulb_channel_receiver.receive().await;
        send_bulb_command(client, &mut buffer, bulb_ip_addr_str, command).await;
    }
}

async fn send_bulb_command(
    client: &mut HttpClient<
        'static,
        TcpClient<'static, 1, HTTP_BUFFER_SIZE, HTTP_BUFFER_SIZE>,
        embassy_net::dns::DnsSocket<'static>,
    >,
    buffer: &mut [u8; HTTP_BUFFER_SIZE],
    bulb_ip_addr: &str,
    command: TasmotaCommand,
) {
    let url = format!("http://{}/cm?cmnd={}", bulb_ip_addr, command);
    let method = Method::POST;
    info!("sending request: {} {}", method, url.as_str());
    let mut req = match client.request(method, url.as_str()).await {
        Ok(req) => req,
        Err(e) => {
            warn!("request build error: {:?}", e);
            Timer::after(Duration::from_secs(2)).await;
            return;
        }
    };
    let res = match req.send(buffer).await {
        Ok(res) => res,
        Err(e) => {
            warn!("request send error: {:?}", e);
            return;
        }
    };
    info!("request sent successfully");
    match res.body().read_to_end().await {
        Ok(read) => {
            info!("response body read successfully, length: {}", read.len());
            match core::str::from_utf8(read) {
                Ok(body) => info!("response body: {:?}", body),
                Err(_) => warn!("response body is not valid UTF-8"),
            }
        }
        Err(e) => warn!("failed to read response body: {}", e),
    }
    Timer::after(Duration::from_millis(COMMAND_DELAY_MS)).await;
}
