use gilrs::ev::Button;
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

fn button_bit(button: Button) -> Option<usize> {
    match button {
        Button::South => Some(0),
        Button::East => Some(1),
        Button::Select => Some(2),
        Button::Start => Some(3),
        Button::DPadUp => Some(4),
        Button::DPadDown => Some(5),
        Button::DPadLeft => Some(6),
        Button::DPadRight => Some(7),
        _ => None,
    }
}

impl Controller {
    pub fn input_keyboard(&mut self, keycode: &VirtualKeyCode, state: &ElementState) {
        if let Some(position) = KEYMAPPING.iter().position(|k| k == keycode) {
            match state {
                ElementState::Pressed => self.current_buttons |= 1 << position,
                ElementState::Released => self.current_buttons &= !(1 << position),
            }
        }
    }

    pub fn gamepad_press(&mut self, button: Button) {
        if let Some(position) = button_bit(button) {
            self.current_buttons |= 1 << position;
        }
    }

    pub fn gamepad_release(&mut self, button: Button) {
        if let Some(position) = button_bit(button) {
            self.current_buttons &= !(1 << position);
        }
    }
}

impl Addressable for Controller {
    fn read_byte(&mut self, address: u16) -> Option<u8> {
        if address == 0x4016 || address == 0x4017 {
            let output = self.buttons & 1;
            self.buttons >>= 1;
            Some(output)
        } else {
            eprintln!("Unexpected read from controller address {address}");
            None
        }
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        if address == 0x4016 && (data & 1) > 0 {
            self.buttons = self.current_buttons;
        }
    }
}
