use chrono::{DateTime, Local, Timelike};
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::data_type::DataType;

use super::_base::Provider;

fn get_time() -> (u8, u8) {
    let now: DateTime<Local> = Local::now();
    let hour = now.hour() as u8;
    let minute = now.minute() as u8;
    return (hour, minute);
}

fn send_data(value: &(u8, u8), host_to_device_sender: &broadcast::Sender<Vec<u8>>) {
    let data = vec![DataType::Time as u8, value.0, value.1];
    if let Err(e) = host_to_device_sender.send(data) {
        tracing::error!("Time Provider failed to send data: {:?}", e);
    }
}

pub struct TimeProvider {
    host_to_device_sender: broadcast::Sender<Vec<u8>>,
    is_started: Arc<AtomicBool>,
}

impl TimeProvider {
    pub fn new(host_to_device_sender: broadcast::Sender<Vec<u8>>) -> Box<dyn Provider> {
        let provider = TimeProvider {
            host_to_device_sender,
            is_started: Arc::new(AtomicBool::new(false)),
        };
        return Box::new(provider);
    }
}

impl Provider for TimeProvider {
    fn start(&self) {
        tracing::info!("Time Provider started");
        self.is_started.store(true, Relaxed);
        let host_to_device_sender = self.host_to_device_sender.clone();
        let is_started = self.is_started.clone();
        std::thread::spawn(move || {
            let mut synced_time = (0u8, 0u8);
            loop {
                if !is_started.load(Relaxed) {
                    break;
                }

                let time = get_time();
                if synced_time != time {
                    synced_time = time;
                    send_data(&synced_time, &host_to_device_sender);
                }

                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            tracing::info!("Time Provider stopped");
        });
    }

    fn stop(&self) {
        self.is_started.store(false, Relaxed);
    }
}
