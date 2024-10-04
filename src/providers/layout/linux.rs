use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;
use std::{ffi, mem, ptr};
use tokio::sync::broadcast;
use x11::xlib::{XGetAtomName, XOpenDisplay, XkbAllocKeyboard, XkbGetNames, XkbGetState, _XDisplay, _XkbDesc, _XkbStateRec};

use crate::config::get_config;
use crate::data_type::DataType;

use super::super::_base::Provider;

fn get_symbols(display: *mut _XDisplay, keyboard: *mut _XkbDesc) -> String {
    unsafe { XkbGetNames(display, 1 << 2, keyboard) };
    let symbols_atom = unsafe { keyboard.read().names.read().symbols };
    let symbols_ptr = unsafe { XGetAtomName(display, symbols_atom) };
    let symbols_cstr = unsafe { ffi::CStr::from_ptr(symbols_ptr) };
    let symbols = String::from_utf8(symbols_cstr.to_bytes().to_vec()).unwrap_or_default();

    tracing::info!("layout symbols: {}", symbols);

    return symbols;
}

fn get_layout_index(display: *mut _XDisplay) -> usize {
    let mut state = unsafe { mem::zeroed::<_XkbStateRec>() };
    unsafe { XkbGetState(display, 0x0100, &mut state) };
    return state.group as usize;
}

fn send_data(value: &String, layouts: &Vec<String>, data_sender: &broadcast::Sender<Vec<u8>>) {
    tracing::info!("new layout: '{0}', layout list: {1:?}", value, layouts);
    let index = layouts.into_iter().position(|r| r == value);
    if let Some(index) = index {
        let data = vec![DataType::Layout as u8, index as u8];
        data_sender.send(data).unwrap();
    }
}

pub struct LayoutProvider {
    data_sender: broadcast::Sender<Vec<u8>>,
    is_started: Arc<AtomicBool>,
}

impl LayoutProvider {
    pub fn new(data_sender: broadcast::Sender<Vec<u8>>) -> Box<dyn Provider> {
        let provider = LayoutProvider {
            data_sender,
            is_started: Arc::new(AtomicBool::new(false)),
        };
        return Box::new(provider);
    }
}

impl Provider for LayoutProvider {
    fn start(&self) {
        tracing::info!("Layout Provider started");
        self.is_started.store(true, Relaxed);
        let layouts = &get_config().layouts;
        let data_sender = self.data_sender.clone();
        let is_started = self.is_started.clone();
        std::thread::spawn(move || {
            let mut synced_layout = 0;
            let display = unsafe { XOpenDisplay(ptr::null()) };
            let keyboard = unsafe { XkbAllocKeyboard() };
            let symbols = get_symbols(display, keyboard);
            let symbol_list = symbols.split('+').map(|x| x.to_string()).collect::<Vec<String>>();

            loop {
                if !is_started.load(Relaxed) {
                    break;
                }

                let layout = get_layout_index(display);
                if synced_layout != layout {
                    synced_layout = layout;
                    let layout_symbol = symbol_list.get(layout + 1).map(|x| x.to_string()).unwrap_or_default();
                    let layout_name = layout_symbol.split([':', '(']).next().unwrap_or_default().to_string();
                    send_data(&layout_name, layouts, &data_sender);
                }

                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            tracing::info!("Layout Provider stopped");
        });
    }

    fn stop(&self) {
        self.is_started.store(false, Relaxed);
    }
}
