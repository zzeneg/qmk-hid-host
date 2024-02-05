use std::{ffi, mem, ptr};

use crate::data_type::DataType;
use breadx::{
    display::{Display, DisplayBase, DisplayConnection, DisplayExt, DisplayFunctionsExt},
    protocol::{
        xproto::{AtomEnum, ChangeWindowAttributesAux, EventMask},
        Event,
    },
};
use tokio::sync::{broadcast, mpsc};
use x11::xlib::{XGetAtomName, XOpenDisplay, XkbAllocKeyboard, XkbGetNames, XkbGetState, _XkbStateRec};

use super::super::_base::Provider;

fn get_layout() -> String {
    unsafe {
        let display = XOpenDisplay(ptr::null());
        let keyboard = XkbAllocKeyboard();

        let mut state = mem::zeroed::<_XkbStateRec>();
        XkbGetState(display, 0x0100, &mut state);
        let current_group_index = state.group as usize;

        XkbGetNames(display, 1 << 2, keyboard);
        let symbols_atom = keyboard.read().names.read().symbols;
        let symbols_ptr = XGetAtomName(display, symbols_atom);
        let symbols = std::str::from_utf8(ffi::CStr::from_ptr(symbols_ptr).to_bytes()).unwrap_or_default();
        tracing::info!("symbols: {}", symbols);

        let current_layout = symbols.split('+').nth(current_group_index + 1).unwrap_or_default();
        let current_layout_name = current_layout.split([':', '(']).next().unwrap_or_default().to_string();
        tracing::info!("layout: {}", current_layout_name);

        return current_layout_name;
    };
}

fn send_data(value: &String, layouts: &Vec<String>, data_sender: &mpsc::Sender<Vec<u8>>) {
    let index = layouts.into_iter().position(|r| r == value);
    if let Some(index) = index {
        let data = vec![DataType::Layout as u8, index as u8];
        data_sender.try_send(data).unwrap_or_else(|e| tracing::error!("{}", e));
    }
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

        let layouts = self.layouts.clone();
        std::thread::spawn(move || {
            if let Ok(mut connection) = DisplayConnection::connect(None) {
                let attributes = ChangeWindowAttributesAux::new().event_mask(EventMask::PROPERTY_CHANGE);
                let _ = connection.change_window_attributes(connection.screens()[0].root, attributes);

                let mut connected_receiver = connected_sender.subscribe();
                let mut synced_layout = String::default();
                loop {
                    if !connected_receiver.try_recv().unwrap_or(true) {
                        break;
                    }

                    let event = connection.wait_for_event();
                    match event {
                        Ok(Event::PropertyNotify(e)) => {
                            let name_str = connection
                                .get_atom_name(e.atom)
                                .ok()
                                .map(|cookie| connection.wait_for_reply(cookie).ok())
                                .and_then(|reply| String::from_utf8(reply?.name).ok())
                                .unwrap_or_default();

                            if name_str == "_NET_ACTIVE_WINDOW" {
                                let window_id = connection
                                    .get_property(false, e.window, e.atom, u8::from(AtomEnum::WINDOW), 0, 4)
                                    .ok()
                                    .map(|property| connection.wait_for_reply(property).ok())
                                    .and_then(|reply| reply?.value32()?.next())
                                    .unwrap_or_default();

                                if window_id > 0 {
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
                }
            }

            tracing::info!("Layout Provider stopped");
        });
    }
}
