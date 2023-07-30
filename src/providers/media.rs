use async_std::task::block_on;
use tokio::sync::{broadcast, mpsc};

use windows::{
    core::Error,
    Foundation::TypedEventHandler,
    Media::Control::{GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager},
};

use crate::data_type::DataType;

use super::_base::Provider;

async fn get_media_session() -> Result<GlobalSystemMediaTransportControlsSession, Error> {
    let mp = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.await?;
    let session = mp.GetCurrentSession();
    return session;
}

async fn get_media_data(media_session: &GlobalSystemMediaTransportControlsSession) -> Result<(String, String), Error> {
    let media_properties = media_session.TryGetMediaPropertiesAsync()?.await?;
    let artist = media_properties.Artist().unwrap().to_string();
    let title = media_properties.Title().unwrap().to_string();

    return Ok((artist, title));
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
            let mut synced_artist = "".to_string();
            let mut synced_title = "".to_string();

            if let Ok(media_session) = block_on(get_media_session()) {
                if let Ok((artist, title)) = block_on(get_media_data(&media_session)) {
                    synced_artist = artist;
                    send_data(DataType::MediaArtist, &synced_artist, &data_sender);
                    synced_title = title;
                    send_data(DataType::MediaTitle, &synced_title, &data_sender);
                }

                let handler = &TypedEventHandler::new(move |_session, _| {
                    if let Some(session) = _session {
                        if let Ok((artist, title)) = block_on(get_media_data(session)) {
                            if synced_artist != artist {
                                synced_artist = artist;
                                send_data(DataType::MediaArtist, &synced_artist, &data_sender);
                            }

                            if synced_title != title {
                                synced_title = title;
                                send_data(DataType::MediaTitle, &synced_title, &data_sender);
                            }
                        }
                    }

                    return Ok(());
                });
                let token = media_session.MediaPropertiesChanged(handler).unwrap();

                loop {
                    if !connected_receiver.try_recv().unwrap_or(true) {
                        break;
                    }

                    std::thread::sleep(std::time::Duration::from_millis(100));
                }

                media_session
                    .RemoveMediaPropertiesChanged(token)
                    .unwrap_or_else(|e| tracing::error!("{}", e));
            }

            tracing::info!("Media Provider stopped");
        });
    }
}
