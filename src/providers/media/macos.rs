use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;
use std::process::Command;
use tokio::sync::broadcast;

use crate::data_type::DataType;
use crate::providers::_base::Provider;

fn get_media_string() -> String {
    let command = r#"
        osascript -e 'if application "Spotify" is running then
          tell application "Spotify"
            set ps to player state
            if ps is playing then
              return name of current track
            else if ps is paused then
              return "Paused"
            else
              return ""
            end if
          end tell
        else
          return "No running"
        end if' \
        | { IFS= read -r track
            if [ "$track" = "Paused" ] || [ "$track" = "No running" ]; then
              printf '%s\n' "$track"
            elif [ "${#track}" -gt 5 ]; then
              printf '%s\n' "${track:0:5}..."
            elif [ "${#track}" -eq 0 ]; then
              :
            else
              printf '%s\n' "$track"
            fi
          }
    "#;

    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => "".to_string(),
    }
}

fn send_data(value: &str, host_to_device_sender: &broadcast::Sender<Vec<u8>>) {
    let mut padded_string = format!("{:<8}", value);
    padded_string.truncate(8);
    let string_bytes = padded_string.as_bytes();

    let mut data = vec![DataType::Spotify as u8];
    data.extend_from_slice(string_bytes);

    if let Err(e) = host_to_device_sender.send(data) {
        tracing::error!("Media Provider failed to send data: {:?}", e);
    }
}

pub struct MediaProvider {
    host_to_device_sender: broadcast::Sender<Vec<u8>>,
    is_started: Arc<AtomicBool>,
}

impl MediaProvider {
    pub fn new(host_to_device_sender: broadcast::Sender<Vec<u8>>) -> Box<dyn Provider> {
        Box::new(MediaProvider {
            host_to_device_sender,
            is_started: Arc::new(AtomicBool::new(false)),
        })
    }
}

impl Provider for MediaProvider {
    fn start(&self) {
        tracing::info!("Media Provider started");
        self.is_started.store(true, Relaxed);
        let host_to_device_sender = self.host_to_device_sender.clone();
        let is_started = self.is_started.clone();

        std::thread::spawn(move || {
            let mut last_media_string = "".to_string();
            loop {
                if !is_started.load(Relaxed) {
                    break;
                }

                let media_string = get_media_string();
                if last_media_string != media_string {
                    last_media_string = media_string.clone();
                    send_data(&media_string, &host_to_device_sender);
                }

                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            tracing::info!("Media Provider stopped");
        });
    }

    fn stop(&self) {
        self.is_started.store(false, Relaxed);
    }
}
