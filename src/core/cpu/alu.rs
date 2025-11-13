use super::{AddressMode, CPU};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq)]
enum OpcodeGroup {
    Ora,
    And,
    Eor,
    Adc,
    Sta,
    Lda,
    Cmp,
    Sbc,
}

impl OpcodeGroup {
    pub fn from_code(code: u8) -> Self {
        match code {
            0 => OpcodeGroup::Ora,
            1 => OpcodeGroup::And,
            2 => OpcodeGroup::Eor,
            3 => OpcodeGroup::Adc,
            4 => OpcodeGroup::Sta,
            5 => OpcodeGroup::Lda,
            6 => OpcodeGroup::Cmp,
            7 => OpcodeGroup::Sbc,
            _ => unreachable!(),
        }
    }
}

impl CPU {
    pub fn run_alu_op(&mut self, opcode: u8) -> usize {
        let address_mode = AddressMode::from_code(opcode);
        let opcode_group = OpcodeGroup::from_code(opcode >> 5);
        let (operand, page_cross) = if opcode_group != OpcodeGroup::Sta {
            self.read_operand(address_mode)
        } else {
            (self.a, true)
        };

        match opcode_group {
            OpcodeGroup::Ora => self.ora(operand),
            OpcodeGroup::And => self.and(operand),
            OpcodeGroup::Eor => self.eor(operand),
            OpcodeGroup::Adc => self.adc(operand),
            OpcodeGroup::Sta => {
                if opcode == 0x89 {
                    self.nop(address_mode);
                } else {
                    self.write_operand(self.a, address_mode);
                }
            }
            OpcodeGroup::Lda => self.lda(operand),
            OpcodeGroup::Cmp => self.cmp(operand),
            OpcodeGroup::Sbc => self.sbc(operand),
        }

        self.pc += 1 + address_mode.byte_code_size();

        1 + address_mode.cycle_cost(page_cross)
    }

    pub(super) fn ora(&mut self, operand: u8) {
        self.a |= operand;
        self.set_nz_flags(self.a);
    }

    pub(super) fn and(&mut self, operand: u8) {
        self.a &= operand;
        self.set_nz_flags(self.a);
    }

    pub(super) fn eor(&mut self, operand: u8) {
        self.a ^= operand;
        self.set_nz_flags(self.a);
    }

    pub(super) fn adc(&mut self, operand: u8) {
        let carry = self.p.c() as u8;
        let (sum1, overflow1) = self.a.overflowing_add(operand);
        let (sum2, overflow2) = sum1.overflowing_add(carry);

        let a = self.a;
        self.a = sum2;
        self.p.set_c(overflow1 || overflow2);
        self.p
            .set_v((a ^ operand) & 0x80 == 0 && (a ^ self.a) & 0x80 != 0);

        self.set_nz_flags(self.a);
    }

    pub(super) fn lda(&mut self, operand: u8) {
        self.a = operand;
        self.set_nz_flags(self.a);
    }

    pub(super) fn cmp(&mut self, operand: u8) {
        let result = self.a.wrapping_sub(operand);
        self.p.set_c(self.a >= operand);
        self.set_nz_flags(result);
    }

    pub(super) fn sbc(&mut self, operand: u8) {
        let carry = self.p.c() as u8;
        let (diff1, overflow1) = self.a.overflowing_sub(operand);
        let (diff2, overflow2) = diff1.overflowing_sub(1 - carry);

        let a = self.a;
        self.a = diff2;
        self.p.set_c(!(overflow1 || overflow2));
        self.p
            .set_v((a ^ operand) & 0x80 != 0 && (a ^ self.a) & 0x80 != 0);

        self.set_nz_flags(self.a);
    }
}
