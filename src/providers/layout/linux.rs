use std::{collections::HashMap, process::Command};

use crate::data_type::DataType;
use breadx::{
    display::{Display, DisplayBase, DisplayConnection, DisplayExt, DisplayFunctionsExt},
    protocol::xproto::{Atom, AtomEnum, ChangeWindowAttributesAux, EventMask, GetPropertyType, Window},
};
use tokio::sync::{broadcast, mpsc};

use super::super::_base::Provider;

fn get_layout() -> String {
    // let stdout = Command::new("gsettings")
    //     .args(["get", "org.gnome.desktop.input-sources", "mru-sources"])
    //     .output()
    //     .unwrap()
    //     .stdout;

    // xkblayout-state print %s
    let stdout = Command::new("./xkblayout-state").args(["print", "%s"]).output().unwrap().stdout;

    tracing::info!("{}", stdout.len());

    // return "".to_string();

    let layout_name = String::from_utf8(stdout).unwrap();
    // let layout_name = output.split("\'").nth(3).unwrap().to_string();

    tracing::info!("{}", layout_name);

    return layout_name;
}

fn send_data(value: &String, layouts: &Vec<String>, data_sender: &mpsc::Sender<Vec<u8>>) {
    let index = layouts.into_iter().position(|r| r == value).unwrap();
    let data = vec![DataType::Layout as u8, index as u8];
    data_sender.try_send(data).unwrap_or_else(|e| tracing::error!("{}", e));
}

pub struct LayoutProvider {
    data_sender: mpsc::Sender<Vec<u8>>,
    connected_sender: broadcast::Sender<bool>,
    layouts: Vec<String>,
}

impl LayoutProvider {
    pub fn new(data_sender: mpsc::Sender<Vec<u8>>, connected_sender: broadcast::Sender<bool>, layouts: Vec<String>) -> Box<dyn Provider> {
        let provider = LayoutProvider {
            data_sender,
            connected_sender,
            layouts,
        };
        return Box::new(provider);
    }
}

impl Provider for LayoutProvider {
    fn start(&self) {
        tracing::info!("Layout Provider started");
        let data_sender = self.data_sender.clone();
        let connected_sender = self.connected_sender.clone();

        let layout = get_layout();
        send_data(&layout, &self.layouts, &data_sender);

        let mut connection = DisplayConnection::connect(None).unwrap();
        let attrs = ChangeWindowAttributesAux::new();
        let a = attrs.event_mask(EventMask::PROPERTY_CHANGE);

        connection.change_window_attributes(connection.screens()[0].root, a).ok();

        let layouts = self.layouts.clone();
        std::thread::spawn(move || {
            let mut connected_receiver = connected_sender.subscribe();
            let mut synced_layout = "".to_string();
            loop {
                if !connected_receiver.try_recv().unwrap_or(true) {
                    break;
                }

                let event = connection.wait_for_event().unwrap();
                match event {
                    breadx::protocol::Event::PropertyNotify(e) => {
                        let cookie = connection.get_atom_name(e.atom).unwrap();
                        let name = connection.wait_for_reply(cookie).unwrap().name;
                        let name_str = String::from_utf8(name).unwrap();
                        // tracing::info!("{}", name_str);

                        if name_str == "_NET_ACTIVE_WINDOW" {
                            // tracing::info!("{}", e.atom);

                            let property = connection
                                .get_property(false, e.window, e.atom, u8::from(AtomEnum::WINDOW), 0, 4)
                                .unwrap();

                            let window_id = connection.wait_for_reply(property).unwrap().value32().unwrap().nth(0).unwrap();

                            // if let Ok(window_name_str) = String::from_utf8(window_name) {
                            //     tracing::info!("{}", window_name_str);
                            // }
                            if window_id > 0 {
                                tracing::info!("window_id: {}", window_id);
                                let layout = get_layout();
                                if synced_layout != layout {
                                    synced_layout = layout;
                                    send_data(&synced_layout, &layouts, &data_sender);
                                }
                            }
                        }
                    }
                    _ => (),
                }

                // let layout = get_layout();
                // if synced_layout != layout {
                //     synced_layout = layout;
                //     send_data(&synced_layout, &layouts, &data_sender);
                // }

                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            tracing::info!("Layout Provider stopped");
        });
    }
}
