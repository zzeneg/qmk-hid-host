mod dataType;
mod hidKeyboard;
mod providers;

use crate::{
    hidKeyboard::HidKeyboard,
    providers::{
        _base::Provider, layout::LayoutProvider, media::MediaProvider, time::TimeProvider,
        volume::VolumeProvider,
    },
};

#[async_std::main]
async fn main() {
    let mut keyboard = HidKeyboard::new(0x0844, 0x61, 0xff60);

    let connected = keyboard.connect().map_err(|e| eprintln!("{}", e)).unwrap();
    // if connected {
    //     let mut buf = [0u8; 32];
    //     let read_result = keyboard.read(&mut buf).unwrap_or_default();
    //     println!("read_result: {}", read_result);
    //     let read_value = String::from_utf8((&buf[..read_result - 1]).to_vec()).unwrap();
    //     println!("read: {}", read_value);
    // }

    let (sender, receiver) = std::sync::mpsc::channel::<Vec<u8>>();

    // let providers = [TimeProvider, VolumeProvider];
    let sender_time = sender.clone();
    std::thread::spawn(move || {
        let mut time_provider = TimeProvider::new(sender_time);
        time_provider.enable();
    });

    let sender_volume = sender.clone();
    std::thread::spawn(move || {
        let mut volume_provider = VolumeProvider::new(sender_volume);
        volume_provider.enable();
    });

    let sender_layout = sender.clone();
    std::thread::spawn(move || {
        let mut layout_provider = LayoutProvider::new(sender_layout);
        layout_provider.enable();
    });

    let sender_media = sender.clone();
    std::thread::spawn(move || {
        let mut media_provider = MediaProvider::new(sender_media);
        media_provider.enable();
    });

    loop {
        let mut data = receiver.recv().unwrap_or_default();
        println!("data to send: {:?}", data);
        data.truncate(32);
        data.insert(0, 0);

        let _ = keyboard.write(data.as_mut());
    }
}
