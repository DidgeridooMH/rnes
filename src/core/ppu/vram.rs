use crate::{core::Addressable, rom::MirrorArrangement};

pub struct VRam {
    nametables: [u8; 0x1000],
    palette: [u8; 0x20],
    mirroring: MirrorArrangement,
}

impl VRam {
    pub fn new(mirroring: MirrorArrangement) -> Self {
        Self {
            nametables: [0; 0x1000],
            palette: [0; 0x20],
            mirroring,
        }
    }

    fn get_mirror_address(&self, address: u16) -> u16 {
        match self.mirroring {
            MirrorArrangement::Vertical => address % 0x800 + 0x2000,
            MirrorArrangement::Horizontal => {
                if address % 0x800 > 0x400 {
                    address - 0x400
                } else {
                    address
                }
            }
        }
    }
}

impl Addressable for VRam {
    fn read_byte(&mut self, address: u16) -> u8 {
        match address {
            0x2000..=0x2FFF => self.nametables[self.get_mirror_address(address) as usize - 0x2000],
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
            0x2000..=0x2FFF => {
                self.nametables[self.get_mirror_address(address) as usize - 0x2000] = data
            }
            0x3000..=0x3EFF => self.write_byte(address - 0x1000, data),
            0x3F00..=0x3FFF => self.palette[(address as usize - 0x3F00) % 0x20] = data,
            _ => {
                eprintln!("Unexpected VRAM write at {address:X}");
            }
        }
    }
}
