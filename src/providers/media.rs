use async_std::task::block_on;
use tokio::sync::{broadcast, mpsc};

use windows::{
    Foundation::{EventRegistrationToken, TypedEventHandler},
    Media::Control::{GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager},
};

use crate::data_type::DataType;

use super::_base::Provider;

fn get_manager() -> GlobalSystemMediaTransportControlsSessionManager {
    let manager = block_on(GlobalSystemMediaTransportControlsSessionManager::RequestAsync().unwrap());
    return manager.unwrap();
}

fn handle_session(session: &GlobalSystemMediaTransportControlsSession, data_sender: &mpsc::Sender<Vec<u8>>) -> EventRegistrationToken {
    let (artist, title) = get_media_data(session);
    send_data(DataType::MediaArtist, &artist, &data_sender);
    send_data(DataType::MediaTitle, &title, &data_sender);
    let mut synced_artist = artist;
    let mut synced_title = title;

    let data_sender = data_sender.clone();
    let session_handler = &TypedEventHandler::new(move |_session: &Option<GlobalSystemMediaTransportControlsSession>, _| {
        let (artist, title) = get_media_data(_session.as_ref().unwrap());
        if synced_artist != artist {
            synced_artist = artist;
            send_data(DataType::MediaArtist, &synced_artist, &data_sender);
        }

        if synced_title != title {
            synced_title = title;
            send_data(DataType::MediaTitle, &synced_title, &data_sender);
        }

        Ok(())
    });

    return session.MediaPropertiesChanged(session_handler).unwrap();
}

fn get_media_data(session: &GlobalSystemMediaTransportControlsSession) -> (String, String) {
    let media_properties = block_on(session.TryGetMediaPropertiesAsync().unwrap()).unwrap();
    let artist = media_properties.Artist().unwrap().to_string();
    let title = media_properties.Title().unwrap().to_string();

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
            let mut session_token: Option<EventRegistrationToken> = None;

            let manager = get_manager();

            if let Ok(session) = manager.GetCurrentSession() {
                session_token = Some(handle_session(&session, &data_sender));
            }

            let manager_handler = TypedEventHandler::new(move |_manager: &Option<GlobalSystemMediaTransportControlsSessionManager>, _| {
                if let Ok(session) = _manager.as_ref().unwrap().GetCurrentSession() {
                    if let Some(token) = session_token {
                        session.RemoveMediaPropertiesChanged(token).unwrap();
                    }
                    session_token = Some(handle_session(&session, &data_sender));
                }

                Ok(())
            });

            let manager_token = manager.CurrentSessionChanged(&manager_handler).unwrap();

            loop {
                if !connected_receiver.try_recv().unwrap_or(true) {
                    break;
                }

                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            manager.RemoveCurrentSessionChanged(manager_token).unwrap();

            tracing::info!("Media Provider stopped");
        });
    }
}
