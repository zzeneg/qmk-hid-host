use std::{path::PathBuf, sync::OnceLock};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub devices: Vec<Device>,
    pub layouts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reconnect_delay: Option<u64>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(serialize_with = "hex_to_string", deserialize_with = "string_to_hex")]
    pub product_id: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_page: Option<u16>,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn get_config() -> &'static Config {
    CONFIG.get().unwrap()
}

pub fn load_config(path: PathBuf) -> &'static Config {
    if let Some(config) = CONFIG.get() {
        return config;
    }

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

    if let Ok(file) = std::fs::read_to_string(&path) {
        let config = serde_json::from_str::<Config>(&file)
            .map_err(|e| tracing::error!("Incorrect config file: {}", e))
            .unwrap();
        return CONFIG.get_or_init(|| config);
    }

    let file_content = serde_json::to_string_pretty(&default_config).unwrap();
    std::fs::write(&path, &file_content)
        .map_err(|e| tracing::error!("Error while saving config file to {:?}: {}", path, e))
        .unwrap();
    tracing::info!("New config file created at {:?}", path);

    CONFIG.get_or_init(|| default_config)
}

fn string_to_hex<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: &str = serde::Deserialize::deserialize(deserializer)?;
    let hex = value.trim_start_matches("0x");
    return u16::from_str_radix(hex, 16).map_err(serde::de::Error::custom);
}

fn hex_to_string<S>(value: &u16, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&format!("0x{:04x}", value))
}
