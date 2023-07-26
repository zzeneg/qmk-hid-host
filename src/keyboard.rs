use hidapi::{HidApi, HidDevice, HidError};
use tokio::{select, sync::broadcast::Sender};
use tracing::{error, info, warn};

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

    pub fn connect(&self, connected_sender: Sender<bool>) {
        let pid = self.pid.clone();
        let usage = self.usage.clone();
        let usage_page = self.usage_page.clone();
        info!("Trying to connect...");

        loop {
            let device_result = Self::get_device(pid, usage, usage_page);
            if let Ok(_) = device_result {
                info!("Connected");
                let _ = connected_sender.send(true);
                break;
            } else {
                let _ = tokio::time::sleep(std::time::Duration::from_secs(5));
            }
        }
    }

    pub fn start_reader(&self, connected_sender: Sender<bool>, pull_sender: Sender<u8>) {
        let pid = self.pid.clone();
        let usage = self.usage.clone();
        let usage_page = self.usage_page.clone();
        tokio::task::spawn_blocking(move || {
            let device_result = Self::get_device(pid, usage, usage_page);
            if let Ok(device) = device_result {
                info!("Reader connected to keyboard");
                let mut buf = [0u8; 32];
                loop {
                    if let Ok(_) = device.read(buf.as_mut()) {
                        info!("Received from keyboard: {:?}", buf);
                        if let Err(_) = pull_sender.send(buf[0]) {
                            warn!("PULL_SENDER CLOSED");
                        }
                    } else {
                        error!("DEVICE READING ERROR");
                        break;
                    }
                }

                let _ = connected_sender.send(false);
                warn!("Reader disconnected from keyboard");
            }
        });
    }

    pub fn start_writer(&self, connected_sender: Sender<bool>, push_sender: Sender<Vec<u8>>) {
        let pid = self.pid.clone();
        let usage = self.usage.clone();
        let usage_page = self.usage_page.clone();
        let mut connected_receiver = connected_sender.subscribe();
        tokio::spawn(async move {
            let device_result = Self::get_device(pid, usage, usage_page);
            if let Ok(device) = device_result {
                info!("Writer connected to keyboard");
                let mut push_receiver = push_sender.subscribe();
                loop {
                    select! {
                        msg = connected_receiver.recv() => {
                            if let Ok(connected) = msg {
                                if !connected {
                                    break;
                                }
                            } else {
                                error!("start_writer connected_receiver: {}", msg.unwrap_err());
                            }
                        }
                        msg = push_receiver.recv() => {
                            if let Ok(mut received) = msg {
                                info!("Sending to keyboard: {:?}", received);
                                received.truncate(32);
                                received.insert(0, 0);
                                if let Err(_) = device.write(received.as_mut()) {
                                    error!("DEVICE WRITING ERROR");
                                    let _ = connected_sender.send(false);
                                    break;
                                }
                            } else {
                                error!("start_writer push_receiver: {}", msg.unwrap_err());
                            }
                        }
                    }
                }

                info!("Writer disconnected from keyboard");
            }
        });
    }
}
