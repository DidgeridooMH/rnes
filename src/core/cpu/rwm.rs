use super::{AddressMode, CoreError, CPU};

// #[cfg(test)]
// mod tests;

impl CPU {
    pub fn run_rwm_op(&mut self, opcode: u8) -> Result<usize, CoreError> {
        let address_mode = AddressMode::from_code(opcode);

        let operand = match opcode {
            0x86 | 0x96 | 0x8E | 0x8A | 0x9A | 0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE | 0xAA | 0xBA
            | 0xCA | 0x1A | 0x3A | 0x5A | 0x7A | 0x82 | 0xC2 | 0xDA | 0xE2 | 0xEA | 0xFA | 0x9E => {
                None
            }
            _ => Some(self.read_operand(address_mode).0),
        };

        let cycles = match opcode {
            0x06 | 0x0A | 0x0E | 0x16 | 0x1E => self.asl(operand.unwrap(), address_mode),
            0x2A | 0x26 | 0x36 | 0x2E | 0x3E => self.rol(operand.unwrap(), address_mode),
            0x4A | 0x46 | 0x56 | 0x4E | 0x5E => self.lsr(operand.unwrap(), address_mode),
            0x6A | 0x66 | 0x76 | 0x6E | 0x7E => self.ror(operand.unwrap(), address_mode),
            0x86 | 0x96 | 0x8E => self.stx(address_mode),
            0x8A => self.txa(),
            0x9A => self.txs(),
            0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => self.ldx(address_mode),
            0xAA => self.tax(),
            0xBA => self.tsx(),
            0xC6 | 0xD6 | 0xCE | 0xDE => self.dec(operand.unwrap(), address_mode),
            0xCA => self.dex(),
            0xE6 | 0xF6 | 0xEE | 0xFE => self.inc(operand.unwrap(), address_mode),
            0x1A | 0x3A | 0x5A | 0x7A | 0x82 | 0xC2 | 0xDA | 0xE2 | 0xEA | 0xFA => {
                self.nop(address_mode)
            }
            0x9E => {
                self.shx(address_mode);
                2
            }
            _ => return Err(CoreError::OpcodeNotImplemented(opcode)),
        };

        self.pc += 1 + address_mode.byte_code_size();

        Ok(cycles + address_mode.cycle_cost(true))
    }

    fn get_common_rwm_cycles(&self, address_mode: AddressMode) -> usize {
        match address_mode {
            AddressMode::Accumulator | AddressMode::Immediate => 1,
            AddressMode::ZeroPage
            | AddressMode::ZeroPageX
            | AddressMode::ZeroPageY
            | AddressMode::Absolute => 3,
            AddressMode::AbsoluteX | AddressMode::AbsoluteY => 4,
            AddressMode::Indirect | AddressMode::IndirectX | AddressMode::IndirectY => 4,
            _ => 3,
        }
    }

    pub(super) fn asl(&mut self, operand: u8, address_mode: AddressMode) -> usize {
        self.p.set_c(operand & 0x80 > 0);
        let operand = operand << 1;
        self.write_operand(operand, address_mode);
        self.set_nz_flags(operand);
        self.get_common_rwm_cycles(address_mode)
    }

    pub(super) fn rol(&mut self, operand: u8, address_mode: AddressMode) -> usize {
        let carry = self.p.c() as u8;
        self.p.set_c(operand & 0x80 > 0);
        let operand = (operand << 1) | carry;
        self.write_operand(operand, address_mode);
        self.set_nz_flags(operand);
        self.get_common_rwm_cycles(address_mode)
    }

    pub(super) fn lsr(&mut self, operand: u8, address_mode: AddressMode) -> usize {
        self.p.set_c(operand & 1 > 0);
        let operand = operand >> 1;
        self.write_operand(operand, address_mode);
        self.set_nz_flags(operand);
        self.get_common_rwm_cycles(address_mode)
    }

    pub(super) fn ror(&mut self, operand: u8, address_mode: AddressMode) -> usize {
        let carry = self.p.c() as u8;
        self.p.set_c(operand & 1 > 0);
        let operand = (operand >> 1) | (carry * 0x80);
        self.write_operand(operand, address_mode);
        self.set_nz_flags(operand);
        self.get_common_rwm_cycles(address_mode)
    }

    fn stx(&mut self, address_mode: AddressMode) -> usize {
        self.write_operand(self.x, address_mode);
        1
    }

    pub(super) fn txa(&mut self) -> usize {
        self.a = self.x;
        self.set_nz_flags(self.a);
        1
    }

    fn txs(&mut self) -> usize {
        self.sp = self.x;
        1
    }

    fn ldx(&mut self, address_mode: AddressMode) -> usize {
        let (operand, page_cross) = self.read_operand(address_mode);
        self.x = operand;
        self.set_nz_flags(self.x);

        1 + page_cross as usize
    }

    pub(super) fn tax(&mut self) -> usize {
        self.x = self.a;
        self.set_nz_flags(self.x);
        1
    }

    fn tsx(&mut self) -> usize {
        self.x = self.sp;
        self.set_nz_flags(self.x);
        1
    }

    pub(super) fn dec(&mut self, operand: u8, address_mode: AddressMode) -> usize {
        let operand = operand.wrapping_sub(1);
        self.write_operand(operand, address_mode);
        self.set_nz_flags(operand);
        self.get_common_rwm_cycles(address_mode)
    }

    fn dex(&mut self) -> usize {
        self.x = self.x.wrapping_sub(1);
        self.set_nz_flags(self.x);
        1
    }

    pub(super) fn inc(&mut self, operand: u8, address_mode: AddressMode) -> usize {
        let operand = operand.wrapping_add(1);
        self.write_operand(operand, address_mode);
        self.set_nz_flags(operand);
        self.get_common_rwm_cycles(address_mode)
    }
}
