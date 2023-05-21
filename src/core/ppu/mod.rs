use crate::core::Addressable;
use bitfield::bitfield;

mod registers;

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
    struct PPUControl(u8);
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

const MAX_CYCLE: u32 = 340;
const MAX_SCANLINE: u32 = 261;

pub struct PPU {
    t: PPUAddress,
    v: PPUAddress,
    cycle: u32,
    scanline: u32,
    increment_size: u32,
    nmi_enabled: bool,
    vblank: bool,
}

impl Default for PPU {
    fn default() -> Self {
        Self::new()
    }
}

impl PPU {
    pub fn new() -> Self {
        Self {
            t: PPUAddress(0),
            v: PPUAddress(0),
            cycle: 0,
            scanline: 0,
            increment_size: 1,
            nmi_enabled: true,
            vblank: false,
        }
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
            }
            self.cycle = 0;
        }
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
    fn read_byte(&self, address: u16) -> u8 {
        // TODO: Implement stale data.
        match address {
            // TODO: Implement sprite overflow and sprite0 hit.
            0x2002 => (self.vblank as u8) << 7,
            _ => {
                unimplemented!("Reading from VRAM at {address:X}");
            }
        }
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        match address {
            0x2000 => {
                // TODO: Implement rest of flags.
                let data = PPUControl(data);
                self.nmi_enabled = data.nmi_enable();
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
            _ => {
                unimplemented!("Writing to VRAM at {address:X}");
            }
        }
    }
}
