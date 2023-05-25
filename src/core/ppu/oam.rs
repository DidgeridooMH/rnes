#[derive(Copy, Clone, Debug)]
pub struct OamEntry {
    pub y: u8,
    pub tile_index: u8,
    pub attributes: u8,
    pub x: u8,
}

impl Default for OamEntry {
    fn default() -> Self {
        Self {
            y: 0xFF,
            tile_index: 0,
            attributes: 0,
            x: 0,
        }
    }
}
