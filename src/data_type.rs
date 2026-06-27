#[cfg(not(target_os = "macos"))]
pub enum DataType {
    Time = 0xAA, // random value that does not conflict with VIA/VIAL, must match firmware
    Volume,
    Layout,
    MediaArtist = 0xAD,
    MediaTitle = 0xAE,
    Weather = 0xAF,
    MediaPlayerLinux = 0xB0,
    MediaExtended = 0xB1,

    RelayFromDevice = 0xCC,
    RelayToDevice,
}

#[cfg(target_os = "macos")]
pub enum DataType {
    Time = 0xAA, // random value that does not conflict with VIA/VIAL, must match firmware
    Volume,
    Layout,
    Spotify = 0xAE,
    Weather = 0xAF,

    RelayFromDevice = 0xCC,
    RelayToDevice,
}
