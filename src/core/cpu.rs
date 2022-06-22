use super::{Addressable, Bus};
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(test)]
mod tests;

const CPU_INTERNAL_RAM_SIZE: usize = 0x800;

pub struct CPU {
    _bus: Rc<RefCell<Bus>>,
    internal_ram: [u8; CPU_INTERNAL_RAM_SIZE],
}

impl CPU {
    pub fn new(bus: &Rc<RefCell<Bus>>) -> Rc<RefCell<Self>> {
        let cpu = Rc::new(RefCell::new(Self {
            _bus: bus.clone(),
            internal_ram: [0; CPU_INTERNAL_RAM_SIZE],
        }));
        bus.borrow_mut()
            .register_region(0x0u16..0x2000u16, cpu.clone());
        cpu
    }
}

impl Addressable for CPU {
    fn read_byte(&self, address: u16) -> u8 {
        self.internal_ram[(address % CPU_INTERNAL_RAM_SIZE as u16) as usize]
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        self.internal_ram[(address % CPU_INTERNAL_RAM_SIZE as u16) as usize] = data
    }
}
