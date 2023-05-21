mod apu;
mod bus;
mod controller;
mod cpu;
mod ppu;

pub use apu::*;
pub use bus::*;
pub use controller::*;
pub use cpu::*;
pub use ppu::*;

use crate::rom::load_rom;
use std::{cell::RefCell, fmt, fs, rc::Rc};

#[derive(Debug, Clone, PartialEq)]
pub enum CoreError {
    AddressDecode(u8),
    OpcodeNotImplemented(u8),
    InvalidRegion(u16),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            CoreError::InvalidRegion(address) => {
                write!(f, "Address access violation at 0x{:04X}", address)
            }
            CoreError::AddressDecode(opcode) => {
                write!(f, "Unknown address mode from 0x{:02X}", opcode)
            }
            CoreError::OpcodeNotImplemented(opcode) => {
                write!(f, "Opcode not implemented: 0x{0:02X}", opcode)
            }
        }
    }
}

pub struct Nes {
    cpu: CPU,
    ppu: Rc<RefCell<PPU>>,
}

impl Nes {
    pub fn new(rom_file: &str, show_ops: bool) -> Result<Self, String> {
        let bus = Bus::new();
        let mut cpu = CPU::new(&bus);
        cpu.set_show_ops(show_ops);

        let ppu = Rc::new(RefCell::new(PPU::new()));
        bus.borrow_mut()
            .register_region(0x2000..=0x2007, ppu.clone());
        bus.borrow_mut()
            .register_region(0x4014..=0x4014, ppu.clone());

        let controller = Rc::new(RefCell::new(Controller::default()));
        bus.borrow_mut()
            .register_region(0x4016..=0x4017, controller);

        let apu = Rc::new(RefCell::new(APU {}));
        bus.borrow_mut()
            .register_region(0x4000..=0x4013, apu.clone());
        bus.borrow_mut()
            .register_region(0x4015..=0x4015, apu.clone());
        bus.borrow_mut().register_region(0x4017..=0x4017, apu);

        let rom_file = match fs::read(rom_file) {
            Ok(f) => f,
            _ => {
                return Err("Unable to read rom file.".into());
            }
        };

        if let Err(e) = load_rom(&rom_file, &bus) {
            return Err(format!("Error while loading rom: {e}"));
        }

        Ok(Self { cpu, ppu })
    }

    pub fn emulate(&mut self) -> Result<(), String> {
        match self.cpu.tick() {
            Ok(cycle_count) => {
                for _ in 0..(cycle_count * 3) {
                    if self.ppu.borrow_mut().tick() {
                        self.cpu.generate_nmi();
                    }
                }
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }
}
