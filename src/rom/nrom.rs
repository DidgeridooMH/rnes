use crate::core::{Addressable, Bus};
use std::{cell::RefCell, rc::Rc};

const PRG_RAM_SIZE: usize = 0x800;
const PRG_ROM_SIZE: usize = 0x4000;
const CHR_ROM_SIZE: usize = 0x2000;

struct NromChr {
    chr_rom: [u8; CHR_ROM_SIZE],
}

pub struct Nrom {
    prg_ram: [u8; PRG_RAM_SIZE],
    prg_rom: [u8; PRG_ROM_SIZE],
}

impl Nrom {
    pub fn register(data: &[u8], bus: &Rc<RefCell<Bus>>, vram_bus: &Rc<RefCell<Bus>>) {
        let rom = Rc::new(RefCell::new(Self {
            prg_ram: [0; PRG_RAM_SIZE],
            prg_rom: data[0..PRG_ROM_SIZE].try_into().unwrap(),
        }));
        bus.borrow_mut().register_region(0x6000..=0xFFFF, rom);

        let chr = Rc::new(RefCell::new(NromChr {
            chr_rom: data[PRG_ROM_SIZE..(PRG_ROM_SIZE + CHR_ROM_SIZE)]
                .try_into()
                .unwrap(),
        }));
        vram_bus.borrow_mut().register_region(0x0..=0x1FFF, chr);
    }
}

impl Addressable for NromChr {
    fn read_byte(&mut self, address: u16) -> u8 {
        self.chr_rom[address as usize]
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        self.chr_rom[address as usize] = data;
    }
}

impl Addressable for Nrom {
    fn read_byte(&mut self, address: u16) -> u8 {
        match address {
            0x6000..=0x7FFF => self.prg_ram[(address as usize - 0x6000) % PRG_RAM_SIZE],
            0x8000..=0xFFFF => self.prg_rom[(address as usize - 0x8000) % PRG_ROM_SIZE],
            _ => {
                eprintln!("(warn) NROM read address 0x{address:X} that was unexpected.");
                0
            }
        }
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        match address {
            0x6000..=0x7FFF => self.prg_ram[(address as usize - 0x6000) % PRG_RAM_SIZE] = data,
            _ => {
                eprintln!("(warn) NROM write address 0x{address:X} that is not writable.");
            }
        }
    }
}
