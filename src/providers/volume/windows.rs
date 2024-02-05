use tokio::sync::{
    broadcast::{self, Receiver},
    mpsc::{self, Sender},
};
use windows::Win32::{
    Media::Audio::{
        eMultimedia, eRender,
        Endpoints::{IAudioEndpointVolume, IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl},
        IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator, AUDIO_VOLUME_NOTIFICATION_DATA,
    },
    System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED},
};

use crate::data_type::DataType;

use super::super::_base::Provider;

fn get_volume() -> f32 {
    let endpoint_volume = unsafe { get_volume_endpoint() };
    return unsafe { endpoint_volume.GetMasterVolumeLevelScalar() }.unwrap();
}

unsafe fn get_volume_endpoint() -> IAudioEndpointVolume {
    CoInitializeEx(None, COINIT_MULTITHREADED).unwrap_or_else(|e| tracing::error!("{}", e));
    let device_enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER).unwrap();
    let device: IMMDevice = device_enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia).unwrap();
    let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None).unwrap();
    return endpoint_volume;
}

#[windows::core::implement(IAudioEndpointVolumeCallback)]
struct VolumeChangeCallback {
    push_sender: mpsc::Sender<Vec<u8>>,
}

impl IAudioEndpointVolumeCallback_Impl for VolumeChangeCallback {
    fn OnNotify(&self, notification_data: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> Result<(), windows::core::Error> {
        let volume = (unsafe { *notification_data }).fMasterVolume;
        send_data(&volume, &self.push_sender);
        return Ok(());
    }
}

fn send_data(value: &f32, push_sender: &mpsc::Sender<Vec<u8>>) {
    let volume = (value * 100.0).round() as u8;
    let data = vec![DataType::Volume as u8, volume];
    push_sender.try_send(data).unwrap_or_else(|e| tracing::error!("{}", e));
}

pub struct VolumeProvider {
    data_sender: mpsc::Sender<Vec<u8>>,
    connected_sender: broadcast::Sender<bool>,
}

impl VolumeProvider {
    pub fn new(data_sender: mpsc::Sender<Vec<u8>>, connected_sender: broadcast::Sender<bool>) -> Box<dyn Provider> {
        let provider = VolumeProvider {
            data_sender,
            connected_sender,
        };
        return Box::new(provider);
    }
}

impl Provider for VolumeProvider {
    fn start(&self) {
        tracing::info!("Volume Provider started");
        let volume = get_volume();
        send_data(&volume, &self.data_sender);
        let data_sender = self.data_sender.clone();
        let connected_sender = self.connected_sender.clone();
        std::thread::spawn(move || {
            let connected_receiver = connected_sender.subscribe();
            subscribe(data_sender, connected_receiver);
            tracing::info!("Volume Provider stopped");
        });
    }
}

fn subscribe(data_sender: Sender<Vec<u8>>, mut connected_receiver: Receiver<bool>) {
    let endpoint_volume = unsafe { get_volume_endpoint() };
    let volume_callback: IAudioEndpointVolumeCallback = VolumeChangeCallback { push_sender: data_sender }.into();
    unsafe { endpoint_volume.RegisterControlChangeNotify(&volume_callback) }.unwrap_or_else(|e| tracing::error!("{}", e));
    loop {
        if !connected_receiver.try_recv().unwrap_or(true) {
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    unsafe { endpoint_volume.UnregisterControlChangeNotify(&volume_callback) }.unwrap_or_else(|e| tracing::error!("{}", e));
}
