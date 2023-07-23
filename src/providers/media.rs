// use std::{sync::mpsc::Sender, time::Instant};

// use async_std::task::block_on;

// use windows::{
//     core::{Error, HSTRING},
//     Media::Control::{
//         GlobalSystemMediaTransportControlsSessionManager,
//         GlobalSystemMediaTransportControlsSessionMediaProperties,
//     },
// };

// use crate::data_type::DataType;

// use super::_base::Provider;

// pub struct MediaProvider {
//     enabled: bool,
//     sender: Sender<Vec<u8>>,
// }

// impl MediaProvider {
//     async fn get_media_properties(
//         &self,
//     ) -> Result<GlobalSystemMediaTransportControlsSessionMediaProperties, Error> {
//         let mp = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.await?;
//         let session = mp.GetCurrentSession()?;
//         let properties = session.TryGetMediaPropertiesAsync()?.await?;
//         Ok(properties)
//     }

//     fn hstring_to_vec(&self, str: HSTRING) -> Vec<u8> {
//         let mut data = str.to_string().into_bytes();
//         data.truncate(30);
//         data.insert(0, data.len() as u8);
//         return data;
//     }
// }

// impl Provider for MediaProvider {
//     fn new(sender: Sender<Vec<u8>>) -> Self {
//         Self {
//             enabled: false,
//             sender,
//         }
//     }

//     fn enable(&mut self) {
//         self.enabled = true;
//         let mut start = Instant::now();
//         self.send();
//         while self.enabled {
//             if start.elapsed().as_secs() > 30 {
//                 start = Instant::now();
//                 self.send();
//             }
//         }
//     }

//     fn disable(&mut self) {
//         self.enabled = false;
//     }

//     fn send(&self) {
//         let properties = block_on(self.get_media_properties()).unwrap();
//         let artist = properties.Artist().unwrap();
//         let mut artist_data = self.hstring_to_vec(artist);
//         artist_data.insert(0, DataType::MediaArtist as u8);
//         let _ = self.sender.send(artist_data.to_vec());

//         let title = properties.Title().unwrap();
//         let mut title_data = self.hstring_to_vec(title);
//         title_data.insert(0, DataType::MediaTitle as u8);
//         let _ = self.sender.send(title_data.to_vec());
//     }
// }
