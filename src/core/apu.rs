use crate::core::Addressable;

pub struct APU {}

impl Addressable for APU {
    fn read_byte(&mut self, _address: u16) -> u8 {
        0
    }

    fn write_byte(&mut self, _address: u16, _data: u8) {}
}
