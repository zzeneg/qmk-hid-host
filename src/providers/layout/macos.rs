use crate::data_type::DataType;
use core_foundation::base::{CFRelease, TCFType};
use core_foundation::string::{CFString, CFStringRef};
use libc::c_void;
use tokio::sync::{broadcast, mpsc};
use super::super::_base::Provider;

#[link(name = "Carbon", kind = "framework")]
extern "C" {
    fn TISCopyCurrentKeyboardLayoutInputSource() -> *mut c_void;
    fn TISGetInputSourceProperty(input_source: *mut c_void, key: CFStringRef) -> *mut CFStringRef;
}

fn get_keyboard_layout() -> Option<String> {
    unsafe {

        let layout_input_source = TISCopyCurrentKeyboardLayoutInputSource();
        if layout_input_source.is_null() {
            return None;
        }

        let k_tis_property_input_source_id = CFString::from_static_string("TISPropertyInputSourceID");

        let layout_id_ptr = TISGetInputSourceProperty(layout_input_source, k_tis_property_input_source_id.as_concrete_TypeRef());
        CFRelease(layout_input_source);

        if layout_id_ptr.is_null() {
            return None;
        }

        let layout_id = layout_id_ptr as CFStringRef;
        if layout_id.is_null() {
            return None;
        }

        let layout_string = CFString::wrap_under_get_rule(layout_id).to_string();

        Some(layout_string)
    }
}

fn send_data(value: &String, layouts: &Vec<String>, data_sender: &mpsc::Sender<Vec<u8>>) {
    tracing::info!("new layout: '{0}', layout list: {1:?}", value, layouts);
    if let Some(index) = layouts.into_iter().position(|r| r == value) {
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
        Box::new(provider)
    }
}

impl Provider for LayoutProvider {
    fn start(&self) {
        tracing::info!("Layout Provider started");

        let data_sender = self.data_sender.clone();
        let layouts = self.layouts.clone();
        let connected_sender = self.connected_sender.clone();
        let mut synced_layout = "".to_string();

        std::thread::spawn(move || {
            let mut connected_receiver = connected_sender.subscribe();
            loop {
                if !connected_receiver.try_recv().unwrap_or(true) {
                    break;
                }
                if let Some(layout) = get_keyboard_layout() {
                    let lang = layout.split('.').last().unwrap().to_string();
                    if synced_layout != lang {
                        synced_layout = lang;
                        send_data(&synced_layout, &layouts, &data_sender);
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }}
        );

        tracing::info!("Layout Provider stopped");
    }
}
