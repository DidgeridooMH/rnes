use crate::core::Addressable;

#[derive(Copy, Clone, Default)]
pub struct Controller {
    buttons: u8,
}

impl Addressable for Controller {
    fn read_byte(&mut self, address: u16) -> u8 {
        if address == 0x4016 || address == 0x4017 {
            /*            let output = self.buttons & 1;
            self.buttons >>= 1;
            output*/
            0
        } else {
            eprintln!("Unexpected read from controller address {address}");
            0
        }
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        if address == 0x4016 && (data & 1) > 0 {
            self.buttons = 8;
        }
    }
}
