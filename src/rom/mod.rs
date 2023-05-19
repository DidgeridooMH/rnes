mod nrom;
pub use nrom::*;

use crate::core::Bus;
use std::{cell::RefCell, rc::Rc};

/*#[derive(Debug)]
enum MirrorArrangement {
    Horizontal,
    Vertical,
}*/

#[derive(Debug)]
pub enum Mapper {
    Nrom,
    Unsupported,
}

impl Mapper {
    pub fn from_id(id: u8) -> Self {
        match id {
            0 => Mapper::Nrom,
            _ => Mapper::Unsupported,
        }
    }
}

#[derive(Debug)]
pub struct RomHeader {
    //prg: u8,
    //chr: u8,
    //battery: bool,
    //mirroring: MirrorArrangement,
    pub mapper: Mapper,
}

impl RomHeader {
    pub fn from_slice(header: &[u8]) -> Result<Self, String> {
        if header[0] != b'N' && header[1] != b'E' && header[2] != b'S' && header[3] != 0x1A {
            return Err("The ROM does not contain a valid iNES header.".into());
        }

        Ok(Self {
            //prg: header[4],
            //chr: header[5],
            //battery: header[6] & 2 > 0,
            /*mirroring: match header[6] & 1 > 0 {
                true => MirrorArrangement::Horizontal,
                false => MirrorArrangement::Vertical,
            },*/
            mapper: Mapper::from_id((header[6] >> 4) & (header[7] & 0xF0u8)),
        })
    }
}

pub fn load_rom(rom: &[u8], bus: &Rc<RefCell<Bus>>) -> Result<(), String> {
    let header = match RomHeader::from_slice(&rom[0..16]) {
        Ok(h) => h,
        Err(e) => return Err(e),
    };

    match header.mapper {
        Mapper::Nrom => Nrom::register(&rom[16..], bus),
        _ => return Err("Unsupported mapper".into()),
    }

    Ok(())
}