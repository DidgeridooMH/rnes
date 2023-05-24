use crate::core::{Addressable, Bus};
use std::{cell::RefCell, rc::Rc};

mod registers;
use registers::{PPUAddress, PPUControl};

mod vram;
use vram::VRam;

const MAX_CYCLE: u32 = 340;
const MAX_SCANLINE: u32 = 261;

pub struct PPU {
    t: PPUAddress,
    v: PPUAddress,
    w: bool,
    cycle: u32,
    scanline: u32,
    increment_size: u16,
    nmi_enabled: bool,
    vblank: bool,
    fine_x: u8,
    frame_count: u32,
    internal_data_buffer: u8,
    vram_bus: Rc<RefCell<Bus>>,
    open_bus: u8,
}

impl PPU {
    pub fn new(bus: Rc<RefCell<Bus>>) -> Self {
        bus.borrow_mut()
            .register_region(0x2000..=0x3FFF, Rc::new(RefCell::new(VRam::new())));

        Self {
            t: PPUAddress(0),
            v: PPUAddress(0),
            w: false,
            cycle: 0,
            scanline: 0,
            increment_size: 1,
            nmi_enabled: true,
            vblank: false,
            fine_x: 0,
            frame_count: 0,
            internal_data_buffer: 0,
            vram_bus: bus,
            open_bus: 0,
        }
    }

    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }

    pub fn reset_frame_count(&mut self) {
        self.frame_count = 0;
    }

    pub fn tick(&mut self) -> bool {
        let mut generate_nmi = false;
        if self.cycle == 1 && self.scanline == 241 {
            self.vblank = true;
            if self.nmi_enabled {
                generate_nmi = true;
            }
        } else if self.cycle == 1 && self.scanline == MAX_SCANLINE {
            self.vblank = false;
        }

        self.update_address();

        self.increment_cycle();

        generate_nmi
    }

    fn increment_cycle(&mut self) {
        self.cycle += 1;
        if self.cycle > MAX_CYCLE {
            self.scanline += 1;
            if self.scanline > MAX_SCANLINE {
                self.scanline = 0;
                self.frame_count += 1;
            }
            self.cycle = 0;
        }
    }

    fn update_address(&mut self) {
        if self.cycle % 8 == 0
            && (0..=255).contains(&self.cycle)
            && (328..=340).contains(&self.cycle)
        {
            self.increment_x();
        } else if self.cycle == 256 {
            self.increment_x();
            self.increment_y();
        } else if self.cycle == 257 {
            self.v.set_coarse_x(self.t.coarse_x());
            self.v.set_nametable_select(
                (self.v.nametable_select() & 0b10) | (self.t.nametable_select() & 0b01),
            );
        } else if (280..=304).contains(&self.cycle) {
            self.v.set_coarse_y(self.t.coarse_y());
            self.v.set_nametable_select(
                (self.v.nametable_select() & 0b01) | (self.t.nametable_select() & 0b10),
            )
        }
    }

    fn increment_x(&mut self) {
        if self.v.coarse_x() == 0b11111 {
            self.v.set_nametable_select(self.v.nametable_select() ^ 1);
        }
        self.v.set_coarse_x(self.v.coarse_x() + 1);
    }

    fn increment_y(&mut self) {
        if self.v.fine_y() == 0b111 {
            if self.v.coarse_y() == 0b11111 {
                self.v
                    .set_nametable_select(self.v.nametable_select() ^ 0b10);
            }
            self.v.set_coarse_y(self.v.coarse_y() + 1);
        }
        self.v.set_fine_y(self.v.fine_y() + 1);
    }
}

impl Addressable for PPU {
    fn read_byte(&mut self, address: u16) -> u8 {
        self.open_bus = match address {
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 => self.open_bus,
            // TODO: Implement sprite overflow and sprite0 hit.
            0x2002 => {
                self.w = false;
                ((self.vblank as u8) << 7) | 0x40 | (self.open_bus & 0x1F)
            }
            0x2007 => {
                let mut read_byte = self.vram_bus.borrow_mut().read_byte(self.v.0).unwrap();
                if address < 0x3F00 {
                    std::mem::swap(&mut self.internal_data_buffer, &mut read_byte);
                } else {
                    self.internal_data_buffer = self
                        .vram_bus
                        .borrow_mut()
                        .read_byte((self.v.0 % 0x1000) + 0x2000)
                        .unwrap();
                }
                self.v.0 = (self.v.0 + self.increment_size) & 0x7FFF;
                read_byte
            }
            _ => {
                unimplemented!("Reading from VRAM at {address:X}");
            }
        };
        self.open_bus
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        self.open_bus = data;
        match address {
            0x2000 => {
                // TODO: Implement rest of flags.
                let data = PPUControl(data);
                self.nmi_enabled = data.nmi_enable();
                if self.nmi_enabled {
                    println!("NMI Enabled");
                }
                // master/slave - 6
                // sprite_size - 5
                // background PTA - 4 - 0000/1000
                // sprite PTA - 3
                if data.vram_increment() {
                    self.increment_size = 32;
                } else {
                    self.increment_size = 1;
                }
                self.t.set_nametable_select(data.nametable().into());
            }
            0x2001 => {
                // TODO: Implement masking.
            }
            0x2003 => {
                // TODO: Implement OAMADDR
            }
            0x2004 => {
                // TODO: Implement OAMDATA
            }
            0x2005 => {
                if !self.w {
                    self.t.set_coarse_x((data >> 3).into());
                    self.fine_x = data & 0b111;
                    self.w = true;
                } else {
                    self.t.set_fine_y((data & 3).into());
                    self.t.set_coarse_y((data >> 3).into());
                    self.w = false;
                }
            }
            0x2006 => {
                if !self.w {
                    self.t.0 = ((data as u16) << 8) | (self.t.0 & 0xFF);
                    self.w = true;
                } else {
                    self.t.0 = (data as u16) | (self.t.0 & 0xFF00);
                    self.v.0 = self.t.0;
                    self.w = false;
                }
            }
            0x2007 => {
                self.vram_bus
                    .borrow_mut()
                    .write_byte(address, data)
                    .unwrap();
                self.v.0 = (self.v.0 + self.increment_size) & 0x7FFF;
            }
            0x4014 => {
                // TODO: Implement OAM DMA
            }
            _ => {
                unimplemented!("Writing to VRAM at {address:X}");
            }
        }
    }
}
