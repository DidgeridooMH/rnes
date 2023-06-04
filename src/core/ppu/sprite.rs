use super::PPU;

#[derive(Default, Copy, Clone, Debug)]
pub struct SpriteShift {
    pattern_low: u8,
    pattern_high: u8,
    attribute: u8,
}

impl SpriteShift {
    pub fn get_pixel_color_index(&self, fine_x: u8) -> u8 {
        let bit_select = match (self.attribute & (1 << 6)) > 0 {
            true => 1 << fine_x,
            false => 0x80 >> fine_x,
        };

        let low_bit = ((self.pattern_low & bit_select) > 0) as u8;
        let high_bit = ((self.pattern_high & bit_select) > 0) as u8;
        let attribute = self.attribute & 3;

        (attribute << 2) | (high_bit << 1) | low_bit
    }
}

impl PPU {
    pub fn evaluate_sprites(&mut self) {
        let sprite_size = if self.sprite_size { 16 } else { 8 };
        for (i, entry) in self.primary_oam.iter().enumerate() {
            let scanline = self.scanline as u8;
            if scanline >= entry.y && scanline <= entry.y + (sprite_size - 1) {
                match self.secondary_oam.iter().position(|&e| e.is_none()) {
                    Some(index) => self.secondary_oam[index] = Some((*entry, i)),
                    None => {
                        if self.mask.show_sprite() || self.mask.show_background() {
                            self.sprite_overflow = true;
                        }
                    }
                }
            }
        }
    }

    pub fn load_sprite_shifts(&mut self) {
        let sprite_size = if self.sprite_size { 16 } else { 8 };

        for i in 0..self.secondary_oam.len() {
            if let Some((entry, _)) = self.secondary_oam[i] {
                let y = if entry.attributes & 0x80 > 0 {
                    (sprite_size - 1) - (self.scanline as u16 - entry.y as u16)
                } else {
                    self.scanline as u16 - entry.y as u16
                };

                let mut vram_bus = self.vram_bus.borrow_mut();

                let tile_index = if y < 8 {
                    if self.sprite_size {
                        entry.tile_index & 0xFE
                    } else {
                        entry.tile_index
                    }
                } else {
                    (entry.tile_index & 0xFE) + 1
                } as u16;

                let sprite_table = if self.sprite_size {
                    (entry.tile_index & 1) as u16 * 0x1000
                } else {
                    self.sprite_table
                };
                self.secondary_shifters[i] = SpriteShift {
                    pattern_low: vram_bus
                        .read_byte(sprite_table + tile_index * 16 + (y % 8))
                        .unwrap(),
                    pattern_high: vram_bus
                        .read_byte(sprite_table + tile_index * 16 + (y % 8) + 8)
                        .unwrap(),
                    attribute: entry.attributes,
                }
            }
        }
    }

    pub fn get_sprite_pixel(&self) -> (u8, usize, bool) {
        if !self.mask.show_sprite_left() && self.cycle <= 8 {
            return (0, 0, false);
        }

        let x = (self.cycle - 1) as u8;
        let mut pattern: Option<(u8, usize, bool)> = None;
        for i in 0..self.current_oam.len() {
            if let Some((entry, index)) = self.current_oam[i] {
                if x >= entry.x && x <= entry.x.wrapping_add(7) {
                    let color_index = self.secondary_shifters[i].get_pixel_color_index(x - entry.x);
                    if (color_index & 3) > 0 && pattern.is_none() {
                        pattern = Some((color_index, index, (entry.attributes & (1 << 5)) > 0));
                    }
                }
            }
        }
        pattern.unwrap_or((0, 0, false))
    }
}
