mod address_mode;
mod alu;
mod control;
mod memory;
mod nop;
mod opcodes;
mod rwm;
mod status;
mod unofficial;

// #[cfg(test)]
// mod tests;

use crate::core::cpu::{address_mode::AddressMode, opcodes::OPCODES};

use self::{memory::InternalRam, status::StatusRegister};
use super::{Addressable, Bus, CoreError};
use std::{cell::RefCell, rc::Rc};

#[derive(Copy, Clone, Debug)]
enum Interrupt {
    Reset,
    Nmi,
}

pub type OpcodeResult = Result<(u16, usize), CoreError>;

const OAM_DMA_SIZE: usize = 256;

#[derive(Copy, Clone, Default)]
pub struct OamDmaRequest {
    address: u16,
    length: usize,
}

impl Addressable for OamDmaRequest {
    fn read_byte(&mut self, _address: u16) -> Option<u8> {
        // Open bus
        None
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        if address == 0x4014 {
            self.address = (data as u16) << 8;
            self.length = OAM_DMA_SIZE
        } else {
            println!("(warn) Unexpected write to {address:X} in OAM CPU register");
        }
    }
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
    oam_request: Rc<RefCell<OamDmaRequest>>,
    cycles: usize,
}

impl CPU {
    pub fn new(bus: &Rc<RefCell<Bus>>) -> Self {
        bus.borrow_mut()
            .register_region(0x0u16..=0x1FFFu16, InternalRam::new());
        let oam_request = Rc::new(RefCell::new(OamDmaRequest::default()));
        bus.borrow_mut()
            .register_region(0x4014u16..=0x4014u16, oam_request.clone());

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
            oam_request,
            cycles: 0,
        }
    }

    pub fn set_show_ops(&mut self, show: bool) {
        self.show_ops = show;
    }

    pub fn generate_nmi(&mut self) {
        self.interrupt = Some(Interrupt::Nmi);
    }

    pub fn tick(&mut self) -> Result<usize, CoreError> {
        let oam_request_length = self.oam_request.borrow().length;
        if oam_request_length > 0 {
            let mut bus = self.bus.borrow_mut();
            let oam_address = {
                let mut request = self.oam_request.borrow_mut();
                let address = request.address;
                request.address += 1;
                request.length -= 1;
                address
            };
            let oam_byte = bus.read_byte(oam_address);
            bus.write_byte(0x2004, oam_byte);
            return Ok(2);
        }

        if let Some(interrupt) = self.interrupt {
            self.push_word(self.pc);
            let mut status = self.p;
            status.set_b(0);
            self.push_byte(status.0);
            let vector_address = match interrupt {
                Interrupt::Nmi => 0xFFFA,
                Interrupt::Reset => 0xFFFC,
            };
            self.pc = self.bus.borrow_mut().read_word(vector_address);
            self.p.set_i(true);
            self.interrupt = None;
        }

        let opcode = self.bus.borrow_mut().read_byte(self.pc);
        if self.show_ops {
            print!(
                "c{} A:{:02X} X:{:02X} Y:{:02X} S:{:02X} P:{} ${:04X}: {}",
                self.cycles,
                self.a,
                self.x,
                self.y,
                self.sp,
                self.p,
                self.pc,
                OPCODES[opcode as usize]
            );
        }
        let cycles = match opcode % 4 {
            0 => self.run_control_op(opcode)?,
            1 => self.run_alu_op(opcode),
            2 => self.run_rwm_op(opcode)?,
            3 => self.run_unofficial_op(opcode)?,
            _ => unreachable!(),
        };

        if self.show_ops {
            println!();
        }

        self.cycles += cycles;

        Ok(cycles)
    }

    fn set_nz_flags(&mut self, operand: u8) {
        self.p.set_n((operand >> 7) > 0);
        self.p.set_z(operand == 0);
    }

    fn get_address(&self, address_mode: AddressMode) -> (u16, bool) {
        match address_mode {
            AddressMode::Implied => (0, false),
            AddressMode::Accumulator => (0, false),
            AddressMode::Immediate => (self.pc + 1, false),
            AddressMode::ZeroPage => (self.bus.borrow_mut().read_byte(self.pc + 1) as u16, false),
            AddressMode::ZeroPageX => (
                (self.get_address(AddressMode::ZeroPage).0 + self.x as u16) % 256,
                false,
            ),
            AddressMode::ZeroPageY => (
                (self.get_address(AddressMode::ZeroPage).0 + self.y as u16) % 256,
                false,
            ),
            AddressMode::Absolute => (self.bus.borrow_mut().read_word(self.pc + 1), false),
            AddressMode::AbsoluteX => {
                let address = self.get_address(AddressMode::Absolute).0;
                (
                    address + self.x as u16,
                    address & 0xFF00 != (address + self.x as u16) & 0xFF00,
                )
            }
            AddressMode::AbsoluteY => {
                let address = self.get_address(AddressMode::Absolute).0;
                (
                    address + self.y as u16,
                    address & 0xFF00 != (address + self.y as u16) & 0xFF00,
                )
            }
            AddressMode::IndirectX => {
                let zero_page_address = self.get_address(AddressMode::ZeroPageX).0;
                (
                    self.bus.borrow_mut().read_word_bug(zero_page_address),
                    false,
                )
            }
            AddressMode::IndirectY => {
                let zero_page_address = self.get_address(AddressMode::ZeroPage).0;
                let address = self.bus.borrow_mut().read_word_bug(zero_page_address);
                (
                    address + self.y as u16,
                    address & 0xFF00 != (address + self.y as u16) & 0xFF00,
                )
            }
            AddressMode::Indirect => {
                let address = self.get_address(AddressMode::Absolute).0;
                (self.bus.borrow_mut().read_word_bug(address), false)
            }
        }
    }

    fn read_operand(&mut self, address_mode: AddressMode) -> (u8, bool) {
        match address_mode {
            AddressMode::Accumulator => (self.a, false),
            _ => {
                let (address, page_cross) = self.get_address(address_mode);
                (self.bus.borrow_mut().read_byte(address), page_cross)
            }
        }
    }

    fn write_operand(&mut self, operand: u8, address_mode: AddressMode) {
        match address_mode {
            AddressMode::Accumulator => self.a = operand,
            _ => {
                let address = self.get_address(address_mode).0;
                self.bus.borrow_mut().write_byte(address, operand);
            }
        }
    }

    fn push_byte(&mut self, data: u8) {
        self.bus
            .borrow_mut()
            .write_byte(0x100 + self.sp as u16, data);
        self.sp -= 1;
    }

    fn pop_byte(&mut self) -> u8 {
        self.sp += 1;
        self.bus.borrow_mut().read_byte(0x100 + self.sp as u16)
    }

    fn push_word(&mut self, data: u16) {
        self.push_byte((data >> 8) as u8);
        self.push_byte(data as u8);
    }

    fn pop_word(&mut self) -> u16 {
        let mut result = self.pop_byte() as u16;
        result |= (self.pop_byte() as u16) << 8;
        result
    }

    pub fn dump(&self) {
        println!("\n==== CPU DUMP ====");
        println!("A: ${:X}\tX: ${:X}", self.a, self.x);
        println!("Y: ${:X}\tSP: ${:X}", self.y, self.sp);
        println!("PC: ${:X}\tP: {:?}", self.pc, self.p);
    }
}
