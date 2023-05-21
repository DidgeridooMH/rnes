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
            0x86 | 0x96 | 0x8E => self.stx(opcode)?,
            0x8A => self.txa()?,
            0x9A => self.txs()?,
            0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => self.ldx(opcode)?,
            0xAA => self.tax()?,
            0xBA => self.tsx()?,
            0xC6 | 0xD6 | 0xCE | 0xDE => self.dec(opcode)?,
            0xCA => self.dex()?,
            0xE6 | 0xF6 | 0xEE | 0xFF => self.inc(opcode)?,
            0xEA => (1, 2),
            _ => return Err(CoreError::OpcodeNotImplemented(opcode)),
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
            let operand = self.bus.borrow_mut().read_byte(address)?;
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
            let mut operand = self.bus.borrow_mut().read_byte(address)?;
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
            let operand = self.bus.borrow_mut().read_byte(address)?;
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
            let mut operand = self.bus.borrow_mut().read_byte(address)?;
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

    fn stx(&mut self, opcode: u8) -> OpcodeResult {
        let mut address_mode = AddressMode::from_code(opcode)?;
        if let AddressMode::ZeroPageX = address_mode {
            address_mode = AddressMode::ZeroPageY;
        }

        let address = self.get_address(address_mode)?.0;
        self.bus.borrow_mut().write_byte(address, self.x)?;

        Ok((
            address_mode.byte_code_size() + 1,
            address_mode.cycle_cost() + 1,
        ))
    }

    fn txa(&mut self) -> OpcodeResult {
        self.a = self.x;
        self.set_nz_flags(self.a);
        Ok((1, 2))
    }

    fn txs(&mut self) -> OpcodeResult {
        self.sp = self.x;
        Ok((1, 2))
    }

    fn ldx(&mut self, opcode: u8) -> OpcodeResult {
        let mut address_mode = AddressMode::from_code(opcode)?;
        address_mode = match address_mode {
            AddressMode::IndirectX => AddressMode::Immediate,
            AddressMode::ZeroPageX => AddressMode::ZeroPageY,
            _ => address_mode,
        };

        let (address, page_cross) = self.get_address(address_mode)?;
        self.x = self.bus.borrow_mut().read_byte(address)?;
        self.set_nz_flags(self.x);

        Ok((
            address_mode.byte_code_size() + 1,
            address_mode.cycle_cost() + 1 + page_cross as usize,
        ))
    }

    fn tax(&mut self) -> OpcodeResult {
        self.x = self.a;
        self.set_nz_flags(self.x);
        Ok((1, 2))
    }

    fn tsx(&mut self) -> OpcodeResult {
        self.x = self.sp;
        self.set_nz_flags(self.x);
        Ok((1, 2))
    }

    fn dec(&mut self, opcode: u8) -> OpcodeResult {
        let address_mode = AddressMode::from_code(opcode)?;
        let address = self.get_address(address_mode)?.0;
        let operand = self.bus.borrow_mut().read_byte(address)?;

        self.bus
            .borrow_mut()
            .write_byte(address, operand.wrapping_sub(1))?;

        self.set_nz_flags(operand.wrapping_sub(1));

        Ok((
            address_mode.byte_code_size() + 1,
            address_mode.cycle_cost() + 1,
        ))
    }

    fn dex(&mut self) -> OpcodeResult {
        self.x = self.x.wrapping_sub(1);
        self.set_nz_flags(self.x);
        Ok((1, 2))
    }

    fn inc(&mut self, opcode: u8) -> OpcodeResult {
        let address_mode = AddressMode::from_code(opcode)?;
        let address = self.get_address(address_mode)?.0;
        let operand = self.bus.borrow_mut().read_byte(address)?;

        self.bus
            .borrow_mut()
            .write_byte(address, operand.wrapping_add(1))?;

        self.set_nz_flags(operand.wrapping_add(1));

        Ok((
            address_mode.byte_code_size() + 1,
            address_mode.cycle_cost() + 1,
        ))
    }
}
