use crate::core::{cpu::AddressMode, CoreError, CPU};

impl CPU {
    pub fn run_unofficial_op(&mut self, opcode: u8) -> Result<usize, CoreError> {
        let address_mode = AddressMode::from_code(opcode);
        let (operand, page_cross) = match opcode {
            0xAB | 0x83 | 0x87 | 0x8F | 0x97 | 0x93 | 0x9F | 0x9B => (0, false),
            _ => self.read_operand(address_mode),
        };

        let mut use_page_cross = false;

        let base_cycle_cost = match opcode {
            0x4B => self.alr(operand),
            0x0B | 0x2B => self.anc(operand),
            0x6B => self.arr(operand),
            0xCB => self.axs(operand),
            0xA3 | 0xA7 | 0xAF | 0xB3 | 0xB7 | 0xBF => {
                use_page_cross = true;
                self.lax(operand)
            }
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
            0xBB => {
                use_page_cross = true;
                self.las(operand)
            }
            0xEB => {
                self.sbc(operand);
                1
            }
            _ => return Err(CoreError::OpcodeNotImplemented(opcode)),
        };

        self.pc += 1 + address_mode.byte_code_size();

        Ok(base_cycle_cost + address_mode.cycle_cost(!use_page_cross || page_cross))
    }

    fn alr(&mut self, operand: u8) -> usize {
        self.and(operand);
        self.lsr(self.a, AddressMode::Accumulator)
    }

    fn anc(&mut self, operand: u8) -> usize {
        self.and(operand);
        self.p.set_c(self.p.n());
        1
    }

    fn arr(&mut self, operand: u8) -> usize {
        self.and(operand);
        let carry = (operand & 0x20) > 0;
        let overflow = carry ^ ((operand & 0x10) > 0);
        self.ror(self.a, AddressMode::Accumulator);
        self.p.set_c(carry);
        self.p.set_v(overflow);
        1
    }

    fn axs(&mut self, operand: u8) -> usize {
        let (diff, overflow) = (self.a & self.x).overflowing_sub(operand);
        self.x = diff;
        self.set_nz_flags(self.x);
        self.p.set_c(overflow);
        1
    }

    fn lax(&mut self, operand: u8) -> usize {
        self.lda(operand);
        self.tax();
        1
    }

    fn lxa(&mut self, operand: u8) -> usize {
        self.a = operand & 0xFF;
        self.x = self.a;
        1
    }

    fn sax(&mut self, address_mode: AddressMode) -> usize {
        self.write_operand(self.a & self.x, address_mode);
        1
    }

    fn dcp(&mut self, operand: u8, address_mode: AddressMode) -> usize {
        self.dec(operand, address_mode);
        let operand = self.read_operand(address_mode).0;
        self.cmp(operand);
        3
    }

    fn isc(&mut self, operand: u8, address_mode: AddressMode) -> usize {
        self.inc(operand, address_mode);
        let operand = self.read_operand(address_mode).0;
        self.sbc(operand);
        3
    }

    fn rla(&mut self, operand: u8, address_mode: AddressMode) -> usize {
        self.rol(operand, address_mode);
        let operand = self.read_operand(address_mode).0;
        self.and(operand);
        3
    }

    fn rra(&mut self, operand: u8, address_mode: AddressMode) -> usize {
        self.ror(operand, address_mode);
        let operand = self.read_operand(address_mode).0;
        self.adc(operand);
        3
    }

    fn slo(&mut self, operand: u8, address_mode: AddressMode) -> usize {
        self.asl(operand, address_mode);
        let operand = self.read_operand(address_mode).0;
        self.ora(operand);
        3
    }

    fn sre(&mut self, operand: u8, address_mode: AddressMode) -> usize {
        self.lsr(operand, address_mode);
        let operand = self.read_operand(address_mode).0;
        self.eor(operand);
        3
    }

    fn xaa(&mut self, operand: u8) -> usize {
        self.txa();
        self.and(operand);
        1
    }

    fn sha(&mut self, address_mode: AddressMode) -> usize {
        let address = self.get_address(address_mode).0;
        let operand = self.a & self.x & (address >> 8) as u8;
        self.write_operand(operand, address_mode);
        2
    }

    fn tas(&mut self, address_mode: AddressMode) -> usize {
        let address = self.get_address(address_mode).0;
        self.sp = self.a & self.x;
        let operand = self.a & self.x & (address >> 8) as u8;
        self.write_operand(operand, address_mode);
        2
    }

    pub(super) fn shy(&mut self, address_mode: AddressMode) -> usize {
        let address = self.get_address(address_mode).0;
        let operand = self.y & (address >> 8) as u8;
        self.write_operand(operand, address_mode);
        1
    }

    pub(super) fn shx(&mut self, address_mode: AddressMode) -> usize {
        let address = self.get_address(address_mode).0;
        let operand = self.x & (address >> 8) as u8;
        self.write_operand(operand, address_mode);
        1
    }

    fn las(&mut self, operand: u8) -> usize {
        self.a = operand & self.sp;
        self.x = self.a;
        self.sp = self.a;
        1
    }
}
