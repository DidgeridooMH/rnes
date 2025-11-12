use crate::core::{CoreError, CPU};

impl CPU {
    pub fn run_unofficial_op(&mut self, opcode: u8) -> Result<usize, CoreError> {
        return Err(CoreError::OpcodeNotImplemented(opcode));
    }
}
