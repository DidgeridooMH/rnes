mod alu;
mod memory;
mod status;
#[cfg(test)]
mod tests;

use self::{memory::InternalRam, status::StatusRegister};
use super::{Addressable, Bus, CoreError};
use std::{cell::RefCell, rc::Rc};

pub struct CPU {
    bus: Rc<RefCell<Bus>>,
    internal_ram: Rc<RefCell<InternalRam>>,
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    p: StatusRegister,
}

impl CPU {
    pub fn new(bus: &Rc<RefCell<Bus>>) -> Rc<RefCell<Self>> {
        let cpu = Rc::new(RefCell::new(Self {
            bus: bus.clone(),
            internal_ram: InternalRam::new(),
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFFu8,
            pc: 0xFFFCu16,
            p: StatusRegister(0),
        }));
        bus.borrow_mut()
            .register_region(0x0u16..0x2000u16, cpu.borrow().internal_ram.clone());
        cpu
    }

    pub fn tick(&mut self) -> Result<usize, CoreError> {
        // TODO: Untested.
        let opcode = self.bus.borrow().read_byte(self.pc)?;
        let cycles = match opcode % 4 {
            0 => todo!("control instructions need to be implemented"),
            1 => self.run_alu_op(opcode)?,
            2 => todo!("RMW operations need to be implemented"),
            3 => todo!("unofficial operations need implemented"),
            _ => unreachable!(),
        };

        Ok(cycles)
    }

    fn set_nz_flags(&mut self, operand: u8) {
        self.p.set_n((operand >> 7) > 0);
        self.p.set_z(operand == 0);
    }

    fn get_address(&self, address_mode: AddressMode) -> Result<(u16, bool), CoreError> {
        match address_mode {
            AddressMode::Immediate => Ok((self.pc + 1, false)),
            AddressMode::ZeroPage => Ok((
                self.bus.borrow_mut().read_byte(self.pc + 1).unwrap() as u16,
                false,
            )),
            AddressMode::ZeroPageX => Ok((
                (self.get_address(AddressMode::ZeroPage)?.0 + self.x as u16) % 256,
                false,
            )),
            AddressMode::ZeroPageY => Ok((
                (self.get_address(AddressMode::ZeroPage)?.0 + self.y as u16) % 256,
                false,
            )),
            AddressMode::Absolute => Ok((self.bus.borrow_mut().read_word(self.pc + 1)?, false)),
            AddressMode::AbsoluteX => {
                let address = self.get_address(AddressMode::Absolute)?.0;
                Ok((
                    address + self.x as u16,
                    address & 0xFF00 != (address + self.x as u16) & 0xFF00,
                ))
            }
            AddressMode::AbsoluteY => {
                let address = self.get_address(AddressMode::Absolute)?.0;
                Ok((
                    address + self.y as u16,
                    address & 0xFF00 != (address + self.y as u16) & 0xFF00,
                ))
            }
            AddressMode::IndirectX => {
                let zero_page_address = self.get_address(AddressMode::ZeroPageX)?.0;
                Ok((
                    self.bus.borrow_mut().read_word_bug(zero_page_address)?,
                    false,
                ))
            }
            AddressMode::IndirectY => {
                let zero_page_address = self.get_address(AddressMode::ZeroPage)?.0;
                let address = self.bus.borrow_mut().read_word_bug(zero_page_address)?;
                Ok((
                    address + self.y as u16,
                    address & 0xFF00 != (address + self.y as u16) & 0xFF00,
                ))
            }
        }
    }
}

#[derive(Copy, Clone)]
enum AddressMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
}

impl AddressMode {
    pub fn from_code(opcode: u8) -> Result<AddressMode, CoreError> {
        let mode_code = (opcode >> 2) % 8;
        match mode_code {
            0 => Ok(AddressMode::IndirectX),
            1 => Ok(AddressMode::ZeroPage),
            2 => Ok(AddressMode::Immediate),
            3 => Ok(AddressMode::Absolute),
            4 => Ok(AddressMode::IndirectY),
            5 => Ok(AddressMode::ZeroPageX),
            6 => Ok(AddressMode::AbsoluteY),
            7 => Ok(AddressMode::AbsoluteX),
            _ => Err(CoreError::AddressDecode(opcode)),
        }
    }

    pub fn cycle_cost(&self) -> usize {
        match &self {
            AddressMode::Immediate => 1,
            AddressMode::ZeroPage => 2,
            AddressMode::ZeroPageX
            | AddressMode::ZeroPageY
            | AddressMode::Absolute
            | AddressMode::AbsoluteX
            | AddressMode::AbsoluteY => 3,
            AddressMode::IndirectX => 6,
            AddressMode::IndirectY => 5,
        }
    }
}
