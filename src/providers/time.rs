use async_std::channel::{Receiver, Sender};
use chrono::{DateTime, Local, Timelike};

use crate::data_type::DataType;

use super::_base::Provider;

pub struct TimeProvider {
    push_sender: Sender<Vec<u8>>,
    pull_receiver: Receiver<u8>,
}

impl TimeProvider {
    pub fn new(push_sender: Sender<Vec<u8>>, pull_receiver: Receiver<u8>) -> Box<dyn Provider> {
        let provider = TimeProvider {
            push_sender,
            pull_receiver,
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
        println!("Time Provider enabled");
        let push_sender = self.push_sender.clone();
        std::thread::spawn(move || {
            let mut is_enabled = true;
            let (mut saved_hour, mut saved_minute) = (0u8, 0u8);
            while is_enabled {
                let (hour, minute) = TimeProvider::get();
                if saved_hour != hour || saved_minute != minute {
                    (saved_hour, saved_minute) = (hour, minute);
                    let data = vec![DataType::Time as u8, saved_hour, saved_minute];
                    if let Err(_) = push_sender.send_blocking(data) {
                        println!("PUSH_SENDER CLOSED");
                        is_enabled = false;
                    }
                }

                std::thread::sleep(std::time::Duration::from_secs(1));
            }

            push_sender.close();
            println!("Time Push stopped");
        });

        let pull_receiver = self.pull_receiver.clone();
        let push_sender2 = self.push_sender.clone();
        std::thread::spawn(move || {
            let mut is_enabled = true;
            while is_enabled {
                if let Ok(pull_receiver) = pull_receiver.recv_blocking() {
                    if pull_receiver == 1 {
                        //DataType::Time as u8) {
                        println!("Time Provider pulled");
                        let (hour, minute) = TimeProvider::get();
                        let data = vec![DataType::Time as u8, hour, minute];
                        if let Err(_) = push_sender2.send_blocking(data) {
                            println!("PUSH_SENDER CLOSED");
                            is_enabled = false;
                        }
                    }
                } else {
                    println!("PULL_RECEIVER CLOSED");
                    is_enabled = false;
                }
            }

            pull_receiver.close();
            push_sender2.close();
            println!("Time Pull stopped");
        });
    }
}
