use std::path::PathBuf;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub device: Device,
    pub layouts: Vec<String>,
    pub reconnect_delay: u64,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub product_id: u16,
    pub usage: u16,
    pub usage_page: u16,
}

pub fn get_config(maybe_path: Option<PathBuf>) -> Config {
    let default_config = Config {
        device: Device {
            product_id: 0x0844,
            usage: 0x61,
            usage_page: 0xff60,
        },
        layouts: vec!["pl".to_string()],
        reconnect_delay: 5000,
    };

    let path = maybe_path.unwrap_or("./qmk-hid-host.json".into());

    if let Ok(file) = std::fs::read_to_string(&path) {
        if let Ok(file_config) = serde_json::from_str::<Config>(&file) {
            tracing::info!("Read config from file {:?}", path);
            return file_config;
        }

        tracing::error!("Error while reading config from file {:?}", path);
    }

    let file_content = serde_json::to_string_pretty(&default_config).unwrap();
    std::fs::write(&path, &file_content).unwrap();
    tracing::info!("New config file created at {:?}", path);

    return default_config;
}
