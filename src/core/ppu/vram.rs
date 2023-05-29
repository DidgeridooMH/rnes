use crate::{core::Addressable, rom::MirrorArrangement};

pub struct VRam {
    nametable0: [u8; 0x400],
    nametable1: [u8; 0x400],
    palette: [u8; 0x20],
    mirroring: MirrorArrangement,
}

impl Default for VRam {
    fn default() -> Self {
        Self {
            nametable0: [0; 0x400],
            nametable1: [0; 0x400],
            palette: [0; 0x20],
            mirroring: MirrorArrangement::Horizontal,
        }
    }
}

impl VRam {
    pub fn set_mirroring(&mut self, mirroring: MirrorArrangement) {
        self.mirroring = mirroring;
    }
}

impl Addressable for VRam {
    fn read_byte(&mut self, address: u16) -> u8 {
        match address {
            0x2000..=0x2FFF => {
                let address = address as usize;
                match self.mirroring {
                    MirrorArrangement::Vertical => match address {
                        0x2000..=0x23FF => self.nametable0[address - 0x2000],
                        0x2400..=0x27FF => self.nametable1[address - 0x2400],
                        0x2800..=0x2BFF => self.nametable0[address - 0x2800],
                        0x2C00..=0x2FFF => self.nametable1[address - 0x2C00],
                        _ => unreachable!(),
                    },
                    MirrorArrangement::Horizontal => match address {
                        0x2000..=0x27FF => self.nametable0[address % 0x400],
                        0x2800..=0x2FFF => self.nametable1[address % 0x400],
                        _ => unreachable!(),
                    },
                    MirrorArrangement::OneScreenLower => self.nametable0[address % 0x400],
                    MirrorArrangement::OneScreenUpper => self.nametable1[address % 0x400],
                }
            }
            0x3000..=0x3EFF => self.read_byte(address - 0x1000),
            0x3F00..=0x3FFF => match address {
                0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => self.read_byte(address - 0x10),
                0x3F04 | 0x3F08 | 0x3F0C => self.palette[address as usize % 4],
                _ => self.palette[(address as usize - 0x3F00) % 0x20],
            },
            _ => {
                eprintln!("Unexpected VRAM read at {address:X}");
                0
            }
        }
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        match address {
            0x2000..=0x2FFF => {
                let address = address as usize;
                match self.mirroring {
                    MirrorArrangement::Vertical => match address {
                        0x2000..=0x23FF => self.nametable0[address - 0x2000] = data,
                        0x2400..=0x27FF => self.nametable1[address - 0x2400] = data,
                        0x2800..=0x2BFF => self.nametable0[address - 0x2800] = data,
                        0x2C00..=0x2FFF => self.nametable1[address - 0x2C00] = data,
                        _ => unreachable!(),
                    },
                    MirrorArrangement::Horizontal => match address {
                        0x2000..=0x27FF => self.nametable0[address % 0x400] = data,
                        0x2800..=0x2FFF => self.nametable1[address % 0x400] = data,
                        _ => unreachable!(),
                    },
                    MirrorArrangement::OneScreenLower => self.nametable0[address % 0x400] = data,
                    MirrorArrangement::OneScreenUpper => self.nametable1[address % 0x400] = data,
                }
            }
            0x3000..=0x3EFF => self.write_byte(address - 0x1000, data),
            0x3F00..=0x3FFF => match address {
                0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => self.write_byte(address - 0x10, data),
                _ => self.palette[(address as usize - 0x3F00) % 0x20] = data,
            },
            _ => {
                eprintln!("Unexpected VRAM write at {address:X}");
            }
        }
    }
}
