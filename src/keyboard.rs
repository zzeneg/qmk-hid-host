use async_std::channel::{self, Receiver, Sender};
use hidapi::{HidApi, HidDevice, HidError};

use crate::data_type::DataType;

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
        let mut is_connected: bool = false;
        println!("Trying to connect...");

        while !is_connected {
            let device_result = Self::get_device(pid, usage, usage_page);
            if let Ok(_) = device_result {
                println!("Connected");
                is_connected = true;
                let _ = connected_sender.send_blocking(true);
            } else {
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        }
    }

    pub fn start_reader(&self, connected_sender: Sender<bool>, pull_sender: Sender<u8>) {
        let pid = self.pid.clone();
        let usage = self.usage.clone();
        let usage_page = self.usage_page.clone();
        std::thread::spawn(move || {
            let device_result = Self::get_device(pid, usage, usage_page);
            if let Ok(device) = device_result {
                println!("Reader connected to keyboard");
                let mut is_connected: bool = true;
                while is_connected {
                    let mut buf = [0u8; 32];
                    if let Ok(_) = device.read(buf.as_mut()) {
                        println!("Received from keyboard: {:?}", buf);
                        if let Err(_) = pull_sender.send_blocking(buf[0]) {
                            println!("PULL_SENDER CLOSED");
                            is_connected = false;
                        }
                    } else {
                        println!("DEVICE READING ERROR");
                        is_connected = false;
                    }
                }

                let _ = connected_sender.send_blocking(false);
                pull_sender.close();
                println!("Reader disconnected from keyboard");
            }
        });
    }

    pub fn start_writer(&self, connected_sender: Sender<bool>, push_receiver: Receiver<Vec<u8>>) {
        let pid = self.pid.clone();
        let usage = self.usage.clone();
        let usage_page = self.usage_page.clone();
        std::thread::spawn(move || {
            let device_result = Self::get_device(pid, usage, usage_page);
            if let Ok(device) = device_result {
                println!("Writer connected to keyboard");
                let mut is_connected: bool = true;
                while is_connected {
                    if let Ok(mut received) = push_receiver.recv_blocking() {
                        println!("Sending to keyboard: {:?}", received);
                        received.truncate(32);
                        received.insert(0, 0);
                        if let Err(_) = device.write(received.as_mut()) {
                            println!("DEVICE WRITING ERROR");
                            is_connected = false;
                        }
                    } else {
                        println!("PUSH_RECEIVER CLOSED");
                        is_connected = false;
                    }
                }

                let _ = connected_sender.send_blocking(false);
                push_receiver.close();
                println!("Writer disconnected from keyboard");
            }
        });
    }
}
