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

use crate::window::NATIVE_RESOLUTION;

use crate::{
    rom::load_rom,
    window::screen::{Pixel, ScreenBuffer},
};
use std::time::{Duration, Instant};
use std::{
    cell::RefCell,
    fmt, fs,
    rc::Rc,
    sync::{Arc, Mutex},
};

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
    vram: Rc<RefCell<Bus>>,
    cycle_count: usize,
    frame_count_start: Instant,
}

impl Nes {
    pub fn new(rom_file: &str, show_ops: bool) -> Result<Self, String> {
        let bus = Bus::new();
        let vram_bus = Bus::new();

        let mut cpu = CPU::new(&bus);
        cpu.set_show_ops(show_ops);

        let ppu = Rc::new(RefCell::new(PPU::new(vram_bus.clone())));
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

        if let Err(e) = load_rom(&rom_file, &bus, &vram_bus) {
            return Err(format!("Error while loading rom: {e}"));
        }

        Ok(Self {
            cpu,
            ppu,
            cycle_count: 0,
            vram: vram_bus,
            frame_count_start: Instant::now(),
        })
    }

    fn render_pattern_table(&mut self, screen: &Arc<Mutex<Box<ScreenBuffer>>>) {
        let mut screen = screen.lock().unwrap();
        for n in 0..0x3C0 {
            let coarse_x = (n % 32) * 8;
            let coarse_y = (n / 32) * 8 * NATIVE_RESOLUTION.width as usize;
            let tile_address = coarse_x + coarse_y;

            let pattern_address =
                0x1000 + self.vram.borrow_mut().read_byte(0x2000 + n as u16).unwrap() as u16 * 16;

            for y in 0..8 {
                let mut pl_low = self
                    .vram
                    .borrow_mut()
                    .read_byte(pattern_address + y)
                    .unwrap();
                let mut pl_high = self
                    .vram
                    .borrow_mut()
                    .read_byte(pattern_address + y + 8)
                    .unwrap();
                for x in 0..8 {
                    let color_index = pl_high >> 6 | pl_low >> 7;
                    let color = match color_index {
                        3 => Pixel {
                            r: 255,
                            g: 0,
                            b: 0,
                            a: 255,
                        },
                        2 => Pixel {
                            r: 0,
                            g: 255,
                            b: 0,
                            a: 255,
                        },
                        1 => Pixel {
                            r: 0,
                            g: 0,
                            b: 255,
                            a: 255,
                        },
                        _ => Pixel {
                            r: 0,
                            g: 0,
                            b: 0,
                            a: 0,
                        },
                    };
                    screen.buffer
                        [tile_address + x + y as usize * NATIVE_RESOLUTION.width as usize] = color;
                    pl_low <<= 1;
                    pl_high <<= 1;
                }
            }
        }
    }

    pub fn emulate(&mut self, screen: &Arc<Mutex<Box<ScreenBuffer>>>) -> Result<(), String> {
        if self.frame_count_start.elapsed() > Duration::from_secs(2) {
            let frame_count = self.ppu.borrow().frame_count();
            let fps = frame_count as f32 / self.frame_count_start.elapsed().as_secs_f32();
            println!("FPS: {fps} {} {}", frame_count, self.cycle_count);
            self.ppu.borrow_mut().reset_frame_count();
            self.frame_count_start = Instant::now();
        }

        match self.cpu.tick() {
            Ok(cycle_count) => {
                self.cycle_count += cycle_count;
                for _ in 0..(cycle_count * 3) {
                    if self.ppu.borrow_mut().tick() {
                        self.cpu.generate_nmi();
                        self.render_pattern_table(screen);
                    }
                }
                Ok(())
            }
            Err(e) => {
                self.cpu.dump();
                println!("PPU Frame Count: {}", self.ppu.borrow().frame_count());
                Err(e.to_string())
            }
        }
    }
}
