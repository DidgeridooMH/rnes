use crate::core::{Addressable, PPU};

use bitfield::bitfield;

const PPUCTRL: u16 = 0x2000;
const PPUMASK: u16 = 0x2001;
const PPUSTATUS: u16 = 0x2002;
const OAMADDR: u16 = 0x2003;
const OAMDATA: u16 = 0x2004;
const PPUSCROLL: u16 = 0x2005;
const PPUADDR: u16 = 0x2006;
const PPUDATA: u16 = 0x2007;

bitfield! {
    #[derive(Copy, Clone, PartialEq)]
    pub struct PPUAddress(u16);
    impl Debug;
    #[inline]
    pub coarse_x, set_coarse_x: 4, 0;
    #[inline]
    pub coarse_y, set_coarse_y: 9, 5;
    #[inline]
    pub nametable_select, set_nametable_select: 11, 10;
    #[inline]
    pub fine_y, set_fine_y: 14, 12;
}

bitfield! {
    #[derive(Copy, Clone, PartialEq)]
    pub struct PPUControl(u8);
    impl Debug;
    #[inline]
    pub nametable, _: 1, 0;
    #[inline]
    pub vram_increment, _: 2;
    #[inline]
    pub sprite_pattern, _: 3;
    #[inline]
    pub background_pattern, _: 4;
    #[inline]
    pub sprite_size, _: 5;
    #[inline]
    pub master_slave, _: 6;
    #[inline]
    pub nmi_enable, _: 7;
}

bitfield! {
    #[derive(Copy, Clone, PartialEq)]
    pub struct PPUMask(u8);
    impl Debug;
    #[inline]
    pub greyscale, _: 0;
    #[inline]
    pub show_background_left, _: 1;
    #[inline]
    pub show_sprite_left, _: 2;
    #[inline]
    pub show_background, _: 3;
    #[inline]
    pub show_sprite, _: 4;
    #[inline]
    pub emphasize_red, _: 5;
    #[inline]
    pub emphasize_green, _: 6;
    #[inline]
    pub emphasize_blue, _: 7;
}

impl Addressable for PPU {
    fn read_byte(&mut self, address: u16) -> u8 {
        let address = address % 8 + 0x2000;

        self.open_bus = match address {
            PPUCTRL | PPUMASK | OAMADDR | PPUSCROLL | PPUADDR => self.open_bus,
            PPUSTATUS => {
                self.w = false;
                let result = ((self.vblank as u8) << 7)
                    | ((self.sprite0_hit as u8) << 6)
                    | ((self.sprite_overflow as u8) << 5)
                    | (self.open_bus & 0x1F);
                self.vblank = false;
                result
            }
            PPUDATA => {
                let mut read_byte = self.vram_bus.borrow_mut().read_byte(self.v.0).unwrap();
                if self.v.0 < 0x3F00 {
                    std::mem::swap(&mut self.internal_data_buffer, &mut read_byte);
                } else {
                    self.internal_data_buffer = self
                        .vram_bus
                        .borrow_mut()
                        .read_byte(self.v.0 - 0x1000)
                        .unwrap();
                }
                self.v.0 = (self.v.0 + self.increment_size) & 0x3FFF;
                read_byte
            }
            OAMDATA => match self.oam_address % 4 {
                0 => self.primary_oam[self.oam_address as usize / 4].y,
                1 => self.primary_oam[self.oam_address as usize / 4].tile_index,
                2 => self.primary_oam[self.oam_address as usize / 4].attributes,
                3 => self.primary_oam[self.oam_address as usize / 4].x,
                _ => unreachable!(),
            },
            _ => {
                unimplemented!("Reading from VRAM at {address:X}");
            }
        };
        self.open_bus
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        let address = address % 8 + 0x2000;

        self.open_bus = data;

        if self.reset
            && (address == PPUCTRL
                || address == PPUMASK
                || address == PPUSCROLL
                || address == PPUADDR)
        {
            return;
        }

        match address {
            PPUCTRL => {
                if self.reset {
                    return;
                }

                let data = PPUControl(data);
                self.nmi_enabled = data.nmi_enable();
                // master/slave - 6
                // sprite_size - 5
                self.sprite_size = data.sprite_size();
                self.background_table = data.background_pattern() as u16 * 0x1000;
                self.sprite_table = data.sprite_pattern() as u16 * 0x1000;
                if data.vram_increment() {
                    self.increment_size = 32;
                } else {
                    self.increment_size = 1;
                }
                self.t.set_nametable_select(data.nametable().into());
            }
            PPUMASK => {
                self.mask.0 = data;
            }
            OAMADDR => {
                self.oam_address = data;
            }
            OAMDATA => {
                match self.oam_address % 4 {
                    0 => self.primary_oam[self.oam_address as usize / 4].y = data,
                    1 => self.primary_oam[self.oam_address as usize / 4].tile_index = data,
                    2 => self.primary_oam[self.oam_address as usize / 4].attributes = data,
                    3 => self.primary_oam[self.oam_address as usize / 4].x = data,
                    _ => unreachable!(),
                }
                self.oam_address = self.oam_address.wrapping_add(1);
            }
            PPUSCROLL => {
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
            PPUADDR => {
                if !self.w {
                    self.t.0 = ((data as u16) << 8) | (self.t.0 & 0xFF);
                    self.w = true;
                } else {
                    self.t.0 = (data as u16) | (self.t.0 & 0xFF00);
                    self.v.0 = self.t.0;
                    self.w = false;
                }
            }
            PPUDATA => {
                self.vram_bus
                    .borrow_mut()
                    .write_byte(self.v.0, data)
                    .unwrap();
                self.v.0 = (self.v.0 + self.increment_size) & 0x3FFF;
            }
            _ => {
                println!("(warn) Wrote to read only port {address:X}");
            }
        }
    }
}
