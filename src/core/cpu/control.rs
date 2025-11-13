use super::{AddressMode, CoreError, StatusRegister, CPU};

// #[cfg(test)]
// mod tests;

impl CPU {
    pub fn run_control_op(&mut self, opcode: u8) -> Result<usize, CoreError> {
        let address_mode = AddressMode::from_code(opcode);
        let operand = match opcode {
            0x10 | 0x30 | 0x50 | 0x70 | 0x90 | 0xB0 | 0xD0 | 0xF0 | 0x24 | 0x2C | 0xA0 | 0xA4
            | 0xB4 | 0xAC | 0xBC | 0xC0 | 0xC4 | 0xCC | 0xE0 | 0xE4 | 0xEC => {
                self.read_operand(address_mode).0
            }
            _ => 0,
        };

        let cycles = match opcode {
            0 => self.brk(),
            0x8 => self.php(),
            0x28 => self.plp(),
            0x48 => self.pha(),
            0x68 => self.pla(),
            0x10 => self.branch(operand, !self.p.n()),
            0x30 => self.branch(operand, self.p.n()),
            0x50 => self.branch(operand, !self.p.v()),
            0x70 => self.branch(operand, self.p.v()),
            0x90 => self.branch(operand, !self.p.c()),
            0xB0 => self.branch(operand, self.p.c()),
            0xD0 => self.branch(operand, !self.p.z()),
            0xF0 => self.branch(operand, self.p.z()),
            0x18 => {
                self.p.set_c(false);
                2
            }
            0x38 => {
                self.p.set_c(true);
                2
            }
            0x58 => {
                self.p.set_i(false);
                2
            }
            0x78 => {
                self.p.set_i(true);
                2
            }
            0xB8 => {
                self.p.set_v(false);
                2
            }
            0xD8 => {
                self.p.set_d(false);
                2
            }
            0xF8 => {
                self.p.set_d(true);
                2
            }
            0x20 => self.jsr(),
            0x24 | 0x2C => self.bit(operand),
            0x40 => self.rti(),
            0x60 => self.rts(),
            0x4C | 0x6C => self.jmp(address_mode),
            0x84 | 0x94 | 0x8C => self.sty(address_mode),
            0x88 => self.dey(),
            0x98 => self.tya(),
            0xA8 => self.tay(),
            0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => self.ldy(operand),
            0xC0 | 0xC4 | 0xCC => self.cpy(operand),
            0xE0 | 0xE4 | 0xEC => self.cpx(operand),
            0xC8 => self.iny(),
            0xE8 => self.inx(),
            0x04 | 0x0C | 0x14 | 0x1C | 0x34 | 0x3C | 0x44 | 0x54 | 0x5C | 0x64 | 0x74 | 0x7C
            | 0x80 | 0xD4 | 0xDC | 0xF4 | 0xFC => self.nop(address_mode),
            0x9C => self.shy(address_mode),
            _ => return Err(CoreError::OpcodeNotImplemented(opcode)),
        };

        match opcode {
            0x00 | 0x20 | 0x4C | 0x6C | 0x40 => {}
            _ => self.pc += 1 + address_mode.byte_code_size(),
        }

        Ok(cycles + address_mode.cycle_cost(false))
    }

    fn brk(&mut self) -> usize {
        self.push_word(self.pc + 2);
        self.php();
        self.pc = self.bus.borrow_mut().read_word(0xFFFEu16);
        self.p.set_i(true);
        7
    }

    fn php(&mut self) -> usize {
        let mut status = self.p;
        status.set_b(3u8);
        self.push_byte(status.0);
        4
    }

    fn plp(&mut self) -> usize {
        let mut status = StatusRegister(self.pop_byte());
        status.set_b(0u8);
        self.p = status;
        4
    }

    fn pha(&mut self) -> usize {
        self.push_byte(self.a);
        4
    }

    fn pla(&mut self) -> usize {
        self.a = self.pop_byte();
        self.set_nz_flags(self.a);
        4
    }

    fn branch(&mut self, operand: u8, should_branch: bool) -> usize {
        if !should_branch {
            return 2;
        }

        let offset = operand as u16;
        let prev_pc = self.pc + 1;
        self.pc = if offset >= 0x80 {
            self.pc.wrapping_add(offset | 0xFF00)
        } else {
            self.pc.wrapping_add(offset)
        };

        if (prev_pc & 0xFF00) != (self.pc & 0xFF00) {
            3
        } else {
            4
        }
    }

    fn jsr(&mut self) -> usize {
        // PC + 3'size of ins' - 1'RTS has size 1'
        self.push_word(self.pc + 2);
        self.pc = self.bus.borrow_mut().read_word(self.pc + 1);
        3
    }

    fn bit(&mut self, operand: u8) -> usize {
        self.p.set_z(self.a & operand == 0);
        self.p.set_v(operand & 0x40 > 0);
        self.p.set_n(operand & 0x80 > 0);
        1
    }

    fn rti(&mut self) -> usize {
        self.p.0 = self.pop_byte();
        self.p.set_b(0);
        self.pc = self.pop_word();
        6
    }

    fn rts(&mut self) -> usize {
        self.pc = self.pop_word();
        6
    }

    fn jmp(&mut self, address_mode: AddressMode) -> usize {
        self.pc = self.get_address(address_mode).0;
        0
    }

    fn sty(&mut self, address_mode: AddressMode) -> usize {
        self.write_operand(self.y, address_mode);
        1
    }

    fn dey(&mut self) -> usize {
        self.y = self.y.wrapping_sub(1);
        self.set_nz_flags(self.y);
        2
    }

    fn tya(&mut self) -> usize {
        self.a = self.y;
        self.set_nz_flags(self.a);
        2
    }

    fn tay(&mut self) -> usize {
        self.y = self.a;
        self.set_nz_flags(self.y);
        2
    }

    fn ldy(&mut self, operand: u8) -> usize {
        self.y = operand;
        self.set_nz_flags(self.y);
        1
    }

    fn cpy(&mut self, operand: u8) -> usize {
        let result = self.y.wrapping_sub(operand);
        self.p.set_c(self.y >= operand);
        self.set_nz_flags(result);
        1
    }

    fn cpx(&mut self, operand: u8) -> usize {
        let result = self.x.wrapping_sub(operand);
        self.p.set_c(self.x >= operand);
        self.set_nz_flags(result);
        1
    }

    fn iny(&mut self) -> usize {
        self.y = self.y.wrapping_add(1);
        self.set_nz_flags(self.y);
        2
    }

    fn inx(&mut self) -> usize {
        self.x = self.x.wrapping_add(1);
        self.set_nz_flags(self.x);
        2
    }
}
