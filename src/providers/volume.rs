// use async_std::channel::{Receiver, Sender};

// use windows::Win32::{
//     Media::Audio::{eConsole, eRender, Endpoints::IAudioEndpointVolume, IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator},
//     System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED},
// };

// use crate::data_type::DataType;

// use super::_base::Provider;

// pub struct VolumeProvider {
//     push_sender: Sender<Vec<u8>>,
//     pull_receiver: Receiver<u8>,
// }

// impl VolumeProvider {
//     pub fn new(push_sender: Sender<Vec<u8>>, pull_receiver: Receiver<u8>) -> Box<dyn Provider> {
//         let provider = VolumeProvider {
//             push_sender,
//             pull_receiver,
//         };
//         return Box::new(provider);
//     }

//     unsafe fn get(&self) -> u8 {
//         CoInitializeEx(None, COINIT_APARTMENTTHREADED).unwrap();
//         let device_enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER).unwrap();
//         let device: IMMDevice = device_enumerator.GetDefaultAudioEndpoint(eRender, eConsole).unwrap();
//         let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None).unwrap();
//         let volume = endpoint_volume.GetMasterVolumeLevelScalar().unwrap_or_default();

//         return (volume * 100.0) as u8;
//     }
// }

// impl Provider for VolumeProvider {
//     fn new(sender: Sender<Vec<u8>>) -> Self {
//         Self { enabled: false, sender }
//     }

//     fn enable(&mut self) {
//         self.enabled = true;
//         self.send();
//     }

//     fn disable(&mut self) {
//         self.enabled = false;
//     }

//     fn send(&self) {
//         let volume = unsafe { self.get() };
//         let data = [DataType::Volume as u8, volume];
//         let _ = self.sender.send(data.to_vec());
//     }
// }
