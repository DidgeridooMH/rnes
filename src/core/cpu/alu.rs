use super::{AddressMode, CoreError, CPU};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq)]
enum OpcodeGroup {
    ORA,
    AND,
    EOR,
    ADC,
    STA,
    LDA,
    CMP,
    SBC,
}

impl OpcodeGroup {
    pub fn from_code(code: u8) -> Self {
        match code {
            0 => OpcodeGroup::ORA,
            1 => OpcodeGroup::AND,
            2 => OpcodeGroup::EOR,
            3 => OpcodeGroup::ADC,
            4 => OpcodeGroup::STA,
            5 => OpcodeGroup::LDA,
            6 => OpcodeGroup::CMP,
            7 => OpcodeGroup::SBC,
            _ => unreachable!(),
        }
    }
}

impl CPU {
    pub fn run_alu_op(&mut self, opcode: u8) -> Result<usize, CoreError> {
        let address_mode = AddressMode::from_code(opcode)?;
        let (address, page_cross) = self.get_address(address_mode)?;
        let operand = self.bus.borrow_mut().read_byte(address)?;
        let opcode_group = OpcodeGroup::from_code(opcode >> 5);

        match opcode_group {
            OpcodeGroup::ORA => self.ora(operand),
            OpcodeGroup::AND => self.and(operand),
            _ => return Err(CoreError::OpcodeNotImplemented(opcode)),
        };

        let mut cycles = 1 + address_mode.cycle_cost();
        if page_cross || (opcode_group == OpcodeGroup::STA) {
            cycles += 1;
        }
        Ok(cycles)
    }

    fn ora(&mut self, operand: u8) {
        self.a = self.a | operand;
        self.set_nz_flags(self.a);
    }

    fn and(&mut self, operand: u8) {
        self.a = self.a & operand;
        self.set_nz_flags(self.a);
    }
}
