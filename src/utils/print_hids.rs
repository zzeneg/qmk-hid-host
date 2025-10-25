use tracing::info;
use std::collections::HashSet;
use hidapi::HidApi;

pub fn print_unique_hid_devices() {
    let api = HidApi::new().unwrap();
    let mut seen = HashSet::new();

    info!("Enumerating unique HID devices...");

    for dev in api.device_list() {
        let product_name = dev.product_string().unwrap_or_default().to_string();
        let key = (dev.vendor_id(), dev.product_id(), product_name.clone());

        if !seen.insert(key.clone()) {
            continue;
        }

        if dev.product_id() == 0 || product_name.is_empty() {
            continue;
        }

        info!(
            "HID: VID={:04x}, PID={:04x}, usage_page={:?}, usage={:?}, productId=0x{:04x} product={:?}",
            dev.vendor_id(),
            dev.product_id(),
            dev.usage_page(),
            dev.usage(),
            dev.product_id(),
            product_name
        );
    }
}
