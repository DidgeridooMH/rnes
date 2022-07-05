use super::{CoreError, StatusRegister, CPU};

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
}
