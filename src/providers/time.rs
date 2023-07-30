use chrono::{DateTime, Local, Timelike};
use tokio::sync::{broadcast, mpsc};

use crate::data_type::DataType;

use super::_base::Provider;

fn get_time() -> (u8, u8) {
    let now: DateTime<Local> = Local::now();
    let hour = now.hour() as u8;
    let minute = now.minute() as u8;
    return (hour, minute);
}

fn send_data(value: &(u8, u8), push_sender: &mpsc::Sender<Vec<u8>>) {
    let data = vec![DataType::Time as u8, value.0, value.1];
    push_sender.try_send(data).unwrap_or_else(|e| tracing::error!("{}", e));
}

pub struct TimeProvider {
    data_sender: mpsc::Sender<Vec<u8>>,
    connected_sender: broadcast::Sender<bool>,
}

impl TimeProvider {
    pub fn new(data_sender: mpsc::Sender<Vec<u8>>, connected_sender: broadcast::Sender<bool>) -> Box<dyn Provider> {
        let provider = TimeProvider {
            data_sender,
            connected_sender,
        };
        return Box::new(provider);
    }
}

impl Provider for TimeProvider {
    fn start(&self) {
        tracing::info!("Time Provider enabled");
        let data_sender = self.data_sender.clone();
        let connected_sender = self.connected_sender.clone();
        std::thread::spawn(move || {
            let mut connected_receiver = connected_sender.subscribe();
            let mut synced_time = (0u8, 0u8);
            loop {
                if !connected_receiver.try_recv().unwrap_or(true) {
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
}
