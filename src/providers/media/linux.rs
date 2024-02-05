use mpris::{Metadata, PlayerFinder};
use tokio::sync::{broadcast, mpsc};

use crate::data_type::DataType;

use super::super::_base::Provider;

fn send_media_data(metadata: &Metadata, data_sender: &mpsc::Sender<Vec<u8>>, current: &(String, String)) -> (String, String) {
    let (mut artist, mut title) = current.clone();

    let new_artist = metadata.artists().map(|x| x[0]).unwrap_or_default().to_string();
    if !new_artist.is_empty() && artist != new_artist {
        tracing::info!("new artist: {}", new_artist);
        artist = new_artist;
        send_data(DataType::MediaArtist, &artist, &data_sender);
    }

    let new_title = metadata.title().unwrap_or_default().to_string();
    if !new_title.is_empty() && title != new_title {
        tracing::info!("new title: {}", new_title);
        title = new_title;
        send_data(DataType::MediaTitle, &title, &data_sender);
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    return (artist, title);
}

fn send_data(data_type: DataType, value: &String, data_sender: &mpsc::Sender<Vec<u8>>) {
    let mut data = value.to_string().into_bytes();
    data.truncate(30);
    data.insert(0, data.len() as u8);
    data.insert(0, data_type as u8);
    data_sender.try_send(data).unwrap_or_else(|e| tracing::error!("{}", e));
}

pub struct MediaProvider {
    data_sender: mpsc::Sender<Vec<u8>>,
    connected_sender: broadcast::Sender<bool>,
}

impl MediaProvider {
    pub fn new(data_sender: mpsc::Sender<Vec<u8>>, connected_sender: broadcast::Sender<bool>) -> Box<dyn Provider> {
        let provider = MediaProvider {
            data_sender,
            connected_sender,
        };
        return Box::new(provider);
    }
}

impl Provider for MediaProvider {
    fn start(&self) {
        tracing::info!("Media Provider started");

        let data_sender = self.data_sender.clone();
        let connected_sender = self.connected_sender.clone();
        std::thread::spawn(move || {
            let mut connected_receiver = connected_sender.subscribe();

            let mut media_data = (String::default(), String::default());

            loop {
                if !connected_receiver.try_recv().unwrap_or(true) {
                    break;
                }

                if let Some(player) = PlayerFinder::new().ok().and_then(|x| x.find_active().ok()) {
                    if let Ok(metadata) = player.get_metadata() {
                        media_data = send_media_data(&metadata, &data_sender, &media_data);
                    }

                    if let Ok(events) = player.events() {
                        for event in events {
                            if let Ok(event) = event {
                                tracing::debug!("{:?}", event);

                                match event {
                                    mpris::Event::Playing => {
                                        if let Ok(metadata) = player.get_metadata() {
                                            media_data = send_media_data(&metadata, &data_sender, &media_data);
                                        }
                                    }
                                    mpris::Event::TrackChanged(metadata) => {
                                        media_data = send_media_data(&metadata, &data_sender, &media_data);
                                    }
                                    _ => (),
                                }
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
}
