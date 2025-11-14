#![cfg(target_os = "macos")]
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::data_type::DataType;

use super::_base::Provider;

fn get_weather(url: &str) -> Option<i8> {
    let output = Command::new("curl").args(["-s", url]).output();

    if let Ok(output) = output {
        if output.status.success() {
            let temp_str = String::from_utf8_lossy(&output.stdout);
            let temp_str = temp_str.trim(); // remove newline
            let temp_str = temp_str.replace(['+', '°', 'C'], "");
            if let Ok(temp) = temp_str.parse::<i8>() {
                tracing::info!("Weather Provider got temperature: {}", temp);
                return Some(temp);
            }
        }
    }
    tracing::error!("Weather Provider failed to get weather");
    None
}

fn send_data(value: &i8, host_to_device_sender: &broadcast::Sender<Vec<u8>>) {
    let data = vec![DataType::Weather as u8, *value as u8];
    if let Err(e) = host_to_device_sender.send(data) {
        tracing::error!("Weather Provider failed to send data: {:?}", e);
    }
}

pub struct WeatherProvider {
    host_to_device_sender: broadcast::Sender<Vec<u8>>,
    is_started: Arc<AtomicBool>,
    url: String,
}

impl WeatherProvider {
    pub fn new(host_to_device_sender: broadcast::Sender<Vec<u8>>, url: String) -> Box<dyn Provider> {
        let provider = WeatherProvider {
            host_to_device_sender,
            is_started: Arc::new(AtomicBool::new(false)),
            url,
        };
        return Box::new(provider);
    }
}

impl Provider for WeatherProvider {
    fn start(&self) {
        tracing::info!("Weather Provider started");
        self.is_started.store(true, Relaxed);
        let host_to_device_sender = self.host_to_device_sender.clone();
        let is_started = self.is_started.clone();
        let url = self.url.clone();
        std::thread::spawn(move || {
            let mut last_weather: Option<i8> = None;
            loop {
                if !is_started.load(Relaxed) {
                    break;
                }

                if let Some(weather) = get_weather(&url) {
                    if last_weather != Some(weather) {
                        last_weather = Some(weather);
                        send_data(&weather, &host_to_device_sender);
                    }
                }

                // Update every 15 minutes
                for _ in 0..(15 * 60) {
                    if !is_started.load(Relaxed) {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }

            tracing::info!("Weather Provider stopped");
        });
    }

    fn stop(&self) {
        self.is_started.store(false, Relaxed);
    }
}
