use super::{AddressMode, CoreError, CPU};

#[cfg(test)]
mod tests;

impl CPU {
    pub fn run_alu_op(&mut self, opcode: u8) -> Result<usize, CoreError> {
        match opcode >> 5 {
            0 => self.ora(opcode),
            _ => Err(CoreError::OpcodeNotImplemented(opcode)),
        }
    }

    fn ora(&mut self, opcode: u8) -> Result<usize, CoreError> {
        let address_mode = AddressMode::from_code(opcode)?;
        let address = self.get_address(address_mode)?;
        let operand = self.bus.borrow_mut().read_byte(address)?;

        self.a = self.a | operand;
        self.set_nz_flags(self.a);

        Ok(1 + address_mode.cycle_cost())
    }
}
