use std::cell::RefCell;
use std::fmt;
use std::ops::Range;
use std::rc::Rc;

#[cfg(test)]
mod tests;

pub trait Addressable {
    fn read_byte(&self, address: u16) -> u8;
    fn write_byte(&mut self, address: u16, data: u8);
}

pub type AddressRegion = Range<u16>;

pub struct MemoryMapping {
    region: AddressRegion,
    component: Rc<RefCell<dyn Addressable>>,
}

#[derive(Debug, Clone)]
pub struct RegionError {
    address: u16,
}

impl fmt::Display for RegionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Address access violation at 0x{:04X}", self.address)
    }
}

pub struct Bus {
    regions: Vec<MemoryMapping>,
}

impl Bus {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            regions: Vec::new(),
        }))
    }

    pub fn register_region(
        &mut self,
        region: AddressRegion,
        component: Rc<RefCell<dyn Addressable>>,
    ) {
        self.regions.push(MemoryMapping { region, component });
    }

    pub fn read_byte(&self, address: u16) -> Result<u8, RegionError> {
        for mapping in &self.regions {
            if mapping.region.contains(&address) {
                return Ok(mapping.component.borrow().read_byte(address));
            }
        }
        Err(RegionError { address })
    }

    pub fn read_word(&self, address: u16) -> Result<u16, RegionError> {
        let low_byte = self.read_byte(address)? as u16;
        let high_byte = self.read_byte(address + 1)? as u16;
        Ok(low_byte | (high_byte << 8))
    }

    pub fn read_word_bug(&self, address: u16) -> Result<u16, RegionError> {
        let low_byte = self.read_byte(address)? as u16;
        let high_byte = self.read_byte((address & 0xFF00) | ((address + 1) & 0xFF))? as u16;
        Ok(low_byte | (high_byte << 8))
    }

    pub fn write_byte(&mut self, address: u16, data: u8) -> Result<(), RegionError> {
        for mapping in &self.regions {
            if mapping.region.contains(&address) {
                mapping.component.borrow_mut().write_byte(address, data);
                return Ok(());
            }
        }
        Err(RegionError { address })
    }

    pub fn write_word(&mut self, address: u16, data: u16) -> Result<(), RegionError> {
        self.write_byte(address, data as u8)?;
        self.write_byte(address + 1, (data >> 8) as u8)?;
        Ok(())
    }
}
