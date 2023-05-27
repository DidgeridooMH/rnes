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

use std::time::{Duration, Instant};
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
    cycle_count: usize,
    frame_count_start: Instant,
}

impl Nes {
    pub fn new(rom_file: &str, show_ops: bool, show_header: bool) -> Result<Self, String> {
        let bus = Bus::new();
        let vram_bus = Bus::new();

        let mut cpu = CPU::new(&bus);
        cpu.set_show_ops(show_ops);

        let ppu = Rc::new(RefCell::new(PPU::new(vram_bus.clone())));
        bus.borrow_mut()
            .register_region(0x2000..=0x3FFF, ppu.clone());

        let controller = Rc::new(RefCell::new(Controller::default()));
        bus.borrow_mut()
            .register_region(0x4016..=0x4017, controller);

        let apu = Rc::new(RefCell::new(APU {}));
        bus.borrow_mut()
            .register_region(0x4000..=0x4013, apu.clone());
        bus.borrow_mut().register_region(0x4015..=0x4015, apu);

        let rom_file = match fs::read(rom_file) {
            Ok(f) => f,
            _ => {
                return Err("Unable to read rom file.".into());
            }
        };

        let rom_info = match load_rom(&rom_file, &bus, &vram_bus, show_header) {
            Ok(i) => i,
            Err(e) => {
                return Err(format!("Error while loading rom: {e}"));
            }
        };

        vram_bus.borrow_mut().register_region(
            0x2000..=0x3FFF,
            Rc::new(RefCell::new(VRam::new(rom_info.mirroring))),
        );

        Ok(Self {
            cpu,
            ppu,
            cycle_count: 0,
            frame_count_start: Instant::now(),
        })
    }

    pub fn emulate(&mut self, cycles: usize, screen: &mut [u32]) -> Result<(), String> {
        if self.frame_count_start.elapsed() > Duration::from_secs(1) {
            let frame_count = self.ppu.borrow().frame_count();
            let fps = frame_count as f32 / self.frame_count_start.elapsed().as_secs_f32();
            println!("FPS: {fps} {frame_count} {}", self.cycle_count);
            self.ppu.borrow_mut().reset_frame_count();
            self.frame_count_start = Instant::now();
        }

        let mut used_cycles = 0;
        while used_cycles < cycles {
            match self.cpu.tick() {
                Ok(cycle_count) => {
                    self.cycle_count += cycle_count;
                    used_cycles += cycle_count;
                    for _ in 0..(cycle_count * 3) {
                        if self.ppu.borrow_mut().tick(screen) {
                            self.cpu.generate_nmi();
                        }
                    }
                }
                Err(e) => {
                    self.cpu.dump();
                    println!("PPU Frame Count: {}", self.ppu.borrow().frame_count());
                    return Err(e.to_string());
                }
            }
        }

        Ok(())
    }
}
