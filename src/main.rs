#![cfg_attr(
    all(target_os = "windows", feature = "silent", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod config;
mod data_type;
mod keyboard;
mod providers;

use std::thread;
use tokio::sync::{broadcast, mpsc};
use config::get_config;
use keyboard::Keyboard;
#[cfg(not(target_os = "macos"))]
use providers::{_base::Provider, layout::LayoutProvider, time::TimeProvider, media::MediaProvider, volume::VolumeProvider};


#[cfg(target_os = "macos")]
use {
    providers::{_base::Provider, layout::LayoutProvider, time::TimeProvider, volume::VolumeProvider},
    core_foundation_sys::runloop::CFRunLoopRun,
};

#[cfg(target_os = "macos")]
fn run(layouts: Vec<String>, data_sender: mpsc::Sender<Vec<u8>>, connected_sender: broadcast::Sender<bool>) {
    let mut is_connected = false;
    let mut connected_receiver = connected_sender.subscribe();

    thread::spawn(move || {
        let providers: Vec<Box<dyn Provider>> = vec![
            TimeProvider::new(data_sender.clone(), connected_sender.clone()),
            LayoutProvider::new(data_sender.clone(), connected_sender.clone(), layouts),
            VolumeProvider::new(data_sender.clone(), connected_sender.clone()),
        ];

        loop {
            if let Ok(connected) = connected_receiver.blocking_recv() {
                if !is_connected && connected {
                    providers.iter().for_each(|p| p.start());
                }

                is_connected = connected;
            }
        }
    });
    unsafe { CFRunLoopRun(); }
}

#[cfg(not(target_os = "macos"))]
fn run(layouts: Vec<String>, data_sender: mpsc::Sender<Vec<u8>>, connected_sender: broadcast::Sender<bool>) {
    let providers: Vec<Box<dyn Provider>> = vec![
        TimeProvider::new(data_sender.clone(), connected_sender.clone()),
        VolumeProvider::new(data_sender.clone(), connected_sender.clone()),
        LayoutProvider::new(data_sender.clone(), connected_sender.clone(), layouts),
        MediaProvider::new(data_sender.clone(), connected_sender.clone()),
    ];

    let mut is_connected = false;
    let mut connected_receiver = connected_sender.subscribe();

    loop {
        if let Ok(connected) = connected_receiver.blocking_recv() {
            if !is_connected && connected {
                providers.iter().for_each(|p| p.start());
            }

            is_connected = connected;
        }
    }
}

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
    let config = get_config(args.config);

    let keyboard = Keyboard::new(config.device, config.reconnect_delay);
    let (connected_sender, data_sender) = keyboard.connect();

    run(config.layouts, data_sender, connected_sender);
}
