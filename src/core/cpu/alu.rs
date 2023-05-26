use super::{AddressMode, CoreError, CPU};

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
    pub fn run_alu_op(&mut self, opcode: u8) -> Result<usize, CoreError> {
        let address_mode = AddressMode::from_code(opcode)?;
        let (address, mut page_cross) = self.get_address(address_mode)?;
        let opcode_group = OpcodeGroup::from_code(opcode >> 5);
        let mut operand = 0;
        if opcode_group != OpcodeGroup::Sta {
            operand = self.bus.borrow_mut().read_byte(address)?;
        }

        if self.show_ops {
            if let AddressMode::Immediate = address_mode {
                print!(" #${:02X}", operand);
            } else {
                print!(" ${:04X} = #${:02X}", address, operand);
            }
        }

        match opcode_group {
            OpcodeGroup::Ora => self.ora(operand),
            OpcodeGroup::And => self.and(operand),
            OpcodeGroup::Eor => self.eor(operand),
            OpcodeGroup::Adc => self.adc(operand),
            OpcodeGroup::Sta => self.bus.borrow_mut().write_byte(address, self.a)?,
            OpcodeGroup::Lda => self.lda(operand),
            OpcodeGroup::Cmp => self.cmp(operand),
            OpcodeGroup::Sbc => self.sbc(operand),
        }

        let mut cycles = 1 + address_mode.cycle_cost();

        if opcode_group == OpcodeGroup::Sta {
            match address_mode {
                AddressMode::AbsoluteX | AddressMode::AbsoluteY | AddressMode::IndirectY => {
                    page_cross = true;
                }
                _ => {}
            }
        }

        if page_cross {
            cycles += 1;
        }

        self.pc += 1 + address_mode.byte_code_size();
        Ok(cycles)
    }

    fn ora(&mut self, operand: u8) {
        self.a |= operand;
        self.set_nz_flags(self.a);
    }

    fn and(&mut self, operand: u8) {
        self.a &= operand;
        self.set_nz_flags(self.a);
    }

    fn eor(&mut self, operand: u8) {
        self.a ^= operand;
        self.set_nz_flags(self.a);
    }

    fn adc(&mut self, operand: u8) {
        let carry = self.p.c() as u8;
        self.p.set_c(self.a.checked_add(operand).is_none());
        self.p
            .set_v((self.a as i8).checked_add(operand as i8).is_none());
        self.a = self.a.wrapping_add(carry.wrapping_add(operand));
        self.set_nz_flags(self.a);
    }

    fn lda(&mut self, operand: u8) {
        self.a = operand;
        self.set_nz_flags(self.a);
    }

    fn cmp(&mut self, operand: u8) {
        let result = self.a.wrapping_sub(operand);
        self.p.set_c(self.a >= operand);
        self.set_nz_flags(result);
    }

    fn sbc(&mut self, operand: u8) {
        self.p
            .set_v(match (self.a as i8).checked_sub(operand as i8) {
                Some(r) => r.checked_sub(1 - (self.p.c() as i8)).is_none(),
                None => true,
            });
        self.a = self.a.wrapping_sub(operand.wrapping_sub(self.p.c() as u8));
        self.set_nz_flags(self.a);
        self.p.set_c(!self.p.v());
    }
}
