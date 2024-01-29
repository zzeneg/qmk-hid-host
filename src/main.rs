#![cfg_attr(
    all(target_os = "windows", feature = "silent", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod config;
mod data_type;
mod keyboard;
mod providers;

use config::get_config;
use keyboard::Keyboard;
use providers::{_base::Provider, layout::LayoutProvider, media::MediaProvider, time::TimeProvider, volume::VolumeProvider};

fn main() {
    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
        .from_env_lossy();
    let tracing_subscriber = tracing_subscriber::fmt().with_env_filter(env_filter).finish();
    let _ = tracing::subscriber::set_global_default(tracing_subscriber);

    let config = get_config();

    let keyboard = Keyboard::new(config.device, config.reconnect_delay);
    let (connected_sender, data_sender) = keyboard.connect();

    let providers: Vec<Box<dyn Provider>> = vec![
        TimeProvider::new(data_sender.clone(), connected_sender.clone()),
        VolumeProvider::new(data_sender.clone(), connected_sender.clone()),
        // LayoutProvider::new(data_sender.clone(), connected_sender.clone(), config.layouts),
        // MediaProvider::new(data_sender.clone(), connected_sender.clone()),
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
