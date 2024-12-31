use apple_music::{AppleMusic, Track};
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::data_type::DataType;

use super::super::_base::Provider;

fn send_media_data(track: &Track, data_sender: &broadcast::Sender<Vec<u8>>, current: &(String, String)) -> (String, String) {
    let (mut artist, mut title) = current.clone();

    let new_artist = track.artist.to_string();
    if !new_artist.is_empty() && artist != new_artist {
        tracing::info!("new artist: {}", new_artist);
        artist = new_artist;
        send_data(DataType::MediaArtist, &artist, &data_sender);
    }

    let new_title = track.name.to_string();
    if !new_title.is_empty() && title != new_title {
        tracing::info!("new title: {}", new_title);
        title = new_title;
        send_data(DataType::MediaTitle, &title, &data_sender);
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    (artist, title)
}

fn send_data(data_type: DataType, value: &String, data_sender: &broadcast::Sender<Vec<u8>>) {
    let mut data = value.to_string().into_bytes();
    data.truncate(30);
    data.insert(0, data.len() as u8);
    data.insert(0, data_type as u8);
    data_sender.send(data).unwrap();
}

pub struct MediaProvider {
    data_sender: broadcast::Sender<Vec<u8>>,
    is_started: Arc<AtomicBool>,
}

impl MediaProvider {
    pub fn new(data_sender: broadcast::Sender<Vec<u8>>) -> Box<dyn Provider> {
        let provider = MediaProvider {
            data_sender,
            is_started: Arc::new(AtomicBool::new(false)),
        };
        Box::new(provider)
    }
}

impl Provider for MediaProvider {
    fn start(&self) {
        tracing::info!("Media Provider started");
        self.is_started.store(true, Relaxed);
        let data_sender = self.data_sender.clone();
        let is_started = self.is_started.clone();
        std::thread::spawn(move || {
            let mut media_data = (String::default(), String::default());

            loop {
                if !is_started.load(Relaxed) {
                    break;
                }

                if let Ok(application_data) = AppleMusic::get_application_data() {
                    if application_data.player_position.is_none() {
                        continue;
                    }

                    if let Ok(current_track) = AppleMusic::get_current_track() {
                        media_data = send_media_data(&current_track, &data_sender, &media_data);
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(1000));
            }

            tracing::info!("Media Provider stopped");
        });
    }

    fn stop(&self) {
        self.is_started.store(false, Relaxed);
    }
}
