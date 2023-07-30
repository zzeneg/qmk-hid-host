mod data_type;
mod keyboard;
mod providers;

use providers::{layout::LayoutProvider, media::MediaProvider};

use crate::{
    keyboard::Keyboard,
    providers::{_base::Provider, time::TimeProvider, volume::VolumeProvider},
};

fn main() {
    let tracing_subscriber = tracing_subscriber::fmt().finish();
    let _ = tracing::subscriber::set_global_default(tracing_subscriber);

    let keyboard = Keyboard::new(0x0844, 0x61, 0xff60);
    let (connected_sender, data_sender) = keyboard.connect();

    let providers: Vec<Box<dyn Provider>> = vec![
        TimeProvider::new(data_sender.clone(), connected_sender.clone()),
        VolumeProvider::new(data_sender.clone(), connected_sender.clone()),
        LayoutProvider::new(data_sender.clone(), connected_sender.clone()),
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
