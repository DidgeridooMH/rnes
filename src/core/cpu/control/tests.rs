use super::*;
use crate::core::{Addressable, Bus};
use std::{cell::RefCell, rc::Rc};

struct VectorMock;

impl Addressable for VectorMock {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xFFFEu16 => 0x00,
            0xFFFFu16 => 0x80,
            _ => 0xFF,
        }
    }

    fn write_byte(&mut self, _address: u16, _data: u8) {}
}

#[test]
fn test_break() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);
    bus.borrow_mut()
        .register_region(0xFFFEu16..=0xFFFFu16, Rc::new(RefCell::new(VectorMock)));

    cpu.p.0 = 0x80;
    cpu.pc = 0x4040u16;
    cpu.sp = 0xFFu8;
    cpu.brk().unwrap();

    let bus = bus.borrow();
    let pc_result = bus.read_word(0xFEu16).unwrap();
    let p_result = bus.read_byte(0xFDu16).unwrap();
    assert_eq!(cpu.pc, 0x8000u16);
    assert_eq!(pc_result, 0x4042u16);
    assert_eq!(p_result, 0xB0u8);
    assert_eq!(cpu.sp, 0xFCu8);
    assert!(cpu.p.i());
}

#[test]
fn test_php() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.p.0 = 0x80u8;
    cpu.sp = 0xFFu8;
    cpu.php().unwrap();

    let result = bus.borrow().read_byte(0xFFu16).unwrap();
    assert_eq!(result, 0xB0u8);
    assert_eq!(cpu.sp, 0xFEu8);
}

#[test]
fn test_plp() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.p.0 = 0x80u8;
    cpu.sp = 0xFEu8;
    bus.borrow_mut().write_byte(0xFFu16, 0x81u8).unwrap();
    cpu.plp().unwrap();

    assert_eq!(cpu.sp, 0xFFu8);
    assert_eq!(cpu.p.0, 0x81u8);
}

#[test]
fn test_pha() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.sp = 0xFFu8;
    cpu.a = 0x8Eu8;
    cpu.pha().unwrap();

    assert_eq!(cpu.sp, 0xFEu8);
}

#[test]
fn test_pla_zero() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    bus.borrow_mut().write_byte(0xFFu16, 0).unwrap();
    cpu.sp = 0xFEu8;
    cpu.pla().unwrap();

    assert_eq!(cpu.sp, 0xFFu8);
    assert!(cpu.p.z());
    assert!(!cpu.p.n());
}

#[test]
fn test_pla_positive() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    bus.borrow_mut().write_byte(0xFFu16, 1).unwrap();
    cpu.sp = 0xFEu8;
    cpu.pla().unwrap();

    assert_eq!(cpu.sp, 0xFFu8);
    assert!(!cpu.p.z());
    assert!(!cpu.p.n());
}

#[test]
fn test_pla_negative() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    bus.borrow_mut().write_byte(0xFFu16, 0x80).unwrap();
    cpu.sp = 0xFEu8;
    cpu.pla().unwrap();

    assert_eq!(cpu.sp, 0xFFu8);
    assert!(!cpu.p.z());
    assert!(cpu.p.n());
}
