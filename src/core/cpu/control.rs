use super::{AddressMode, CoreError, StatusRegister, CPU};

#[cfg(test)]
mod tests;

impl CPU {
    pub fn run_control_op(&mut self, opcode: u8) -> Result<usize, CoreError> {
        let address_mode = AddressMode::from_code(opcode);

        let (ins_size, cycles) = match opcode {
            0 => self.brk(),
            0x8 => self.php(),
            0x28 => self.plp(),
            0x48 => self.pha(),
            0x68 => self.pla(),
            o if o & 0x1F == 0x10 => self.branch(o),
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
            0x20 => self.jsr(),
            0x24 | 0x2C => self.bit(opcode),
            0x40 => self.rti(),
            0x60 => self.rts(),
            0x4C | 0x6C => self.jmp(opcode),
            0x84 | 0x94 | 0x8C => self.sty(opcode),
            0x88 => self.dey(),
            0x98 => self.tya(),
            0xA8 => self.tay(),
            0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => self.ldy(opcode),
            0xC0 | 0xC4 | 0xCC => self.cpy(opcode),
            0xE0 | 0xE4 | 0xEC => self.cpx(opcode),
            0xC8 => self.iny(),
            0xE8 => self.inx(),
            0x04 | 0x0C | 0x14 | 0x1C | 0x34 | 0x3C | 0x44 | 0x54 | 0x5C | 0x64 | 0x74 | 0x7C
            | 0x80 | 0xD4 | 0xDC | 0xF4 | 0xFC => (
                1 + AddressMode::from_code(opcode).byte_code_size(),
                self.nop(AddressMode::from_code(opcode)),
            ),
            0x9C => {
                self.shy(address_mode);
                (3, 5)
            }
            _ => return Err(CoreError::OpcodeNotImplemented(opcode)),
        };

        self.pc += ins_size;

        Ok(cycles)
    }

    fn brk(&mut self) -> (u16, usize) {
        self.push_word(self.pc + 2);
        self.php();
        self.pc = self.bus.borrow_mut().read_word(0xFFFEu16);
        self.p.set_i(true);
        (0, 7)
    }

    fn php(&mut self) -> (u16, usize) {
        let mut status = self.p;
        status.set_b(3u8);
        self.push_byte(status.0);
        (1, 3)
    }

    fn plp(&mut self) -> (u16, usize) {
        let mut status = StatusRegister(self.pop_byte());
        status.set_b(0u8);
        self.p = status;
        (1, 4)
    }

    fn pha(&mut self) -> (u16, usize) {
        self.push_byte(self.a);
        (1, 3)
    }

    fn pla(&mut self) -> (u16, usize) {
        self.a = self.pop_byte();
        self.set_nz_flags(self.a);
        (1, 4)
    }

