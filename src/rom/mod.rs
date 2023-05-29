mod nrom;
pub use nrom::*;

mod mmc1;
pub use mmc1::*;

use crate::core::{Bus, VRam};
use std::{cell::RefCell, rc::Rc};

#[derive(Copy, Clone, Debug)]
pub enum MirrorArrangement {
    OneScreenLower,
    OneScreenUpper,
    Horizontal,
    Vertical,
}

#[derive(Copy, Clone, Debug)]
pub enum Mapper {
    Nrom,
    Mmc1,
    Unsupported,
}

impl Mapper {
    pub fn from_id(id: u8) -> Self {
        match id {
            0 => Mapper::Nrom,
            1 => Mapper::Mmc1,
            _ => Mapper::Unsupported,
        }
    }
}

#[derive(Debug)]
pub struct RomHeader {
    prg: u8,
    chr: u8,
    pub mirroring: MirrorArrangement,
    pub mapper: Mapper,
}

impl RomHeader {
    pub fn from_slice(header: &[u8]) -> Result<Self, String> {
        if header[0] != b'N' && header[1] != b'E' && header[2] != b'S' && header[3] != 0x1A {
            return Err("The ROM does not contain a valid iNES header.".into());
        }

        Ok(Self {
            prg: header[4],
            chr: header[5],
            mirroring: match header[6] & 1 > 0 {
                true => MirrorArrangement::Vertical,
                false => MirrorArrangement::Horizontal,
            },
            mapper: Mapper::from_id((header[6] >> 4) | (header[7] & 0xF0u8)),
        })
    }
}

pub fn load_rom(
    rom: &[u8],
    bus: &Rc<RefCell<Bus>>,
    vram_bus: &Rc<RefCell<Bus>>,
    show_header: bool,
    vram: &Rc<RefCell<VRam>>,
) -> Result<(), String> {
    let header = match RomHeader::from_slice(&rom[0..16]) {
        Ok(h) => h,
        Err(e) => return Err(e),
    };

    vram.borrow_mut().set_mirroring(header.mirroring);

    if show_header {
        println!("{header:?}");
    }

    match header.mapper {
        Mapper::Nrom => Nrom::register(&rom[16..], header.prg, header.chr, bus, vram_bus),
        Mapper::Mmc1 => Mmc1::register(&rom[16..], header.prg, header.chr, bus, vram_bus, vram),
        _ => return Err("Unsupported mapper".into()),
    }

    Ok(())
}
