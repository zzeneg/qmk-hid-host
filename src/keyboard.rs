use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;

use hidapi::{DeviceInfo, HidApi, HidDevice};
use tokio::sync::{broadcast, mpsc};

use crate::config::Device;
use crate::data_type::DataType;

pub struct Keyboard {
    name: String,
    product_id: u16,
    usage: u16,
    usage_page: u16,
    reconnect_delay: u64,
    is_connected: Arc<AtomicBool>,
}

impl Keyboard {
    pub fn new(device: &Device, reconnect_delay: u64) -> Self {
        return Self {
            name: device.name.clone().unwrap_or("keyboard".to_string()),
            product_id: device.product_id,
            usage: device.usage.unwrap_or(0x61),
            usage_page: device.usage_page.unwrap_or(0xff60),
            reconnect_delay,
            is_connected: Arc::new(AtomicBool::new(false)),
        };
    }

    fn get_device_info(hid_api: &HidApi, product_id: &u16, usage: &u16, usage_page: &u16) -> Option<DeviceInfo> {
        let devices = hid_api.device_list();
        for device_info in devices {
            if device_info.product_id() == *product_id && device_info.usage() == *usage && device_info.usage_page() == *usage_page {
                return Some(device_info.clone());
            }
        }

        None
    }

    pub fn connect(
        &self,
        host_to_device_sender: broadcast::Sender<Vec<u8>>,
        device_to_host_sender: broadcast::Sender<Vec<u8>>,
        is_connected_sender: mpsc::Sender<bool>,
    ) {
        let name = self.name.clone();
        let pid = self.product_id;
        let usage = self.usage;
        let usage_page = self.usage_page;
        let reconnect_delay = self.reconnect_delay;
        let is_connected = self.is_connected.clone();

        std::thread::spawn(move || {
            tracing::info!("Waiting for {}...", name);
            loop {
                tracing::debug!("{}: trying to connect...", name);

                let hid_api = HidApi::new().unwrap();
                if let Some(device_info) = Self::get_device_info(&hid_api, &pid, &usage, &usage_page) {
                    loop {
                        match device_info.open_device(&hid_api) {
                            Ok(device) => {
                                start_write(&name, device, &is_connected, &host_to_device_sender);
                                break;
                            }
                            Err(err) => tracing::error!("{}", err)
                        }

                        match device_info.open_device(&hid_api) {
                            Ok(device) => {
                                start_read(&name, device, &is_connected, &device_to_host_sender);
                                break
                            }
                            Err(err) => tracing::error!("{}", err)
                        }

                        std::thread::sleep(std::time::Duration::from_millis(1000));
                    }


                    tracing::info!("{}: connected", name);
                    is_connected.store(true, Relaxed);
                    let _ = is_connected_sender.try_send(true);

                    loop {
                        if !is_connected.load(Relaxed) {
                            tracing::warn!("{}: disconnected", name);
                            let _ = is_connected_sender.try_send(false);
                            break;
                        }

                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(reconnect_delay));
            }
        });
    }
}

fn start_write(name: &String, device: HidDevice, is_connected: &Arc<AtomicBool>, host_to_device_sender: &broadcast::Sender<Vec<u8>>) {
    let name = name.clone();
    let is_connected = is_connected.clone();
    let mut host_to_device_receiver = host_to_device_sender.subscribe();
    std::thread::spawn(move || loop {
        tracing::debug!("{}: waiting for data to send...", name);
        if let Ok(mut received) = host_to_device_receiver.blocking_recv() {
            tracing::info!("{}: sending {:?}", name, received);
            received.truncate(32);
            received.insert(0, 0);
            if let Err(_) = device.write(received.as_mut()) {
                is_connected.store(false, Relaxed);
                break;
            }
        }
    });
}

fn start_read(name: &String, device: HidDevice, is_connected: &Arc<AtomicBool>, device_to_host_sender: &broadcast::Sender<Vec<u8>>) {
    let name = name.clone();
    let is_connected = is_connected.clone();
    let device_to_host_sender = device_to_host_sender.clone();
    let mut data = [0u8; 32];
    std::thread::spawn(move || loop {
        tracing::debug!("{}: waiting for data from keyboard...", name);

        if let Ok(result) = device.read(data.as_mut()) {
            tracing::debug!("{}: received {:?}", name, data);
            if result > 0 && data[0] == DataType::RelayFromDevice as u8 {
                let _ = device_to_host_sender.send(data.to_vec());
            }
        } else {
            is_connected.store(false, Relaxed);
            break;
        }
    });
}
