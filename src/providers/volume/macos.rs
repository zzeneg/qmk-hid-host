use block2::{Block, RcBlock};
use coreaudio::audio_unit::macos_helpers::get_default_device_id;
use coreaudio_sys::{
    dispatch_queue_t, kAudioDevicePropertyScopeOutput, kAudioDevicePropertyVolumeScalar, kAudioHardwarePropertyDefaultOutputDevice,
    kAudioObjectPropertyElementMain, kAudioObjectPropertyScopeGlobal, kAudioObjectPropertyScopeOutput, kAudioObjectSystemObject,
    AudioObjectGetPropertyData, AudioObjectID, AudioObjectIsPropertySettable, AudioObjectPropertyAddress, OSStatus,
};
use std::option::Option;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::data_type::DataType;
use crate::providers::_base::Provider;

extern "C" {
    pub fn AudioObjectAddPropertyListenerBlock(
        in_object_id: AudioObjectID,
        in_address: *const AudioObjectPropertyAddress,
        in_dispatch_queue: dispatch_queue_t,
        in_listener: &Block<dyn Fn(u32, u64)>,
    ) -> OSStatus;

    pub fn AudioObjectRemovePropertyListenerBlock(
        in_object_id: AudioObjectID,
        in_address: *const AudioObjectPropertyAddress,
        in_dispatch_queue: dispatch_queue_t,
        in_listener: &Block<dyn Fn(u32, u64)>,
    ) -> OSStatus;
}

fn get_current_volume() -> Option<f32> {
    let device_id = get_default_device_id(false);
    if device_id.is_none() {
        return None;
    }
    let active_channel = get_channel(device_id?);
    let mut volume: f32 = 0.0;
    let mut property_size = size_of_val(&volume) as u32;
    let element = active_channel.unwrap_or(kAudioObjectPropertyElementMain as u32);

    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyVolumeScalar,
        mScope: kAudioObjectPropertyScopeOutput,
        mElement: element,
    };

    let status = unsafe {
        AudioObjectGetPropertyData(
            device_id?,
            &property_address,
            0,
            ptr::null(),
            &mut property_size,
            &mut volume as *mut _ as *mut _,
        )
    };

    if status == 0 {
        Some(volume)
    } else {
        tracing::info!("Error getting volume for device {}", device_id?);
        None
    }
}

fn is_volume_control_supported(device_id: AudioObjectID, channel: u32) -> bool {
    let mut is_writable = 0;
    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyVolumeScalar,
        mScope: kAudioDevicePropertyScopeOutput,
        mElement: channel,
    };

    let status = unsafe { AudioObjectIsPropertySettable(device_id, &property_address, &mut is_writable) };

    status == 0 && is_writable != 0
}

fn get_channel(device_id: AudioObjectID) -> Option<u32> {
    for i in 0..=1 {
        if is_volume_control_supported(device_id, i) {
            return Some(i);
        }
    }
    None
}

fn register_volume_listener(listener: &RcBlock<dyn Fn(u32, u64)>) {
    let device_id = get_default_device_id(false);
    if device_id.is_none() {
        return;
    }
    let channel = get_channel(device_id.unwrap());
    if channel.is_none() {
        return;
    }

    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyVolumeScalar,
        mScope: kAudioObjectPropertyScopeOutput,
        mElement: channel.unwrap(),
    };

    let listener_status =
        unsafe { AudioObjectRemovePropertyListenerBlock(device_id.unwrap(), &property_address, ptr::null_mut(), &listener) };
    if listener_status == 0 {
        tracing::info!(
            "Volume listener successfully removed for channel {} of device {}",
            channel.unwrap(),
            device_id.unwrap()
        );
    } else {
        tracing::info!(
            "Failed to remove volume listener for channel {} of device {}",
            channel.unwrap(),
            device_id.unwrap()
        )
    }

    let listener_status = unsafe { AudioObjectAddPropertyListenerBlock(device_id.unwrap(), &property_address, ptr::null_mut(), &listener) };

    if listener_status == 0 {
        tracing::info!(
            "Volume listener successfully registered for channel {} of device {}",
            channel.unwrap(),
            device_id.unwrap()
        );
    } else {
        tracing::info!(
            "Failed to register volume listener for channel {} of device {}",
            channel.unwrap(),
            device_id.unwrap()
        )
    }
}

fn register_device_change_listener(listener: &RcBlock<dyn Fn(u32, u64)>) {
    let property_address = AudioObjectPropertyAddress {
        mSelector: kAudioHardwarePropertyDefaultOutputDevice,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain,
    };

    let listener_status =
        unsafe { AudioObjectRemovePropertyListenerBlock(kAudioObjectSystemObject, &property_address, ptr::null_mut(), &listener) };
    if listener_status == 0 {
        tracing::info!("Default device change listener successfully removed");
    } else {
        tracing::info!("Failed to remove default device change listener");
    }

    let listener_status =
        unsafe { AudioObjectAddPropertyListenerBlock(kAudioObjectSystemObject, &property_address, ptr::null_mut(), &listener) };

    if listener_status == 0 {
        tracing::info!("Default device change listener registered successfully");
    } else {
        tracing::info!("Failed to register default device change listener");
    }
}

fn send_data(value: &f32, push_sender: &broadcast::Sender<Vec<u8>>) {
    let volume = (value * 100.0).round() as u8;
    let data = vec![DataType::Volume as u8, volume];
    push_sender.send(data).unwrap();
}

pub struct VolumeProvider {
    is_started: Arc<AtomicBool>,
    device_changed_block: RcBlock<dyn Fn(u32, u64)>,
    volume_changed_block: RcBlock<dyn Fn(u32, u64)>,
}

impl VolumeProvider {
    pub fn new(data_sender: broadcast::Sender<Vec<u8>>) -> Box<dyn Provider> {
        let sender = data_sender.clone();
        let volume_changed_block = RcBlock::new(move |_: u32, _: u64| {
            if let Some(volume) = get_current_volume() {
                send_data(&volume, &sender.clone());
            }
        });

        let sender = data_sender.clone();
        let volume_changed_block_clone = volume_changed_block.clone();
        let device_changed_block: RcBlock<dyn Fn(u32, u64)> = RcBlock::new(move |_: u32, _: u64| {
            register_volume_listener(&volume_changed_block_clone);
            if let Some(volume) = get_current_volume() {
                send_data(&volume, &sender.clone());
            }
        });

        let provider = VolumeProvider {
            is_started: Arc::new(AtomicBool::new(false)),
            device_changed_block,
            volume_changed_block,
        };
        Box::new(provider)
    }
}

impl Provider for VolumeProvider {
    fn start(&self) {
        tracing::info!("Volume Provider started");
        self.is_started.store(true, Relaxed);
        let is_started = self.is_started.clone();

        register_volume_listener(&self.volume_changed_block);
        register_device_change_listener(&self.device_changed_block);

        std::thread::spawn(move || {
            loop {
                if !is_started.load(Relaxed) {
                    break;
                }

                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            tracing::info!("Volume Provider stopped");
        });
    }

    fn stop(&self) {
        self.is_started.store(false, Relaxed);
    }
}
