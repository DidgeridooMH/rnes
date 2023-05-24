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
