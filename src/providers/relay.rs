use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::data_type::DataType;

use super::_base::Provider;

pub struct RelayProvider {
    host_to_device_sender: broadcast::Sender<Vec<u8>>,
    device_to_host_sender: broadcast::Sender<Vec<u8>>,
    is_started: Arc<AtomicBool>,
}

impl RelayProvider {
    pub fn new(host_to_device_sender: broadcast::Sender<Vec<u8>>, device_to_host_sender: broadcast::Sender<Vec<u8>>) -> Box<dyn Provider> {
        let provider = RelayProvider {
            host_to_device_sender,
            device_to_host_sender,
            is_started: Arc::new(AtomicBool::new(false)),
        };
        return Box::new(provider);
    }
}

impl Provider for RelayProvider {
    fn start(&self) {
        tracing::info!("Relay Provider started");
        self.is_started.store(true, Relaxed);
        let host_to_device_sender = self.host_to_device_sender.clone();
        let is_started = self.is_started.clone();
        let mut relay_subscriber = self.device_to_host_sender.subscribe();
        std::thread::spawn(move || {
            loop {
                if !is_started.load(Relaxed) {
                    break;
                }

                tracing::debug!("Relay Provider: waiting for data...");
                if let Ok(mut data) = relay_subscriber.blocking_recv() {
                    data[0] = DataType::RelayToDevice as u8;
                    if let Err(e) = host_to_device_sender.send(data) {
                        tracing::error!("Relay Provider failed to send data: {:?}", e);
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            tracing::info!("Relay Provider stopped");
        });
    }

    fn stop(&self) {
        self.is_started.store(false, Relaxed);
    }
}
