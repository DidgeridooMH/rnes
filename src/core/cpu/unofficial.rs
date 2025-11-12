use crate::core::{cpu::AddressMode, CoreError, CPU};

impl CPU {
    pub fn run_unofficial_op(&mut self, opcode: u8) -> Result<usize, CoreError> {
        let address_mode = AddressMode::from_code(opcode);
        let operand = self.read_operand(address_mode).0;

        match opcode {
            0x4B => self.alr(operand),
            0x0B | 0x2B => self.anc(operand),
            0x6B => self.arr(operand),
            0xCB => self.axs(operand),
            0xA3 | 0xA7 | 0xAF | 0xB3 | 0xB7 | 0xBF => self.lax(operand),
            0xAB => self.lxa(operand),
            0x83 | 0x87 | 0x8F | 0x97 => self.sax(address_mode),
            0xC3 | 0xC7 | 0xCF | 0xD3 | 0xD7 | 0xDB | 0xDF => self.dcp(operand, address_mode),
            0xE3 | 0xE7 | 0xEF | 0xF3 | 0xF7 | 0xFB | 0xFF => self.isc(operand, address_mode),
            0x23 | 0x27 | 0x2F | 0x33 | 0x37 | 0x3B | 0x3F => self.rla(operand, address_mode),
            0x63 | 0x67 | 0x6F | 0x73 | 0x77 | 0x7B | 0x7F => self.rra(operand, address_mode),
            0x03 | 0x07 | 0x0F | 0x13 | 0x17 | 0x1B | 0x1F => self.slo(operand, address_mode),
            0x43 | 0x47 | 0x4F | 0x53 | 0x57 | 0x5B | 0x5F => self.sre(operand, address_mode),
            0x8B => self.xaa(operand),
            0x93 | 0x9F => self.sha(address_mode),
            0x9B => self.tas(address_mode),
            0xBB => self.las(operand),
            0xEB => self.sbc(operand),
            _ => return Err(CoreError::OpcodeNotImplemented(opcode)),
        };

        self.pc += 1 + address_mode.byte_code_size();

        Ok(1 + address_mode.cycle_cost())
    }

    fn alr(&mut self, operand: u8) {
        self.and(operand);
        self.lsr(self.a, AddressMode::Accumulator);
    }

    fn anc(&mut self, operand: u8) {
        self.and(operand);
        self.p.set_c(self.p.n());
    }

    fn arr(&mut self, operand: u8) {
        self.and(operand);
        let carry = (operand & 0x20) > 0;
        let overflow = carry ^ ((operand & 0x10) > 0);
        self.ror(self.a, AddressMode::Accumulator);
        self.p.set_c(carry);
        self.p.set_v(overflow);
    }

    fn axs(&mut self, operand: u8) {
        let (diff, overflow) = (self.a & self.x).overflowing_sub(operand);
        self.x = diff;
        self.set_nz_flags(self.x);
        self.p.set_c(overflow);
    }

    fn lax(&mut self, operand: u8) {
        self.lda(operand);
        self.tax();
    }

    fn lxa(&mut self, operand: u8) {
        self.a = operand & 0xFF;
        self.x = self.a;
    }

    fn sax(&mut self, address_mode: AddressMode) {
        self.write_operand(self.a & self.x, address_mode);
    }

    fn dcp(&mut self, operand: u8, address_mode: AddressMode) {
        self.dec(operand, address_mode);
        let operand = self.read_operand(address_mode).0;
        self.cmp(operand);
    }

    fn isc(&mut self, operand: u8, address_mode: AddressMode) {
        self.inc(operand, address_mode);
        let operand = self.read_operand(address_mode).0;
        self.sbc(operand);
    }

    fn rla(&mut self, operand: u8, address_mode: AddressMode) {
        self.rol(operand, address_mode);
        let operand = self.read_operand(address_mode).0;
        self.and(operand);
    }

    fn rra(&mut self, operand: u8, address_mode: AddressMode) {
        self.ror(operand, address_mode);
        let operand = self.read_operand(address_mode).0;
        self.adc(operand);
    }

    fn slo(&mut self, operand: u8, address_mode: AddressMode) {
        self.asl(operand, address_mode);
        let operand = self.read_operand(address_mode).0;
        self.ora(operand);
    }

    fn sre(&mut self, operand: u8, address_mode: AddressMode) {
        self.lsr(operand, address_mode);
        let operand = self.read_operand(address_mode).0;
        self.eor(operand);
    }

    fn xaa(&mut self, operand: u8) {
        self.txa();
        self.and(operand);
    }

    fn sha(&mut self, address_mode: AddressMode) {
        let address = self.get_address(address_mode).0;
        let operand = self.a & self.x & (address >> 8) as u8;
        self.write_operand(operand, address_mode);
    }

    fn tas(&mut self, address_mode: AddressMode) {
        let address = self.get_address(address_mode).0;
        self.sp = self.a & self.x;
        let operand = self.a & self.x & (address >> 8) as u8;
        self.write_operand(operand, address_mode);
    }

    pub(super) fn shy(&mut self, address_mode: AddressMode) {
        let address = self.get_address(address_mode).0;
        let operand = self.y & (address >> 8) as u8;
        self.write_operand(operand, address_mode);
    }

    pub(super) fn shx(&mut self, address_mode: AddressMode) {
        let address = self.get_address(address_mode).0;
        let operand = self.x & (address >> 8) as u8;
        self.write_operand(operand, address_mode);
    }

    fn las(&mut self, operand: u8) {
        self.a = operand & self.sp;
        self.x = self.a;
        self.sp = self.a;
    }
}
