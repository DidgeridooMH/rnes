mod bus;
mod cpu;
mod ppu;

pub use bus::*;
pub use cpu::*;
pub use ppu::*;

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum CoreError {
    AddressDecode(u8),
    OpcodeNotImplemented(u8),
    InvalidRegion(u16),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            CoreError::InvalidRegion(address) => {
                write!(f, "Address access violation at 0x{:04X}", address)
            }
            CoreError::AddressDecode(opcode) => {
                write!(f, "Unknown address mode from 0x{:02X}", opcode)
            }
            CoreError::OpcodeNotImplemented(opcode) => {
                write!(f, "Opcode not implemented: 0x{0:02X}", opcode)
            }
        }
    }
}
