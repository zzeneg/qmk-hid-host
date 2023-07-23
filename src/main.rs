mod data_type;
mod keyboard;
mod providers;

use async_std::channel::{self, Sender};

use crate::{
    data_type::DataType,
    keyboard::Keyboard,
    providers::{_base::Provider, time::TimeProvider},
};

fn main() {
    let keyboard = Keyboard::new(0x0844, 0x61, 0xff60);
    let (connected_sender, connected_receiver) = channel::unbounded::<bool>();
    keyboard.connect(connected_sender.clone());
    let mut current_status = false;

    loop {
        if let Ok(connected) = connected_receiver.recv_blocking() {
            if connected != current_status {
                current_status = connected;
                let (pull_sender, pull_receiver) = channel::unbounded::<u8>();
                let (push_sender, push_receiver) = channel::unbounded::<Vec<u8>>();
                if current_status {
                    keyboard.start_reader(connected_sender.clone(), pull_sender);
                    keyboard.start_writer(connected_sender.clone(), push_receiver);
                    let providers: Vec<Box<dyn Provider>> = vec![TimeProvider::new(push_sender, pull_receiver)];
                    providers.iter().for_each(|p| p.enable());
                } else {
                    println!("Keyboard disconnected");
                    pull_sender.close();
                    pull_receiver.close();
                    push_sender.close();
                    push_receiver.close();
                    keyboard.connect(connected_sender.clone());
                }
            }
        }
    }
}
