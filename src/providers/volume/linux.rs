use pulsectl::controllers::{DeviceControl, SinkController};
use tokio::sync::{
    broadcast::{self, Receiver},
    mpsc::{self, Sender},
};

use crate::data_type::DataType;

use super::_base::Provider;

fn get_volume() -> f32 {
    let mut handler = SinkController::create().unwrap();
    if let Ok(default) = handler.get_default_device() {
        let volume = default.volume.get().first().unwrap();
        tracing::info!("volume {:?}", volume.0);
        tracing::info!("base_volume {:?}", default.base_volume.0);
        tracing::info!("result volume {:?}", volume.0 as f32 / default.base_volume.0 as f32);
        return volume.0 as f32 / default.base_volume.0 as f32;
    }

    return 0f32;
}

fn send_data(value: &f32, push_sender: &mpsc::Sender<Vec<u8>>) {
    let volume = (value * 100.0).round() as u8;
    let data = vec![DataType::Volume as u8, volume];
    push_sender.try_send(data).unwrap_or_else(|e| tracing::error!("{}", e));
}

pub struct VolumeProvider {
    data_sender: mpsc::Sender<Vec<u8>>,
    connected_sender: broadcast::Sender<bool>,
}

impl VolumeProvider {
    pub fn new(data_sender: mpsc::Sender<Vec<u8>>, connected_sender: broadcast::Sender<bool>) -> Box<dyn Provider> {
        let provider = VolumeProvider {
            data_sender,
            connected_sender,
        };
        return Box::new(provider);
    }
}

impl Provider for VolumeProvider {
    fn start(&self) {
        tracing::info!("Volume Provider started");
        let volume = get_volume();
        send_data(&volume, &self.data_sender);
        let data_sender = self.data_sender.clone();
        let connected_sender = self.connected_sender.clone();
        std::thread::spawn(move || {
            let connected_receiver = connected_sender.subscribe();
            subscribe(data_sender, connected_receiver);
            tracing::info!("Volume Provider stopped");
        });
    }
}

fn subscribe(data_sender: Sender<Vec<u8>>, mut connected_receiver: Receiver<bool>) {
    // TODO
}
