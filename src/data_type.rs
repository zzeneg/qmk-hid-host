pub enum DataType {
    Time = 0xAA, // random value that does not conflict with VIA/VIAL, must match firmware
    Volume,
    Layout,
    MediaArtist,
    MediaTitle,
}
