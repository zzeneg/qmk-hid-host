use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;
use tokio::sync::broadcast;
use windows::{
    core::Error,
    Win32::{
        Media::Audio::{
            eMultimedia, eRender,
            Endpoints::{IAudioEndpointVolume, IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl},
            IMMDeviceEnumerator, MMDeviceEnumerator, AUDIO_VOLUME_NOTIFICATION_DATA,
        },
        System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED},
    },
};

use crate::data_type::DataType;

use super::super::_base::Provider;

fn get_volume() -> Result<f32, ()> {
    let endpoint_volume = unsafe { get_volume_endpoint() }.map_err(|e| tracing::error!("Can not get volume endpoint: {}", e));
    return unsafe { endpoint_volume?.GetMasterVolumeLevelScalar() }.map_err(|e| tracing::error!("Can not get volume level: {}", e));
}

unsafe fn get_volume_endpoint() -> Result<IAudioEndpointVolume, Error> {
    let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    let instance: windows::core::Result<IMMDeviceEnumerator> = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER);
    return instance?
        .GetDefaultAudioEndpoint(eRender, eMultimedia)?
        .Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None);
}

#[windows::core::implement(IAudioEndpointVolumeCallback)]
struct VolumeChangeCallback {
    push_sender: broadcast::Sender<Vec<u8>>,
}

impl IAudioEndpointVolumeCallback_Impl for VolumeChangeCallback {
    fn OnNotify(&self, notification_data: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> Result<(), windows::core::Error> {
        let volume = (unsafe { *notification_data }).fMasterVolume;
        send_data(&volume, &self.push_sender);
        return Ok(());
    }
}

fn send_data(value: &f32, push_sender: &broadcast::Sender<Vec<u8>>) {
    let volume = (value * 100.0).round() as u8;
    let data = vec![DataType::Volume as u8, volume];
    push_sender.send(data).unwrap();
}

pub struct VolumeProvider {
    data_sender: broadcast::Sender<Vec<u8>>,
    is_started: Arc<AtomicBool>,
}

impl VolumeProvider {
    pub fn new(data_sender: broadcast::Sender<Vec<u8>>) -> Box<dyn Provider> {
        let provider = VolumeProvider {
            data_sender,
            is_started: Arc::new(AtomicBool::new(false)),
        };
        return Box::new(provider);
    }
}

impl Provider for VolumeProvider {
    fn start(&self) {
        tracing::info!("Volume Provider started");
        self.is_started.store(true, Relaxed);
        if let Ok(volume) = get_volume() {
            send_data(&volume, &self.data_sender);
        }

        let data_sender = self.data_sender.clone();
        let is_started = self.is_started.clone();
        std::thread::spawn(move || loop {
            if subscribe_and_wait(&data_sender, &is_started) {
                tracing::info!("Volume Provider stopped");
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(10000));
        });
    }

    fn stop(&self) {
        self.is_started.store(false, Relaxed);
    }
}

fn subscribe_and_wait(data_sender: &broadcast::Sender<Vec<u8>>, is_started: &Arc<AtomicBool>) -> bool {
    if let Ok(endpoint_volume) = unsafe { get_volume_endpoint() } {
        let push_sender = data_sender.clone();
        let volume_callback: IAudioEndpointVolumeCallback = VolumeChangeCallback { push_sender }.into();
        if let Err(e) = unsafe { endpoint_volume.RegisterControlChangeNotify(&volume_callback) } {
            tracing::error!("Can not register Volume callback: {}", e);
            return false;
        }

        loop {
            if !is_started.load(Relaxed) {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        let _ = unsafe { endpoint_volume.UnregisterControlChangeNotify(&volume_callback) };
        return true;
    }

    return false;
}
