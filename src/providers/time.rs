use std::{sync::mpsc::Sender, time::Instant};

use chrono::{DateTime, Local, Timelike};

use crate::dataType::DataType;

use super::_base::Provider;

pub struct TimeProvider {
    hour: u8,
    minute: u8,
    enabled: bool,
    sender: Sender<Vec<u8>>,
}

impl TimeProvider {
    fn push(&mut self) {
        let now: DateTime<Local> = Local::now();
        let hour = now.hour() as u8;
        let minute = now.minute() as u8;
        if self.hour != hour || self.minute != minute {
            self.hour = hour;
            self.minute = minute;
            self.send();
        }
    }
}

impl Provider for TimeProvider {
    fn new(sender: Sender<Vec<u8>>) -> Self {
        Self {
            hour: 0,
            minute: 0,
            enabled: false,
            sender,
        }
    }

    fn enable(&mut self) {
        self.enabled = true;
        let mut start = Instant::now();
        self.push();
        while self.enabled {
            if start.elapsed().as_secs() > 1 {
                start = Instant::now();
                self.push();
            }
        }
    }

    fn disable(&mut self) {
        self.enabled = false;
    }

    fn send(&self) {
        let data = [DataType::Time as u8, self.hour, self.minute];
        let _ = self.sender.send(data.to_vec());
    }
}
