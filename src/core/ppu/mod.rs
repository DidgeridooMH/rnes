use crate::core::Bus;
use crate::window::NATIVE_RESOLUTION;
use std::{cell::RefCell, rc::Rc};

mod registers;
use registers::{PPUAddress, PPUMask};

mod palette;

mod vram;
pub use vram::VRam;

use self::oam::OamEntry;

mod oam;

const NAMETABLE_BASE_ADDR: u16 = 0x2000;

const MAX_CYCLE: u32 = 340;
const MAX_SCANLINE: u32 = 261;

#[derive(Default, Copy, Clone, Debug)]
struct PPUShift {
    pattern: [u16; 2],
    attribute: u32,
}

impl PPUShift {
    pub fn load_pattern_low(&mut self, data: u8) {
        self.pattern[0] = (self.pattern[0] & 0xFF00) | data as u16;
    }

    pub fn load_pattern_high(&mut self, data: u8) {
        self.pattern[1] = (self.pattern[1] & 0xFF00) | data as u16;
    }

    pub fn load_attribute(&mut self, data: u8) {
        for i in 0..8 {
            self.attribute |= (data as u32) << (i * 2);
        }
    }

    pub fn get_pixel_color_index(&self, fine_x: u8) -> u8 {
        let low_bit = ((self.pattern[0] & (0x8000 >> fine_x)) > 0) as u8;
        let high_bit = ((self.pattern[1] & (0x8000 >> fine_x)) > 0) as u8;
        let attribute = (self.attribute >> 30) as u8;

        (attribute << 2) | (high_bit << 1) | low_bit
    }

    pub fn shift(&mut self) {
        self.pattern[0] <<= 1;
        self.pattern[1] <<= 1;
        self.attribute <<= 2;
    }
}

pub struct PPU {
    t: PPUAddress,
    v: PPUAddress,
    w: bool,
    cycle: u32,
    scanline: u32,
    increment_size: u16,
    nmi_enabled: bool,
    vblank: bool,
    reset: bool,
    fine_x: u8,
    frame_count: u32,
    internal_data_buffer: u8,
    vram_bus: Rc<RefCell<Bus>>,
    open_bus: u8,
    odd_frame: bool,
    mask: PPUMask,
    shifter: PPUShift,
    background_table: u16,
    sprite_table: u16,
    name_table_selector: u8,
    pattern_low: u8,
    pattern_high: u8,
    attribute: u8,
    internal_screen: Vec<u32>,
    oam_address: u8,
    primary_oam: [OamEntry; 64],
    secondary_oam: [OamEntry; 8],
}

impl PPU {
    pub fn new(bus: Rc<RefCell<Bus>>) -> Self {
        Self {
            t: PPUAddress(0),
            v: PPUAddress(0),
            w: false,
            cycle: 0,
            scanline: 261,
            increment_size: 1,
            nmi_enabled: false,
            vblank: true,
            reset: true,
            fine_x: 0,
            frame_count: 0,
            internal_data_buffer: 0,
            vram_bus: bus,
            open_bus: 0,
            odd_frame: false,
            mask: PPUMask(0),
            shifter: PPUShift::default(),
            sprite_table: 0,
            background_table: 0,
            name_table_selector: 0,
            pattern_low: 0,
            pattern_high: 0,
            attribute: 0,
            internal_screen: vec![
                0u32;
                NATIVE_RESOLUTION.width as usize
                    * NATIVE_RESOLUTION.height as usize
            ],
            oam_address: 0,
            primary_oam: [OamEntry::default(); 64],
            secondary_oam: [OamEntry::default(); 8],
        }
    }

    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }

    pub fn reset_frame_count(&mut self) {
        self.frame_count = 0;
    }

    pub fn blit(&self, display_screen: &mut [u32]) {
        display_screen[..self.internal_screen.len()].copy_from_slice(&self.internal_screen[..]);
    }

    pub fn tick(&mut self) -> bool {
        let mut generate_nmi = false;

        if self.odd_frame && self.cycle == 0 && self.scanline == 0 {
            self.cycle = 1;
        }

        if self.cycle == 1 && self.scanline == 241 {
            self.vblank = true;
            if self.nmi_enabled {
                generate_nmi = true;
            }
        } else if self.cycle == 1 && self.scanline == MAX_SCANLINE {
            self.vblank = false;
            self.reset = false;
        }

        let fetching_cycle = (1..=257).contains(&self.cycle) || (321..=336).contains(&self.cycle);
        let visible_scanline = (0..=239).contains(&self.scanline) || self.scanline == 261;

        if self.mask.show_background() || self.mask.show_sprite() {
            if fetching_cycle {
                match self.cycle % 8 {
                    1 => {
                        self.secondary_oam = [OamEntry::default(); 8];

                        self.shifter.load_pattern_low(self.pattern_low);
                        self.shifter.load_pattern_high(self.pattern_high);
                        self.shifter.load_attribute(self.attribute);

                        self.name_table_selector = self
                            .vram_bus
                            .borrow_mut()
                            .read_byte((self.v.0 & 0xFFF) + NAMETABLE_BASE_ADDR)
                            .unwrap();
                    }
                    3 => {
                        let x = self.v.coarse_x() / 4;
                        let y = self.v.coarse_y() / 4;
                        let nametable = self.v.nametable_select();
                        let attribute_address = 0x23C0 | (nametable * 0x400) | (y << 3) | x;
                        self.attribute = self
                            .vram_bus
                            .borrow_mut()
                            .read_byte(attribute_address)
                            .unwrap();
                        self.attribute >>=
                            (self.v.coarse_x() & 0b10) | ((self.v.coarse_y() & 0b10) << 1);
                        self.attribute &= 0b11;
                    }
                    5 => {
                        self.pattern_low = self
                            .vram_bus
                            .borrow_mut()
                            .read_byte(
                                self.background_table
                                    + self.name_table_selector as u16 * 16
                                    + self.v.fine_y(),
                            )
                            .unwrap();
                    }
                    7 => {
                        self.pattern_high = self
                            .vram_bus
                            .borrow_mut()
                            .read_byte(
                                self.background_table
                                    + self.name_table_selector as u16 * 16
                                    + self.v.fine_y()
                                    + 8,
                            )
                            .unwrap();
                    }
                    _ => {}
                }
            }

            if (1..=256).contains(&self.cycle) && self.scanline < 240 {
                let color_index = self.shifter.get_pixel_color_index(self.fine_x);
                let color = self
                    .vram_bus
                    .borrow_mut()
                    .read_byte(0x3F00 + color_index as u16)
                    .unwrap();
                self.internal_screen[self.cycle as usize - 1
                    + self.scanline as usize * NATIVE_RESOLUTION.width as usize] =
                    palette::PALETTE[color as usize];
            }

            if fetching_cycle {
                self.shifter.shift();
            }

            if visible_scanline {
                self.update_address();
            }
        }

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
                self.odd_frame = !self.odd_frame;
            }
            self.cycle = 0;
        }
    }

    fn update_address(&mut self) {
        if self.cycle % 8 == 0
            && ((1..=255).contains(&self.cycle) || (328..=336).contains(&self.cycle))
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
        } else if (280..=304).contains(&self.cycle) && self.scanline == 261 {
            self.v.set_fine_y(self.t.fine_y());
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
