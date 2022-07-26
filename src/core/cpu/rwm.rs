use super::{AddressMode, CoreError, CPU};

#[cfg(test)]
mod tests;

type OpcodeResult = Result<(u16, usize), CoreError>;

impl CPU {
    pub fn run_rwm_op(&mut self, opcode: u8) -> Result<usize, CoreError> {
        let (ins_size, cycles) = match opcode {
            0x06 | 0x0A | 0x0E | 0x16 | 0x1E => self.asl(opcode)?,
            0x2A | 0x26 | 0x36 | 0x2E | 0x3E => self.rol(opcode)?,
            0x4A | 0x46 | 0x56 | 0x4E | 0x5E => self.lsr(opcode)?,
            0x6A | 0x66 | 0x76 | 0x6E | 0x7E => self.ror(opcode)?,
            _ => unreachable!(),
        };

        self.pc += ins_size;

        Ok(cycles)
    }

    fn asl(&mut self, opcode: u8) -> OpcodeResult {
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
            Ok((
                address_mode.byte_code_size() + 1,
                address_mode.cycle_cost() + 2,
            ))
        }
    }

    fn rol(&mut self, opcode: u8) -> OpcodeResult {
        let address_mode = AddressMode::from_code(opcode)?;
        let carry = self.p.c() as u8;
        if let AddressMode::Immediate = address_mode {
            self.p.set_c(self.a & 0x80 > 0);
            self.a <<= 1;
            self.a |= carry;
            self.set_nz_flags(self.a);
            Ok((1, 2))
        } else {
            let address = self.get_address(address_mode)?.0;
            let mut operand = self.bus.borrow().read_byte(address)?;
            self.p.set_c(operand & 0x80 > 0);
            operand <<= 1;
            operand |= carry;
            self.bus.borrow_mut().write_byte(address, operand)?;
            self.set_nz_flags(operand);
            Ok((
                address_mode.byte_code_size() + 1,
                address_mode.cycle_cost() + 2,
            ))
        }
    }

    fn lsr(&mut self, opcode: u8) -> OpcodeResult {
        let address_mode = AddressMode::from_code(opcode)?;
        if let AddressMode::Immediate = address_mode {
            self.p.set_c(self.a & 1 > 0);
            self.a >>= 1;
            self.set_nz_flags(self.a);
            Ok((1, 2))
        } else {
            let address = self.get_address(address_mode)?.0;
            let operand = self.bus.borrow().read_byte(address)?;
            self.p.set_c(operand & 1 > 0);
            self.bus.borrow_mut().write_byte(address, operand >> 1)?;
            self.set_nz_flags(operand >> 1);
            Ok((
                address_mode.byte_code_size() + 1,
                address_mode.cycle_cost() + 2,
            ))
        }
    }

    fn ror(&mut self, opcode: u8) -> OpcodeResult {
        let address_mode = AddressMode::from_code(opcode)?;
        let carry = self.p.c() as u8;
        if let AddressMode::Immediate = address_mode {
            self.p.set_c(self.a & 1 > 0);
            self.a >>= 1;
            self.a |= carry * 0x80;
            self.set_nz_flags(self.a);
            Ok((1, 2))
        } else {
            let address = self.get_address(address_mode)?.0;
            let mut operand = self.bus.borrow().read_byte(address)?;
            self.p.set_c(operand & 1 > 0);
            operand >>= 1;
            operand |= carry * 0x80;
            self.bus.borrow_mut().write_byte(address, operand)?;
            self.set_nz_flags(operand);
            Ok((
                address_mode.byte_code_size() + 1,
                address_mode.cycle_cost() + 2,
            ))
        }
    }
}
