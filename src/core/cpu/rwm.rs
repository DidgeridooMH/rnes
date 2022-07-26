use super::{AddressMode, CoreError, CPU};

#[cfg(test)]
mod tests;

impl CPU {
    pub fn run_rwm_op(&mut self, opcode: u8) -> Result<usize, CoreError> {
        let (ins_size, cycles) = match opcode {
            0x06 | 0x0A | 0x0E | 0x16 | 0x1E => self.asl(opcode)?,
            _ => unreachable!(),
        };

        self.pc += ins_size;

        Ok(cycles)
    }

    fn asl(&mut self, opcode: u8) -> Result<(u16, usize), CoreError> {
        let address_mode = AddressMode::from_code(opcode)?;
        if let AddressMode::Immediate = address_mode {
            self.p.set_c(self.a & 0x80 > 0);
            self.a <<= 1;
            self.set_nz_flags(self.a);
            Ok((1, 2))
        } else {
            let address = self.get_address(address_mode)?.0;
            let operand = self.bus.borrow().read_byte(address)?;
            self.p.set_c(operand & 0x80 > 0);
            self.bus.borrow_mut().write_byte(address, operand << 1)?;
            self.set_nz_flags(operand << 1);
            Ok((address_mode.byte_code_size() + 1, address_mode.cycle_cost()))
        }
    }
}
