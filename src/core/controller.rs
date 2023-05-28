use winit::event::{ElementState, VirtualKeyCode};

use crate::core::Addressable;

#[derive(Copy, Clone, Default)]
pub struct Controller {
    buttons: u8,
    current_buttons: u8,
}

const KEYMAPPING: [VirtualKeyCode; 8] = [
    VirtualKeyCode::K,
    VirtualKeyCode::J,
    VirtualKeyCode::Comma,
    VirtualKeyCode::Period,
    VirtualKeyCode::W,
    VirtualKeyCode::S,
    VirtualKeyCode::A,
    VirtualKeyCode::D,
];

impl Controller {
    pub fn input(&mut self, keycode: &VirtualKeyCode, state: &ElementState) {
        if let Some(position) = KEYMAPPING.iter().position(|k| k == keycode) {
            match state {
                ElementState::Pressed => self.current_buttons |= 1 << position,
                ElementState::Released => self.current_buttons &= !(1 << position),
            }
        }
    }
}

impl Addressable for Controller {
    fn read_byte(&mut self, address: u16) -> u8 {
        if address == 0x4016 || address == 0x4017 {
            let output = self.buttons & 1;
            self.buttons >>= 1;
            output
        } else {
            eprintln!("Unexpected read from controller address {address}");
            0
        }
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        if address == 0x4016 && (data & 1) > 0 {
            self.buttons = self.current_buttons;
        }
    }
}
