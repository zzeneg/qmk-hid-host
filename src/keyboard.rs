use hidapi::{HidApi, HidDevice, HidError};
use tokio::sync::{broadcast, mpsc};

use crate::config::Device;

pub struct Keyboard {
    name: String,
    product_id: u16,
    usage: u16,
    usage_page: u16,
    reconnect_delay: u64,
}

impl Keyboard {
    pub fn new(device: Device, reconnect_delay: u64) -> Self {
        return Self {
            name: device.name.unwrap_or_else(|| "keyboard".to_string()),
            product_id: device.product_id,
            usage: device.usage,
            usage_page: device.usage_page,
            reconnect_delay,
        };
    }

    fn get_device(product_id: &u16, usage: &u16, usage_page: &u16) -> Result<HidDevice, HidError> {
        let hid_api = HidApi::new()?;
        let devices = hid_api.device_list();
        for device_info in devices {
            if device_info.product_id() == *product_id && device_info.usage() == *usage && device_info.usage_page() == *usage_page {
                let device = device_info.open_device(&hid_api)?;
                return Ok(device);
            }
        }

        return Err(HidError::HidApiErrorEmpty);
    }

    pub fn connect(&self, data_sender: broadcast::Sender<Vec<u8>>, is_connected_sender: mpsc::Sender<bool>) {
        let name = self.name.clone();
        let pid = self.product_id;
        let usage = self.usage;
        let usage_page = self.usage_page;
        let reconnect_delay = self.reconnect_delay;
        let is_connected_sender = is_connected_sender.clone();
        let mut data_receiver = data_sender.subscribe();
        std::thread::spawn(move || {
            tracing::info!("Waiting for {}...", name);
            loop {
                tracing::debug!("Trying to connect to {}...", name);
                if let Ok(device) = Self::get_device(&pid, &usage, &usage_page) {
                    let _ = &is_connected_sender.try_send(true).unwrap_or_else(|e| tracing::error!("{}", e));
                    tracing::info!("Connected to {}", name);
                    loop {
                        if let Ok(mut received) = data_receiver.blocking_recv() {
                            tracing::info!("Sending to {}: {:?}", name, received);
                            received.truncate(32);
                            received.insert(0, 0);
                            if let Err(_) = device.write(received.as_mut()) {
                                let _ = is_connected_sender.try_send(false).unwrap_or_else(|e| tracing::error!("{}", e));
                                tracing::warn!("Disconnected from {}", name);

                                break;
                            }
                        }
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(reconnect_delay));
            }
        });
    }
}
