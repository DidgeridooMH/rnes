use crate::core::Addressable;

pub struct VRam {
    nametables: [u8; 0x1000],
    palette: [u8; 0x20],
}

impl VRam {
    pub fn new() -> Self {
        Self {
            nametables: [0; 0x1000],
            palette: [0; 0x20],
        }
    }
}

impl Addressable for VRam {
    fn read_byte(&mut self, address: u16) -> u8 {
        match address {
            // TODO: Mirroring configuration needs to be added.
            0x2000..=0x2FFF => self.nametables[(address as usize - 0x2000) % 0x400],
            0x3000..=0x3EFF => self.read_byte(address - 0x1000),
            0x3F00..=0x3FFF => self.palette[(address as usize - 0x3F00) % 0x20],
            _ => {
                eprintln!("Unexpected VRAM read at {address:X}");
                0
            }
        }
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        match address {
            0x2000..=0x2FFF => self.nametables[(address as usize - 0x2000) % 0x400] = data,
            0x3000..=0x3EFF => self.write_byte(address - 0x1000, data),
            0x3F00..=0x3FFF => self.palette[(address as usize - 0x3F00) % 0x20] = data,
            _ => {
                eprintln!("Unexpected VRAM write at {address:X}");
            }
        }
    }
}
