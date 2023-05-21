use super::Addressable;
use std::{cell::RefCell, rc::Rc};

#[cfg(test)]
mod tests;

const CPU_INTERNAL_RAM_SIZE: usize = 0x800;

pub struct InternalRam {
    data: [u8; CPU_INTERNAL_RAM_SIZE],
}

impl InternalRam {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            data: [0; CPU_INTERNAL_RAM_SIZE],
        }))
    }
}

impl Addressable for InternalRam {
    fn read_byte(&mut self, address: u16) -> u8 {
        self.data[(address % CPU_INTERNAL_RAM_SIZE as u16) as usize]
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        self.data[(address % CPU_INTERNAL_RAM_SIZE as u16) as usize] = data
    }
}
