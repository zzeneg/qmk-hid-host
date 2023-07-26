use std::time::Duration;

use chrono::{DateTime, Local, Timelike};
use tokio::{
    select,
    sync::{broadcast::Sender, watch},
    time::interval,
};
use tracing::{error, info};

use crate::data_type::DataType;

use super::_base::Provider;

pub struct TimeProvider {
    push_sender: Sender<Vec<u8>>,
    enabled_sender: watch::Sender<bool>,
}

impl TimeProvider {
    pub fn new(push_sender: Sender<Vec<u8>>) -> Box<dyn Provider> {
        let (enabled_sender, _) = watch::channel::<bool>(true);
        let provider = TimeProvider {
            push_sender,
            enabled_sender,
        };
        return Box::new(provider);
    }

    fn get() -> (u8, u8) {
        let now: DateTime<Local> = Local::now();
        let hour = now.hour() as u8;
        let minute = now.minute() as u8;
        return (hour, minute);
    }
}

impl Provider for TimeProvider {
    fn enable(&self) {
        info!("Time Provider enabled");
        let push_sender = self.push_sender.clone();
        let mut interval = interval(Duration::from_secs(1));
        let mut enabled_receiver = self.enabled_sender.subscribe();
        self.enabled_sender.send(true).unwrap_or_else(|e| error!("enable {}", e));
        tokio::spawn(async move {
            let (mut saved_hour, mut saved_minute) = (0u8, 0u8);
            loop {
                select! {
                    _ = enabled_receiver.wait_for(|e| *e == false) => {
                        break;
                    }
                    _ = interval.tick() => {
                        let (hour, minute) = TimeProvider::get();
                        if saved_hour != hour || saved_minute != minute {
                            (saved_hour, saved_minute) = (hour, minute);
                            let data = vec![DataType::Time as u8, saved_hour, saved_minute];
                            let _ = push_sender.send(data);
                        }
                    }
                }
            }

            info!("Time Provider stopped");
        });
    }

    fn disable(&self) {
        self.enabled_sender.send(false).unwrap_or_else(|e| error!("disable: {}", e));
    }
}
