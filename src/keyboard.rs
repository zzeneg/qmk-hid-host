use hidapi::{HidApi, HidDevice, HidError};
use tokio::sync::{broadcast, mpsc};

use crate::config::Device;

pub struct Keyboard {
    product_id: u16,
    usage: u16,
    usage_page: u16,
    reconnect_delay: u64,
}

impl Keyboard {
    pub fn new(device: Device, reconnect_delay: u64) -> Self {
        return Self {
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

    pub fn connect(&self) -> (broadcast::Sender<bool>, mpsc::Sender<Vec<u8>>) {
        let pid = self.product_id;
        let usage = self.usage;
        let usage_page = self.usage_page;
        let reconnect_delay = self.reconnect_delay;
        let (data_sender, mut data_receiver) = mpsc::channel::<Vec<u8>>(32);
        let (connected_sender, _) = broadcast::channel::<bool>(32);
        let internal_connected_sender = connected_sender.clone();
        std::thread::spawn(move || {
            tracing::info!("Waiting for keyboard...");
            loop {
                tracing::debug!("Trying to connect...");
                if let Ok(device) = Self::get_device(&pid, &usage, &usage_page) {
                    let _ = &internal_connected_sender.send(true).unwrap();
                    tracing::info!("Connected to keyboard");
                    loop {
                        let msg = data_receiver.blocking_recv();
                        if let Some(mut received) = msg {
                            tracing::info!("Sending to keyboard: {:?}", received);
                            received.truncate(32);
                            received.insert(0, 0);
                            if let Err(_) = device.write(received.as_mut()) {
                                let _ = internal_connected_sender.send(false).unwrap();
                                tracing::warn!("Disconnected from keyboard");

                                break;
                            }
                        }
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(reconnect_delay));
            }
        });

        return (connected_sender, data_sender);
    }
}
