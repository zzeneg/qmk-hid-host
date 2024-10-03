use std::path::PathBuf;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub devices: Vec<Device>,
    pub layouts: Vec<String>,
    pub reconnect_delay: Option<u64>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub name: Option<String>,
    #[serde(deserialize_with = "string_to_hex")]
    pub product_id: u16,
    pub usage: Option<u16>,
    pub usage_page: Option<u16>,
}

pub fn get_config(maybe_path: Option<PathBuf>) -> Config {
    let default_config = Config {
        devices: vec![Device {
            name: None,
            product_id: 0x0844,
            usage: None,
            usage_page: None,
        }],
        layouts: vec!["en".to_string()],
        reconnect_delay: None,
    };

    let path = maybe_path.unwrap_or("./qmk-hid-host.json".into());

    if let Ok(file) = std::fs::read_to_string(&path) {
        return serde_json::from_str::<Config>(&file)
            .map_err(|e| tracing::error!("Incorrect config file: {}", e))
            .unwrap();
    }

    let file_content = serde_json::to_string_pretty(&default_config).unwrap();
    std::fs::write(&path, &file_content).unwrap();
    tracing::info!("New config file created at {:?}", path);

    return default_config;
}

fn string_to_hex<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: &str = serde::Deserialize::deserialize(deserializer)?;
    let hex = value.trim_start_matches("0x");
    return u16::from_str_radix(hex, 16).map_err(serde::de::Error::custom);
}
