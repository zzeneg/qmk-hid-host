use hidapi::{HidApi, HidDevice, HidError};

#[derive(Debug)]
pub struct HidKeyboard {
    // providers: Vec<Box<dyn Provider>>,
    keyboard: Option<HidDevice>,
    pid: u16,
    usage: u16,
    usage_page: u16,
}

impl HidKeyboard {
    pub fn new(pid: u16, usage: u16, usage_page: u16) -> Self {
        Self {
            pid,
            usage,
            usage_page,
            keyboard: None,
        }
    }

    pub fn connect(&mut self) -> Result<bool, HidError> {
        let hid_api = HidApi::new()?;
        let devices = hid_api.device_list();

        println!("{0} {1} {2}", self.pid, self.usage, self.usage_page);

        for device_info in devices {
            if device_info.product_id() == self.pid
                && device_info.usage() == self.usage
                && device_info.usage_page() == self.usage_page
            {
                let device = device_info.open_device(&hid_api)?;
                self.keyboard = Some(device);
                return Ok(true);
            }
        }

        return Ok(false);
    }

    fn is_connected(&self) -> bool {
        return self.keyboard.is_some();
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<usize, String> {
        let keyboard = self
            .keyboard
            .as_ref()
            .ok_or(String::from("not connected"))?;

        return keyboard.read(buf).map_err(|err: HidError| err.to_string());
    }

    pub fn write(&self, buf: &mut [u8]) -> Result<usize, String> {
        let keyboard = self
            .keyboard
            .as_ref()
            .ok_or(String::from("not connected"))?;

        return keyboard.write(buf).map_err(|err: HidError| err.to_string());
    }
}
