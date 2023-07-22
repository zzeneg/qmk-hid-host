use std::sync::mpsc::Sender;

use windows::Win32::{
    Globalization::{GetLocaleInfoW, LOCALE_SISO639LANGNAME},
    UI::{
        Input::KeyboardAndMouse::GetKeyboardLayout,
        TextServices::HKL,
        WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
    },
};

use crate::dataType::DataType;

use super::_base::Provider;

pub struct LayoutProvider {
    enabled: bool,
    sender: Sender<Vec<u8>>,
}

impl LayoutProvider {
    unsafe fn get(&self) -> u8 {
        const LAYOUTS: [&str; 2] = ["en", "ru"];
        let focused_window = GetForegroundWindow();
        let active_thread = GetWindowThreadProcessId(focused_window, Some(std::ptr::null_mut()));
        let layout = GetKeyboardLayout(active_thread);
        let locale_id = (std::mem::transmute::<HKL, u64>(layout) & 0xFFFF) as u32;
        let mut layout_name = [0u16; 9];
        let _ = GetLocaleInfoW(
            locale_id,
            LOCALE_SISO639LANGNAME,
            Some(layout_name.as_mut()),
        );

        let layout = String::from_utf16(&layout_name).unwrap_or_default();

        let index = LAYOUTS
            .iter()
            .position(|&r| r == layout)
            .unwrap_or_default();

        return index as u8;
    }
}

impl Provider for LayoutProvider {
    fn new(sender: Sender<Vec<u8>>) -> Self {
        Self {
            enabled: false,
            sender,
        }
    }

    fn enable(&mut self) {
        self.enabled = true;
        self.send();
    }

    fn disable(&mut self) {
        self.enabled = false;
    }

    fn send(&self) {
        let layout_index = unsafe { self.get() };
        let data = [DataType::Layout as u8, layout_index];
        let _ = self.sender.send(data.to_vec());
    }
}
