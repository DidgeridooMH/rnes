use super::{AddressMode, CoreError, StatusRegister, CPU};

#[cfg(test)]
mod tests;

type OpcodeResult = Result<(u16, usize), CoreError>;

impl CPU {
    pub fn run_control_op(&mut self, opcode: u8) -> Result<usize, CoreError> {
        let (ins_size, cycles) = match opcode {
            0 => self.brk()?,
            0x8 => self.php()?,
            0x28 => self.plp()?,
            0x48 => self.pha()?,
            0x68 => self.pla()?,
            o if o & 0x1F == 0x10 => self.branch(o)?,
            0x18 => {
                self.p.set_c(false);
                (1, 2)
            }
            0x38 => {
                self.p.set_c(true);
                (1, 2)
            }
            0x58 => {
                self.p.set_i(false);
                (1, 2)
            }
            0x78 => {
                self.p.set_i(true);
                (1, 2)
            }
            0xB8 => {
                self.p.set_v(false);
                (1, 2)
            }
            0xD8 => {
                self.p.set_d(false);
                (1, 2)
            }
            0xF8 => {
                self.p.set_d(true);
                (1, 2)
            }
            0x20 => self.jsr()?,
            0x24 | 0x2C => self.bit(opcode)?,
            0x40 => self.rti()?,
            0x4C | 0x6C => self.jmp(opcode)?,
            _ => unimplemented!("Rest of control opcodes"),
        };

        self.pc += ins_size;

        Ok(cycles)
    }

    fn brk(&mut self) -> OpcodeResult {
        self.push_word(self.pc + 2)?;
        self.php()?;
        self.pc = self.bus.borrow_mut().read_word(0xFFFEu16)?;
        self.p.set_i(true);

        Ok((0, 7))
    }

    fn php(&mut self) -> OpcodeResult {
        let mut status = self.p;
        status.set_b(3u8);
        self.push_byte(status.0)?;

        Ok((1, 3))
    }

    fn plp(&mut self) -> OpcodeResult {
        let mut status = StatusRegister {
            0: self.pop_byte()?,
        };
        status.set_b(0u8);
        self.p = status;

        Ok((1, 4))
    }

    fn pha(&mut self) -> OpcodeResult {
        self.push_byte(self.a)?;
        Ok((1, 3))
    }

    fn pla(&mut self) -> OpcodeResult {
        self.a = self.pop_byte()?;
        self.set_nz_flags(self.a);
        Ok((1, 4))
    }

    fn branch(&mut self, opcode: u8) -> OpcodeResult {
        let mut should_branch = match opcode >> 6 {
            0 => self.p.n(),
            1 => self.p.v(),
            2 => self.p.c(),
            3 => self.p.z(),
            _ => unreachable!(),
        };

        if opcode & 0x20 == 0 {
            should_branch = !should_branch;
        }

        if should_branch {
            let prev_pc = self.pc;
            let offset = self.bus.borrow_mut().read_byte(self.pc + 1)? as i8;
            self.pc = (self.pc as i16 + offset as i16) as u16;

            if (prev_pc >> 8) != (self.pc >> 8) {
                Ok((0, 5))
            } else {
                Ok((0, 3))
            }
        } else {
            Ok((2, 2))
        }
    }

    fn jsr(&mut self) -> OpcodeResult {
        // PC + 3'size of ins' - 1'RTS has size 1'
        self.push_word(self.pc + 2)?;
        self.pc = self.bus.borrow_mut().read_word(self.pc + 1)?;
        Ok((0, 6))
    }

    fn bit(&mut self, opcode: u8) -> OpcodeResult {
        let addr_mode = AddressMode::from_code(opcode)?;
        let (address, _) = self.get_address(addr_mode)?;
        let operand = self.bus.borrow_mut().read_byte(address)?;
        let result = self.a & operand;

        self.set_nz_flags(result);
        self.p.set_v(result & 0x40u8 > 0);

        Ok((addr_mode.byte_code_size() + 1, addr_mode.cycle_cost() + 1))
    }

    fn rti(&mut self) -> OpcodeResult {
        self.p.0 = self.pop_byte()?;
        self.p.set_b(0);
        self.pc = self.pop_word()?;
        Ok((1, 6))
    }

    fn jmp(&mut self, opcode: u8) -> OpcodeResult {
        let addr_mode = match opcode {
            0x4Cu8 => AddressMode::Absolute,
            0x6Cu8 => AddressMode::Indirect,
            _ => unreachable!(),
        };

        let (address, _) = self.get_address(addr_mode)?;
        println!("{}", address);
        self.pc = address;

        // Don't increment the PC so that jmps go direct.
        Ok((0, addr_mode.cycle_cost()))
    }
}
