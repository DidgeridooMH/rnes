use bitfield::bitfield;

bitfield! {
    pub struct StatusRegister(u8);
    impl Debug;
    #[inline]
    pub c, set_c: 0;
    #[inline]
    pub z, set_z: 1;
    #[inline]
    pub i, set_i: 2;
    #[inline]
    pub d, set_d: 3;
    #[inline]
    pub b, set_b: 5, 4;
    #[inline]
    pub v, set_v: 6;
    #[inline]
    pub n, set_n: 7;
}
