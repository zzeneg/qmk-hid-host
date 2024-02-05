use std::ops::Deref;

use libpulse_binding::context::subscribe::Facility;
use pulsectl::controllers::{DeviceControl, SinkController};
use tokio::sync::{broadcast, mpsc};

use crate::data_type::DataType;

use super::super::_base::Provider;

fn get_volume() -> Option<f32> {
    let mut controller = SinkController::create().ok()?;
    if let Ok(default) = controller.get_default_device() {
        let device_volume = default.volume.get().first()?.0 as f32;
        let base_volume = default.base_volume.0 as f32;
        return Some(device_volume / base_volume);
    }

    return None;
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
        let data_sender = self.data_sender.clone();
        let connected_sender = self.connected_sender.clone();

        let mut volume = get_volume().unwrap_or_default();
        send_data(&volume, &self.data_sender);

        std::thread::spawn(move || {
            let mut connected_receiver = connected_sender.subscribe();

            let controller = SinkController::create().map_err(|e| tracing::error!("{}", e)).unwrap();
            let mut ctx = controller.handler.context.deref().borrow_mut();

            ctx.set_subscribe_callback(Some(Box::new(move |_, _, _| {
                let new_volume = get_volume().unwrap_or_default();
                if volume != new_volume {
                    volume = new_volume;
                    send_data(&volume, &data_sender);
                }
            })));

            ctx.subscribe(Facility::Sink.to_interest_mask(), |_| {});

            loop {
                if !connected_receiver.try_recv().unwrap_or(true) {
                    break;
                }

                controller.handler.mainloop.deref().borrow_mut().iterate(true);
            }

            tracing::info!("Volume Provider stopped");
        });
    }
}
