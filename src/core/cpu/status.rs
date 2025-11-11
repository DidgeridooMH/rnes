use bitfield::bitfield;

bitfield! {
    #[derive(Copy, Clone, PartialEq)]
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

impl std::fmt::Display for StatusRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut status = String::with_capacity(8);
        status.push((b'N' + !self.n() as u8 * 0x20) as char);
        status.push((b'V' + !self.v() as u8 * 0x20) as char);
        status.push('u');
        status.push((b'B' + !self.b() as u8 * 0x20) as char);
        status.push((b'D' + !self.d() as u8 * 0x20) as char);
        status.push((b'I' + !self.i() as u8 * 0x20) as char);
        status.push((b'Z' + !self.z() as u8 * 0x20) as char);
        status.push((b'C' + !self.c() as u8 * 0x20) as char);
        write!(f, "{status}")
    }
}
