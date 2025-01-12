#![cfg_attr(
    all(target_os = "windows", feature = "silent", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod config;
mod data_type;
mod keyboard;
mod providers;

use config::load_config;
use keyboard::Keyboard;
use providers::{_base::Provider, layout::LayoutProvider, relay::RelayProvider, time::TimeProvider, volume::VolumeProvider};
use tokio::sync::{broadcast, mpsc};

#[cfg(not(target_os = "macos"))]
use providers::media::MediaProvider;

#[cfg(target_os = "macos")]
use core_foundation_sys::runloop::CFRunLoopRun;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    /// Path to the configuration file
    #[arg(short, long)]
    config: Option<std::path::PathBuf>,
}

fn main() {
    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
        .from_env_lossy();
    let tracing_subscriber = tracing_subscriber::fmt().with_env_filter(env_filter).finish();
    let _ = tracing::subscriber::set_global_default(tracing_subscriber);

    let (is_connected_sender, is_connected_receiver) = mpsc::channel::<bool>(1);
    let (host_to_device_sender, _) = broadcast::channel::<Vec<u8>>(1);
    let (device_to_host_sender, _) = broadcast::channel::<Vec<u8>>(1);

    let args = Args::parse();
    let config = load_config(args.config.unwrap_or("./qmk-hid-host.json".into()));
    let reconnect_delay = config.reconnect_delay.unwrap_or(5000);
    for device in &config.devices {
        let host_to_device_sender = host_to_device_sender.clone();
        let device_to_host_sender = device_to_host_sender.clone();
        let is_connected_sender = is_connected_sender.clone();
        let keyboard = Keyboard::new(device, reconnect_delay);
        keyboard.connect(host_to_device_sender, device_to_host_sender, is_connected_sender);
    }

    run(host_to_device_sender, device_to_host_sender, is_connected_receiver);
}

#[cfg(not(target_os = "macos"))]
fn get_providers(
    host_to_device_sender: &broadcast::Sender<Vec<u8>>,
    device_to_host_sender: &broadcast::Sender<Vec<u8>>,
) -> Vec<Box<dyn Provider>> {
    return vec![
        TimeProvider::new(host_to_device_sender.clone()),
        VolumeProvider::new(host_to_device_sender.clone()),
        LayoutProvider::new(host_to_device_sender.clone()),
        MediaProvider::new(host_to_device_sender.clone()),
        RelayProvider::new(host_to_device_sender.clone(), device_to_host_sender.clone()),
    ];
}

#[cfg(target_os = "macos")]
fn get_providers(data_sender: &broadcast::Sender<Vec<u8>>) -> Vec<Box<dyn Provider>> {
    return vec![
        TimeProvider::new(data_sender.clone()),
        VolumeProvider::new(data_sender.clone()),
        LayoutProvider::new(data_sender.clone()),
    ];
}

#[cfg(not(target_os = "macos"))]
fn run(
    host_to_device_sender: broadcast::Sender<Vec<u8>>,
    device_to_host_sender: broadcast::Sender<Vec<u8>>,
    is_connected_receiver: mpsc::Receiver<bool>,
) {
    start(host_to_device_sender, device_to_host_sender, is_connected_receiver);
}

#[cfg(target_os = "macos")]
fn run(
    host_to_device_sender: broadcast::Sender<Vec<u8>>,
    device_to_host_sender: broadcast::Sender<Vec<u8>>,
    is_connected_receiver: mpsc::Receiver<bool>,
) {
    std::thread::spawn(move || {
        start(host_to_device_sender, device_to_host_sender, is_connected_receiver);
    });
    unsafe {
        CFRunLoopRun();
    }
}

fn start(
    host_to_device_sender: broadcast::Sender<Vec<u8>>,
    device_to_host_sender: broadcast::Sender<Vec<u8>>,
    mut is_connected_receiver: mpsc::Receiver<bool>,
) {
    let providers = get_providers(&host_to_device_sender, &device_to_host_sender);

    let mut connected_count = 0;
    let mut is_started = false;

    loop {
        if let Some(is_connected) = is_connected_receiver.blocking_recv() {
            connected_count += if is_connected { 1 } else { -1 };
            tracing::info!("Connected devices: {}", connected_count);

            // if new device is connected - restart providers to send all available data
            if is_started && (connected_count == 0 || is_connected) {
                tracing::info!("Stopping providers");
                is_started = false;
                providers.iter().for_each(|p| p.stop());
                std::thread::sleep(std::time::Duration::from_millis(200));
            }

            if !is_started && connected_count > 0 {
                tracing::info!("Starting providers");
                is_started = true;
                providers.iter().for_each(|p| p.start());
            }
        }
    }
}
