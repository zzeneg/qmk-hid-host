use tokio::{
    select,
    sync::{broadcast::Sender, watch},
};
use tracing::{error, info};
use windows::{
    core::HSTRING,
    Win32::{
        Media::Audio::{
            eConsole, eMultimedia, eRender,
            Endpoints::{IAudioEndpointVolume, IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl},
            IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator, AUDIO_VOLUME_NOTIFICATION_DATA,
        },
        System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, COINIT_MULTITHREADED},
    },
};

use crate::data_type::DataType;

use super::_base::Provider;

#[windows::core::implement(IAudioEndpointVolumeCallback)]
struct VolumeChangeCallback {
    push_sender: Sender<Vec<u8>>,
}

impl IAudioEndpointVolumeCallback_Impl for VolumeChangeCallback {
    fn OnNotify(&self, pnotify: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> ::windows::core::Result<()> {
        unsafe {
            let volume = ((*pnotify).fMasterVolume * 100.0) as u8;
            let data = vec![DataType::Volume as u8, volume];
            let _ = self.push_sender.send(data);
        }
        return Ok(());
    }
}

pub struct VolumeProvider {
    push_sender: Sender<Vec<u8>>,
    pull_sender: Sender<u8>,
    enabled_sender: watch::Sender<bool>,
}

impl VolumeProvider {
    pub fn new(push_sender: Sender<Vec<u8>>, pull_sender: Sender<u8>) -> Box<dyn Provider> {
        let (enabled_sender, _) = watch::channel::<bool>(true);
        let provider = VolumeProvider {
            push_sender,
            pull_sender,
            enabled_sender,
        };
        return Box::new(provider);
    }

    unsafe fn load_endpoint() -> IAudioEndpointVolume {
        CoInitializeEx(None, COINIT_MULTITHREADED).unwrap();
        let device_enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER).unwrap();
        let device: IMMDevice = device_enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia).unwrap();
        let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None).unwrap();
        return endpoint_volume;
    }

    unsafe fn get(endpoint_volume: IAudioEndpointVolume) -> u8 {
        let volume = endpoint_volume.GetMasterVolumeLevelScalar().unwrap_or_default();

        return (volume * 100.0) as u8;
    }

    fn push(push_sender: &Sender<Vec<u8>>) {
        let endpoint = unsafe { VolumeProvider::load_endpoint() };
        let volume = unsafe { VolumeProvider::get(endpoint) };
        let data = vec![DataType::Volume as u8, volume];
        let _ = push_sender.send(data);
    }
}

impl Provider for VolumeProvider {
    fn enable(&self) {
        info!("Volume Provider enabled");
        let _ = VolumeProvider::push(&self.push_sender);
        // let push_sender = self.push_sender.clone();
        let mut enabled_receiver = self.enabled_sender.subscribe();
        // let mut pull_receiver = self.pull_sender.subscribe();
        self.enabled_sender.send(true).unwrap_or_else(|e| error!("enable {}", e));
        // tokio::spawn(async move {
        //     loop {
        //         select! {
        //             _ = enabled_receiver.wait_for(|e| *e == false) => {
        //                 break;
        //             }
        //             msg = pull_receiver.recv() => {
        //                 if let Ok(received) = msg {
        //                     if received == (DataType::Volume as u8) {
        //                         info!("Volume Provider pulled");
        //                         VolumeProvider::push(&push_sender);
        //                     }
        //                 }
        //             }
        //         }
        //     }

        //     info!("Volume Provider stopped");
        // });

        let push_sender = self.push_sender.clone();
        tokio::task::spawn_blocking(move || {
            let endpoint_volume = unsafe { VolumeProvider::load_endpoint() };
            let volume_callback: IAudioEndpointVolumeCallback = VolumeChangeCallback { push_sender }.into();
            unsafe { endpoint_volume.RegisterControlChangeNotify(&volume_callback) };
            loop {
                if let Ok(changed) = enabled_receiver.has_changed() {
                    if !changed {
                        break;
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }

            let _ = unsafe { endpoint_volume.UnregisterControlChangeNotify(&volume_callback) };

            info!("Volume Provider stopped");
        });
    }

    fn disable(&self) {
        self.enabled_sender.send(false).unwrap_or_else(|e| error!("disable: {}", e));
    }
}
