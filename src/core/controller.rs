use crate::core::Addressable;

#[derive(Copy, Clone, Default)]
pub struct Controller {
    strobe_latch: bool,
}

impl Addressable for Controller {
    fn read_byte(&mut self, address: u16) -> u8 {
        if address == 0x4016 || address == 0x4017 {
            0
        } else {
            eprintln!("Unexpected read from controller address {address}");
            0
        }
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        if address == 0x4016 {
            self.strobe_latch = data & 1 > 0;
        }
    }
}
