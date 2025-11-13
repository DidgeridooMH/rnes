use crate::core::{cpu::AddressMode, CPU};

impl CPU {
    pub(super) fn nop(&mut self, address_mode: AddressMode) -> usize {
        if let AddressMode::Implied = address_mode {
            1
        } else {
            1 + self.read_operand(address_mode).1 as usize
        }
    }
}
