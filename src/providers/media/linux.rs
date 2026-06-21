use mpris::{Metadata, PlayerFinder};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::data_type::DataType;

use super::super::_base::Provider;

fn send_media_data(
    metadata: &Metadata,
    data_sender: &broadcast::Sender<Vec<u8>>,
    current: &(String, String),
    fallback_title: &str,
) -> (String, String) {
    let (mut artist, mut title) = current.clone();

    let new_artist = metadata.artists().and_then(|x| x.get(0).map(|x| x.to_string())).unwrap_or_default();
    if !new_artist.is_empty() && artist != new_artist {
        tracing::info!("new artist: {}", new_artist);
        artist = new_artist;
        send_data(DataType::MediaArtist, &artist, &data_sender);
    }

    let new_title = get_display_title(metadata, fallback_title);
    if !new_title.is_empty() && title != new_title {
        tracing::info!("new title: {}", new_title);
        title = new_title;
        send_data(DataType::MediaTitle, &title, &data_sender);
        send_media_player_text(&title, &data_sender);
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    return (artist, title);
}

fn get_display_title(metadata: &Metadata, fallback_title: &str) -> String {
    if let Some(title) = metadata.title().map(str::trim).filter(|title| !title.is_empty()) {
        return title.to_string();
    }

    if let Some(url) = metadata.url().map(str::trim).filter(|url| !url.is_empty()) {
        let path = url.strip_prefix("file://").unwrap_or(url);
        if let Some(file_name) = Path::new(path).file_name().and_then(|name| name.to_str()) {
            return file_name.to_string();
        }
        return url.to_string();
    }

    let fallback_title = fallback_title.trim();
    if !fallback_title.is_empty() {
        tracing::info!("media metadata has no title or url, using player identity: {}", fallback_title);
        return fallback_title.to_string();
    }

    tracing::info!("media metadata has no title, url, or player identity: {:?}", metadata);
    String::default()
}

fn send_media_player_text(value: &String, data_sender: &broadcast::Sender<Vec<u8>>) {
    let compact_text = compact_media_text(value);
    let padded_string = format!("{:<8}", compact_text);

    let mut data = vec![DataType::MediaPlayerLinux as u8];
    data.extend_from_slice(padded_string.as_bytes());

    if let Err(e) = data_sender.send(data) {
        tracing::error!("Media Provider failed to send compact media data: {:?}", e);
    }
}

fn compact_media_text(value: &str) -> String {
    let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = value.chars();
    let prefix: String = chars.by_ref().take(6).collect();

    if chars.next().is_some() {
        truncate_utf8_bytes(&format!("{}...", prefix), 10)
    } else {
        truncate_utf8_bytes(&value, 10)
    }
}

fn truncate_utf8_bytes(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_string();
    }

    let end = value
        .char_indices()
        .map(|(index, _)| index)
        .take_while(|index| *index <= max_bytes)
        .last()
        .unwrap_or(0);

    value[..end].to_string()
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
        return Box::new(provider);
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

            'outer: loop {
                if !is_started.load(Relaxed) {
                    break;
                }

                if let Ok(Ok(player)) = PlayerFinder::new().map(|x| x.find_active()) {
                    if let Ok(metadata) = player.get_metadata() {
                        media_data = send_media_data(&metadata, &data_sender, &media_data, player.identity());
                    }

                    if let Ok(events) = player.events() {
                        for event in events {
                            tracing::debug!("{:?}", event);

                            if !is_started.load(Relaxed) {
                                break 'outer;
                            }

                            match event {
                                Ok(mpris::Event::Playing) => {
                                    if let Ok(metadata) = player.get_metadata() {
                                        media_data = send_media_data(&metadata, &data_sender, &media_data, player.identity());
                                    }
                                }
                                Ok(mpris::Event::TrackChanged(metadata)) => {
                                    media_data = send_media_data(&metadata, &data_sender, &media_data, player.identity());
                                }
                                _ => (),
                            }
                        }
                    }
                }

                tracing::info!("waiting for player...");

                std::thread::sleep(std::time::Duration::from_millis(1000));
            }

            tracing::info!("Media Provider stopped");
        });
    }

    fn stop(&self) {
        self.is_started.store(false, Relaxed);
    }
}
