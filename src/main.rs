mod data_type;
mod keyboard;
mod providers;

use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::{
    keyboard::Keyboard,
    providers::{_base::Provider, time::TimeProvider, volume::VolumeProvider},
};

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt().finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    info!("Started");

    let (connected_sender, mut connected_receiver) = broadcast::channel::<bool>(1);
    let (pull_sender, _) = broadcast::channel::<u8>(1);
    let (push_sender, _) = broadcast::channel::<Vec<u8>>(1);

    let providers: Vec<Box<dyn Provider>> = vec![
        TimeProvider::new(push_sender.clone()),
        VolumeProvider::new(push_sender.clone(), pull_sender.clone()),
    ];

    let keyboard = Keyboard::new(0x0844, 0x61, 0xff60);
    keyboard.connect(connected_sender.clone());

    let mut current_status = false;

    loop {
        if let Ok(connected) = connected_receiver.recv().await {
            if connected != current_status {
                current_status = connected;
                if current_status {
                    // keyboard.start_reader(connected_sender.clone(), pull_sender.clone());
                    keyboard.start_writer(connected_sender.clone(), push_sender.clone());
                    while push_sender.receiver_count() == 0 {
                        let _ = tokio::time::sleep(std::time::Duration::from_millis(50));
                    }
                    providers.iter().for_each(|p| p.enable());
                } else {
                    warn!("Keyboard disconnected");
                    providers.iter().for_each(|p| p.disable());
                    keyboard.connect(connected_sender.clone());
                }
            }
        }
    }
}
