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
        send_media_raw(DataType::MediaArtist, &artist, &data_sender);
    }

    let new_title = get_display_title(metadata, fallback_title);
    if !new_title.is_empty() && title != new_title {
        tracing::info!("new title: {}", new_title);
        title = new_title;
        send_media_raw(DataType::MediaTitle, &title, &data_sender);
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

fn send_media_raw(data_type: DataType, value: &str, data_sender: &broadcast::Sender<Vec<u8>>) {
    let mut data = value.to_string().into_bytes();
    data.truncate(30);
    data.insert(0, data.len() as u8);
    data.insert(0, data_type as u8);
    if let Err(e) = data_sender.send(data) {
        tracing::error!("Media Provider failed to send data: {:?}", e);
    }
}

fn send_media_player_text(value: &str, data_sender: &broadcast::Sender<Vec<u8>>) {
    let mut payload = compact_media_text(value).into_bytes();
    payload.resize(8, b' ');

    let mut data = vec![DataType::MediaPlayerLinux as u8];
    data.extend_from_slice(&payload);

    if let Err(e) = data_sender.send(data) {
        tracing::error!("Media Provider failed to send compact media data: {:?}", e);
    }
}

fn compact_media_text(value: &str) -> String {
    let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = value.chars();
    let prefix: String = chars.by_ref().take(6).collect();

    if chars.next().is_some() {
        truncate_utf8_bytes(&format!("{}..", prefix), 8)
    } else {
        truncate_utf8_bytes(&value, 8)
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

fn truncate_to_bytes(value: &str, max_bytes: usize) -> &str {
    if value.len() <= max_bytes {
        return value;
    }
    let mut boundary = max_bytes;
    while !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    &value[..boundary]
}

fn send_media_extended(
    data_sender: &broadcast::Sender<Vec<u8>>,
    total_time: u16,
    position: u16,
    status: u8,
    artist: &str,
) {
    let artist = truncate_to_bytes(artist, 21);

    let mut data = vec![DataType::MediaExtended as u8];
    data.extend_from_slice(&total_time.to_le_bytes());
    data.extend_from_slice(&position.to_le_bytes());
    data.push(status);
    data.push(artist.len() as u8);
    data.extend_from_slice(artist.as_bytes());

    if let Err(e) = data_sender.send(data) {
        tracing::error!("Media Provider failed to send extended media data: {:?}", e);
    }
}

pub struct MediaProvider {
    data_sender: broadcast::Sender<Vec<u8>>,
    is_started: Arc<AtomicBool>,
    extended: bool,
}

impl MediaProvider {
    pub fn new(data_sender: broadcast::Sender<Vec<u8>>, extended: bool) -> Box<dyn Provider> {
        let provider = MediaProvider {
            data_sender,
            is_started: Arc::new(AtomicBool::new(false)),
            extended,
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
        let extended = self.extended;
        std::thread::spawn(move || {
            let mut media_data = (String::default(), String::default());

            'outer: loop {
                if !is_started.load(Relaxed) {
                    break;
                }

                if let Ok(Ok(player)) = PlayerFinder::new().map(|x| x.find_active()) {
                    let metadata = player.get_metadata().ok();
                    if let Some(ref metadata) = metadata {
                        media_data = send_media_data(metadata, &data_sender, &media_data, player.identity());
                        if extended {
                            let status = player.get_playback_status().ok().map(status_to_byte).unwrap_or(0);
                            let (total_time, position) = get_time_info(&player, metadata);
                            send_media_extended(&data_sender, total_time, position, status, &media_data.0);
                        }
                    }

                    if let Ok(events) = player.events() {
                        for event in events {
                            tracing::debug!("{:?}", event);

                            if !is_started.load(Relaxed) {
                                break 'outer;
                            }

                            match event {
                                Ok(mpris::Event::Playing) | Ok(mpris::Event::Paused) => {
                                    if let Ok(metadata) = player.get_metadata() {
                                        let status = player.get_playback_status().ok().map(status_to_byte).unwrap_or(0);
                                        media_data = send_media_data(&metadata, &data_sender, &media_data, player.identity());
                                        if extended {
                                            let (total_time, position) = get_time_info(&player, &metadata);
                                            send_media_extended(&data_sender, total_time, position, status, &media_data.0);
                                        }
                                    }
                                }
                                Ok(mpris::Event::TrackChanged(metadata)) => {
                                    let status = player.get_playback_status().ok().map(status_to_byte).unwrap_or(0);
                                    media_data = send_media_data(&metadata, &data_sender, &media_data, player.identity());
                                    if extended {
                                        let (total_time, position) = get_time_info(&player, &metadata);
                                        send_media_extended(&data_sender, total_time, position, status, &media_data.0);
                                    }
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

fn get_time_info(player: &mpris::Player, metadata: &mpris::Metadata) -> (u16, u16) {
    let total_time = metadata.length_in_microseconds().map(|us| (us / 1_000_000).min(65535) as u16).unwrap_or(0);
    let position = player.get_position_in_microseconds().ok().map(|us| (us / 1_000_000).min(65535) as u16).unwrap_or(0);
    (total_time, position)
}

fn status_to_byte(status: mpris::PlaybackStatus) -> u8 {
    match status {
        mpris::PlaybackStatus::Playing => 1,
        mpris::PlaybackStatus::Paused => 2,
        mpris::PlaybackStatus::Stopped => 0,
    }
}
