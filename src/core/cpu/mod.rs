mod alu;
mod control;
mod memory;
mod opcodes;
mod rwm;
mod status;

#[cfg(test)]
mod tests;

use self::{memory::InternalRam, status::StatusRegister};
use super::{Addressable, Bus, CoreError};
use std::{cell::RefCell, rc::Rc};

#[derive(Copy, Clone, Debug)]
enum Interrupt {
    Reset,
    Nmi,
}

pub struct CPU {
    bus: Rc<RefCell<Bus>>,
    pub a: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    pub p: StatusRegister,
    interrupt: Option<Interrupt>,
    show_ops: bool,
}

impl CPU {
    pub fn new(bus: &Rc<RefCell<Bus>>) -> Self {
        bus.borrow_mut()
            .register_region(0x0u16..=0x2000u16, InternalRam::new());
        Self {
            bus: bus.clone(),
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFFu8,
            pc: 0xFFFCu16,
            p: StatusRegister(0),
            interrupt: Some(Interrupt::Reset),
            show_ops: false,
        }
    }

    pub fn set_show_ops(&mut self, show: bool) {
        self.show_ops = show;
    }

    pub fn generate_nmi(&mut self) {
        self.interrupt = Some(Interrupt::Nmi);
    }

    pub fn tick(&mut self) -> Result<usize, CoreError> {
        if let Some(interrupt) = self.interrupt {
            self.push_word(self.pc)?;
            let mut status = self.p;
            status.set_b(0);
            self.push_byte(status.0)?;
            let vector_address = match interrupt {
                Interrupt::Nmi => 0xFFFA,
                Interrupt::Reset => 0xFFFC,
            };
            self.pc = self.bus.borrow_mut().read_word(vector_address)?;
            self.p.set_i(true);
            self.interrupt = None;
        }

        let opcode = self.bus.borrow_mut().read_byte(self.pc)?;
        if self.show_ops {
            print!(
                "0x{:X}: {}({:X})",
                self.pc,
                opcodes::OPCODES[opcode as usize],
                opcode
            );
        }
        let cycles = match opcode % 4 {
            0 => self.run_control_op(opcode)?,
            1 => self.run_alu_op(opcode)?,
            2 => self.run_rwm_op(opcode)?,
            3 => todo!("unofficial operations need implemented"),
            _ => unreachable!(),
        };

        if self.show_ops {
            println!();
        }

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
            AddressMode::Indirect => {
                let address = self.get_address(AddressMode::Absolute)?.0;
                Ok((self.bus.borrow_mut().read_word_bug(address)?, false))
            }
        }
    }

    fn push_byte(&mut self, data: u8) -> Result<(), CoreError> {
        self.bus.borrow_mut().write_byte(self.sp as u16, data)?;
        self.sp -= 1;
        Ok(())
    }

    fn pop_byte(&mut self) -> Result<u8, CoreError> {
        self.sp += 1;
        self.bus.borrow_mut().read_byte(self.sp as u16)
    }

    fn push_word(&mut self, data: u16) -> Result<(), CoreError> {
        self.push_byte((data >> 8) as u8)?;
        self.push_byte(data as u8)?;
        Ok(())
    }

    fn pop_word(&mut self) -> Result<u16, CoreError> {
        let mut result = self.pop_byte()? as u16;
        result |= (self.pop_byte()? as u16) << 8;
        Ok(result)
    }
}

#[derive(Copy, Clone, Debug)]
enum AddressMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
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
            AddressMode::Indirect => 4,
        }
    }

    pub fn byte_code_size(&self) -> u16 {
        match &self {
            AddressMode::Immediate
            | AddressMode::ZeroPage
            | AddressMode::ZeroPageX
            | AddressMode::ZeroPageY
            | AddressMode::IndirectX
            | AddressMode::IndirectY => 1,
            AddressMode::Absolute
            | AddressMode::AbsoluteX
            | AddressMode::AbsoluteY
            | AddressMode::Indirect => 2,
        }
    }
}
