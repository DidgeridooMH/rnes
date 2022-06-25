mod alu;
mod memory;
mod status;
#[cfg(test)]
mod tests;

use self::{memory::InternalRam, status::StatusRegister};
use super::{Addressable, Bus};
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

    fn set_nz_flags(&mut self, operand: u8) {
        self.p.set_n((operand >> 7) > 0);
        self.p.set_z(operand == 0);
    }

    fn get_address(&self, address_mode: AddressMode) -> u16 {
        match address_mode {
            AddressMode::Immediate => self.pc + 1,
            AddressMode::ZeroPage => self.bus.borrow_mut().read_byte(self.pc + 1).unwrap() as u16,
            AddressMode::ZeroPageX => {
                (self.get_address(AddressMode::ZeroPage) + self.x as u16) % 256
            }
            AddressMode::ZeroPageY => {
                (self.get_address(AddressMode::ZeroPage) + self.y as u16) % 256
            }
            AddressMode::Absolute => self.bus.borrow_mut().read_word(self.pc + 1).unwrap(),
            AddressMode::AbsoluteX => self.get_address(AddressMode::Absolute) + self.x as u16,
            AddressMode::AbsoluteY => self.get_address(AddressMode::Absolute) + self.y as u16,
            AddressMode::IndirectX => {
                let zero_page_address = self.get_address(AddressMode::ZeroPageX);
                self.bus
                    .borrow_mut()
                    .read_word_bug(zero_page_address)
                    .unwrap()
            }
            AddressMode::IndirectY => {
                let zero_page_address = self.get_address(AddressMode::ZeroPage);
                self.bus
                    .borrow_mut()
                    .read_word_bug(zero_page_address)
                    .unwrap()
                    + self.y as u16
            }
        }
    }
}

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
