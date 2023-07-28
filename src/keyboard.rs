use hidapi::{HidApi, HidDevice, HidError};
use tokio::sync::{broadcast, mpsc};

pub struct Keyboard {
    pid: u16,
    usage: u16,
    usage_page: u16,
}

impl Keyboard {
    pub fn new(pid: u16, usage: u16, usage_page: u16) -> Self {
        return Self { pid, usage, usage_page };
    }

    fn get_device(pid: u16, usage: u16, usage_page: u16) -> Result<HidDevice, HidError> {
        let hid_api = HidApi::new()?;
        let devices = hid_api.device_list();
        for device_info in devices {
            if device_info.product_id() == pid && device_info.usage() == usage && device_info.usage_page() == usage_page {
                let device = device_info.open_device(&hid_api)?;
                return Ok(device);
            }
        }

        return Err(HidError::HidApiErrorEmpty);
    }

    pub fn connect(&self) -> (broadcast::Sender<bool>, mpsc::Sender<Vec<u8>>) {
        let pid = self.pid.clone();
        let usage = self.usage.clone();
        let usage_page = self.usage_page.clone();
        let (data_sender, mut data_receiver) = mpsc::channel::<Vec<u8>>(1);
        let (connected_sender, _) = broadcast::channel::<bool>(32);
        let internal_connected_sender = connected_sender.clone();
        std::thread::spawn(move || {
            tracing::info!("Trying to connect...");
            loop {
                if let Ok(device) = Self::get_device(pid, usage, usage_page) {
                    let _ = internal_connected_sender.send(true).unwrap();
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

                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        });

        return (connected_sender, data_sender);
    }
}
