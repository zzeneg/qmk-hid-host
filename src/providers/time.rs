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

fn send_data(value: &(u8, u8), push_sender: &broadcast::Sender<Vec<u8>>) {
    let data = vec![DataType::Time as u8, value.0, value.1];
    push_sender.send(data).unwrap();
}

pub struct TimeProvider {
    data_sender: broadcast::Sender<Vec<u8>>,
    is_started: Arc<AtomicBool>,
}

impl TimeProvider {
    pub fn new(data_sender: broadcast::Sender<Vec<u8>>) -> Box<dyn Provider> {
        let provider = TimeProvider {
            data_sender,
            is_started: Arc::new(AtomicBool::new(false)),
        };
        return Box::new(provider);
    }
}

impl Provider for TimeProvider {
    fn start(&self) {
        tracing::info!("Time Provider started");
        self.is_started.store(true, Relaxed);
        let data_sender = self.data_sender.clone();
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
                    send_data(&synced_time, &data_sender);
                }

                std::thread::sleep(std::time::Duration::from_secs(1));
            }

            tracing::info!("Time Provider stopped");
        });
    }

    fn stop(&self) {
        self.is_started.store(false, Relaxed);
    }
}
