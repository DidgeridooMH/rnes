use crate::core::Addressable;
use bitfield::bitfield;

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

const MAX_CYCLE: u32 = 340;
const MAX_SCANLINE: u32 = 261;

pub struct PPU {
    t: PPUAddress,
    v: PPUAddress,
    cycle: u32,
    scanline: u32,
    increment_size: u32,
}

impl PPU {
    pub fn new() -> Self {
        Self {
            t: PPUAddress(0),
            v: PPUAddress(0),
            cycle: 0,
            scanline: 0,
            increment_size: 1,
        }
    }

    pub fn tick(&mut self) {
        self.update_address();

        self.cycle = (self.cycle + 1) % (MAX_CYCLE + 1);
        self.scanline = (self.scanline + 1) % (MAX_SCANLINE + 1);
    }

    fn update_address(&mut self) {
        if self.cycle % 8 == 0
            && (0..=255).contains(&self.cycle)
            && (328..=340).contains(&self.cycle)
        {
            self.v.set_coarse_x(self.v.coarse_x() + 1);
        } else if self.cycle == 256 {
            self.v.set_coarse_x(self.v.coarse_x() + 1);
            self.v.set_coarse_y(self.v.coarse_y() + 1);
        } else if self.cycle == 257 {
            self.v.set_coarse_x(self.t.coarse_x());
        } else if (280..=304).contains(&self.cycle) {
            self.v.set_coarse_y(self.t.coarse_y());
        }
    }
}

impl Addressable for PPU {
    fn read_byte(&self, address: u16) -> u8 {}

    fn write_byte(&mut self, address: u16, data: u8) {
        match address {
            0x2002 => {
                // NMI enable/disable - 7
                // master/slave - 6
                // sprite_size - 5
                // background PTA - 4 - 0000/1000
                // sprite PTA - 3
                // VRAM inc - 2
                if data & 4 > 0 {
                    self.increment_size = 32;
                } else {
                    self.increment_size = 1;
                }
                self.t.set_nametable_select((data & 3).into());
            }
        }
    }
}