    fn branch(&mut self, opcode: u8) -> (u16, usize) {
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
            // PC is incremented after memory fetch.
            let offset = self.bus.borrow_mut().read_byte(self.pc + 1) as u16;
            self.pc += 2;
            let prev_pc = self.pc;
            self.pc = if offset & 0x80 > 0 {
                self.pc.wrapping_add(offset | 0xFF00)
            } else {
                self.pc.wrapping_add(offset)
            };

            if (prev_pc >> 8) != (self.pc >> 8) {
                (0, 5)
            } else {
                (0, 3)
            }
        } else {
            (2, 2)
        }
    }

    fn jsr(&mut self) -> (u16, usize) {
        // PC + 3'size of ins' - 1'RTS has size 1'
        self.push_word(self.pc + 2);
        self.pc = self.bus.borrow_mut().read_word(self.pc + 1);
        (0, 6)
    }

    fn bit(&mut self, opcode: u8) -> (u16, usize) {
        let addr_mode = AddressMode::from_code(opcode);
        let (address, _) = self.get_address(addr_mode);
        let operand = self.bus.borrow_mut().read_byte(address);
        let result = self.a & operand;

        self.p.set_z(result == 0);
        self.p.set_v(operand & 0x40 > 0);
        self.p.set_n(operand & 0x80 > 0);

        (addr_mode.byte_code_size() + 1, addr_mode.cycle_cost() + 1)
    }

    fn rti(&mut self) -> (u16, usize) {
        self.p.0 = self.pop_byte();
        self.p.set_b(0);
        self.pc = self.pop_word();
        (0, 6)
    }

    fn rts(&mut self) -> (u16, usize) {
        self.pc = self.pop_word();
        (1, 6)
    }

    fn jmp(&mut self, opcode: u8) -> (u16, usize) {
        let addr_mode = match opcode {
            0x4Cu8 => AddressMode::Absolute,
            0x6Cu8 => AddressMode::Indirect,
            _ => unreachable!(),
        };

        let (address, _) = self.get_address(addr_mode);
        self.pc = address;

        // Don't increment the PC so that jmps go direct.
        (0, addr_mode.cycle_cost())
    }

    fn sty(&mut self, opcode: u8) -> (u16, usize) {
        let addr_mode = AddressMode::from_code(opcode);
        let (address, _) = self.get_address(addr_mode);

        self.bus.borrow_mut().write_byte(address, self.y);

        (addr_mode.byte_code_size() + 1, addr_mode.cycle_cost() + 1)
    }

    fn dey(&mut self) -> (u16, usize) {
        self.y = self.y.wrapping_sub(1);
        self.set_nz_flags(self.y);
        (1, 2)
    }

    fn tya(&mut self) -> (u16, usize) {
        self.a = self.y;
        self.set_nz_flags(self.a);
        (1, 2)
    }

    fn tay(&mut self) -> (u16, usize) {
        self.y = self.a;
        self.set_nz_flags(self.y);
        (1, 2)
    }

    fn ldy(&mut self, opcode: u8) -> (u16, usize) {
        let addr_mode = match opcode {
            0xA0 => AddressMode::Immediate,
            _ => AddressMode::from_code(opcode),
        };
        let (address, page_cross) = self.get_address(addr_mode);

        self.y = self.bus.borrow_mut().read_byte(address);
        self.set_nz_flags(self.y);

        let mut cycles = addr_mode.cycle_cost() + 1;
        if page_cross {
            cycles += 1;
        }
        (addr_mode.byte_code_size() + 1, cycles)
    }

    fn cpy(&mut self, opcode: u8) -> (u16, usize) {
        let addr_mode = match opcode {
            0xC0 => AddressMode::Immediate,
            _ => AddressMode::from_code(opcode),
        };
        let (address, page_cross) = self.get_address(addr_mode);

        let operand = self.bus.borrow_mut().read_byte(address);
        let result = self.y.wrapping_sub(operand);
        self.p.set_c(self.y >= operand);
        self.set_nz_flags(result);

        let mut cycles = addr_mode.cycle_cost() + 1;
        if page_cross {
            cycles += 1;
        }
        (addr_mode.byte_code_size() + 1, cycles)
    }

    fn cpx(&mut self, opcode: u8) -> (u16, usize) {
        let addr_mode = match opcode {
            0xE0 => AddressMode::Immediate,
            _ => AddressMode::from_code(opcode),
        };
        let (address, page_cross) = self.get_address(addr_mode);

        let operand = self.bus.borrow_mut().read_byte(address);
        let result = self.x.wrapping_sub(operand);
        self.p.set_c(self.x >= operand);
        self.set_nz_flags(result);

        let mut cycles = addr_mode.cycle_cost() + 1;
        if page_cross {
            cycles += 1;
        }
        (addr_mode.byte_code_size() + 1, cycles)
    }

    fn iny(&mut self) -> (u16, usize) {
        self.y = self.y.wrapping_add(1);
        self.set_nz_flags(self.y);
        (1, 2)
    }

    fn inx(&mut self) -> (u16, usize) {
        self.x = self.x.wrapping_add(1);
        self.set_nz_flags(self.x);
        (1, 2)
    }
}
