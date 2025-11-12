use std::cell::RefCell;
use std::fmt::Display;
use std::ops::RangeInclusive;
use std::rc::Rc;

// #[cfg(test)]
// mod tests;

pub trait Addressable {
    fn read_byte(&mut self, address: u16) -> Option<u8>;
    fn write_byte(&mut self, address: u16, data: u8);
}

pub struct MemoryMapping {
    region: RangeInclusive<u16>,
    component: Rc<RefCell<dyn Addressable>>,
}

pub struct Bus {
    regions: Vec<MemoryMapping>,
    last_read: u8,
}

impl Display for Bus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bus mappings: {{ ")?;
        for r in &self.regions {
            write!(f, "{:?}, ", r.region)?;
        }
        write!(f, " }}")?;

        Ok(())
    }
}

impl Bus {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            regions: Vec::new(),
            last_read: 0,
        }))
    }

    pub fn register_region(
        &mut self,
        region: RangeInclusive<u16>,
        component: Rc<RefCell<dyn Addressable>>,
    ) {
        self.regions.push(MemoryMapping { region, component });
    }

    pub fn read_byte(&mut self, address: u16) -> u8 {
        for mapping in &self.regions {
            if mapping.region.contains(&address) {
                return mapping
                    .component
                    .borrow_mut()
                    .read_byte(address)
                    .unwrap_or(self.last_read);
            }
        }

        self.last_read
    }

    pub fn read_word(&mut self, address: u16) -> u16 {
        let low_byte = self.read_byte(address) as u16;
        let high_byte = self.read_byte(address + 1) as u16;
        low_byte | (high_byte << 8)
    }

    pub fn read_word_bug(&mut self, address: u16) -> u16 {
        let low_byte = self.read_byte(address) as u16;
        let high_byte = self.read_byte((address & 0xFF00) | ((address + 1) & 0xFF)) as u16;
        low_byte | (high_byte << 8)
    }

    pub fn write_byte(&mut self, address: u16, data: u8) {
        for mapping in &self.regions {
            if mapping.region.contains(&address) {
                mapping.component.borrow_mut().write_byte(address, data);
            }
        }
    }

    pub fn write_word(&mut self, address: u16, data: u16) {
        self.write_byte(address, data as u8);
        self.write_byte(address + 1, (data >> 8) as u8);
    }
}
