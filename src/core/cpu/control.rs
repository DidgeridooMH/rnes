use super::{CoreError, CPU};

impl CPU {
    pub fn run_control_op(&mut self, opcode: u8) -> Result<usize, CoreError> {
        match opcode {
            0 => self.break_int(),
            _ => unimplemented!("Rest of control opcodes"),
        }
    }

    fn break_int(&mut self) -> Result<usize, CoreError> {
        self.push_word(self.pc)?;
        self.push_byte(self.p.0)?;
        self.pc = self.bus.borrow_mut().read_word(0xFFFEu16)?;

        Ok(0)
    }
}
