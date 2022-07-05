use rnes::core::{cpu::CPU, Addressable, Bus};
use std::{cell::RefCell, rc::Rc};

struct TestCartridge {
    program: Vec<u8>,
}

impl Addressable for TestCartridge {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xFFFCu16 => 0x00,
            0xFFFDu16 => 0x80,
            _ => self.program[(address - 0x8000u16) as usize % self.program.len()],
        }
    }

    fn write_byte(&mut self, _address: u16, _data: u8) {}
}

#[test]
fn test_cpu_alu() {
    let program = vec![
        0xA9u8, 0x12u8, 0x09u8, 0x1u8, 0x49u8, 0x1u8, 0x8Du8, 0xF0u8, 0x7u8, 0x69u8, 0x23u8,
        0xe9u8, 0x5u8, 0x29u8, 0x10u8, 0xc9u8, 0x10u8,
    ];

    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);
    let test_cart = Rc::new(RefCell::new(TestCartridge { program }));

    bus.borrow_mut()
        .register_region(0x8000u16..=0xFFFFu16, test_cart);

    for _ in 0..8 {
        cpu.tick().unwrap();
    }

    assert_eq!(cpu.a, 0x10);
    assert_eq!(cpu.p.c(), true);
}
