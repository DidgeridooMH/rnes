use bitfield::bitfield;

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
