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
use providers::{_base::Provider, layout::LayoutProvider, media::MediaProvider, time::TimeProvider, volume::VolumeProvider};
use std::thread;
use tokio::sync::{broadcast, mpsc};

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

    let args = Args::parse();
    let config = load_config(args.config.unwrap_or("./qmk-hid-host.json".into()));

    let (data_sender, _) = broadcast::channel::<Vec<u8>>(1);
    let (is_connected_sender, is_connected_receiver) = mpsc::channel::<bool>(1);

    for device in &config.devices {
        let data_sender = data_sender.clone();
        let is_connected_sender = is_connected_sender.clone();
        let reconnect_delay = config.reconnect_delay.unwrap_or(5000);
        thread::spawn(move || {
            let keyboard = Keyboard::new(device, reconnect_delay);
            keyboard.connect(data_sender, is_connected_sender);
        });
    }

    run(data_sender, is_connected_receiver);
}

fn get_providers(data_sender: &broadcast::Sender<Vec<u8>>) -> Vec<Box<dyn Provider>> {
    return vec![
        TimeProvider::new(data_sender.clone()),
        VolumeProvider::new(data_sender.clone()),
        LayoutProvider::new(data_sender.clone()),
        MediaProvider::new(data_sender.clone()),
    ];
}

#[cfg(not(target_os = "macos"))]
fn run(data_sender: broadcast::Sender<Vec<u8>>, is_connected_receiver: mpsc::Receiver<bool>) {
    start(data_sender, is_connected_receiver);
}

#[cfg(target_os = "macos")]
fn run(data_sender: broadcast::Sender<Vec<u8>>, is_connected_receiver: mpsc::Receiver<bool>) {
    thread::spawn(move || {
        start(data_sender, is_connected_receiver);
    });
    unsafe {
        CFRunLoopRun();
    }
}

fn start(data_sender: broadcast::Sender<Vec<u8>>, mut is_connected_receiver: mpsc::Receiver<bool>) {
    let providers = get_providers(&data_sender);

    let mut connected_count = 0;
    let mut is_started = false;

    loop {
        if let Some(is_connected) = is_connected_receiver.blocking_recv() {
            connected_count += if is_connected { 1 } else { -1 };
            tracing::info!("Connected devices: {}", connected_count);

            if connected_count > 0 && !is_started {
                tracing::info!("Starting providers");
                is_started = true;
                providers.iter().for_each(|p| p.start());
            } else if connected_count == 0 && is_started {
                tracing::info!("Stopping providers");
                is_started = false;
                providers.iter().for_each(|p| p.stop());
            }
        }
    }
}
