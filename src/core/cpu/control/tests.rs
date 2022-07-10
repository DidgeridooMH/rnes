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

fn setup() -> (Rc<RefCell<Bus>>, CPU) {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);
    (bus, cpu)
}

#[test]
fn test_break() {
    let (bus, mut cpu) = setup();
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
    let (bus, mut cpu) = setup();

    cpu.p.0 = 0x80u8;
    cpu.sp = 0xFFu8;
    cpu.php().unwrap();

    let result = bus.borrow().read_byte(0xFFu16).unwrap();
    assert_eq!(result, 0xB0u8);
    assert_eq!(cpu.sp, 0xFEu8);
}

#[test]
fn test_plp() {
    let (bus, mut cpu) = setup();

    cpu.p.0 = 0x80u8;
    cpu.sp = 0xFEu8;
    bus.borrow_mut().write_byte(0xFFu16, 0x81u8).unwrap();
    cpu.plp().unwrap();

    assert_eq!(cpu.sp, 0xFFu8);
    assert_eq!(cpu.p.0, 0x81u8);
}

#[test]
fn test_pha() {
    let (_, mut cpu) = setup();

    cpu.sp = 0xFFu8;
    cpu.a = 0x8Eu8;
    cpu.pha().unwrap();

    assert_eq!(cpu.sp, 0xFEu8);
}

#[test]
fn test_pla_zero() {
    let (bus, mut cpu) = setup();

    bus.borrow_mut().write_byte(0xFFu16, 0).unwrap();
    cpu.sp = 0xFEu8;
    cpu.pla().unwrap();

    assert_eq!(cpu.sp, 0xFFu8);
    assert!(cpu.p.z());
    assert!(!cpu.p.n());
}

#[test]
fn test_pla_positive() {
    let (bus, mut cpu) = setup();

    bus.borrow_mut().write_byte(0xFFu16, 1).unwrap();
    cpu.sp = 0xFEu8;
    cpu.pla().unwrap();

    assert_eq!(cpu.sp, 0xFFu8);
    assert!(!cpu.p.z());
    assert!(!cpu.p.n());
}

#[test]
fn test_pla_negative() {
    let (bus, mut cpu) = setup();

    bus.borrow_mut().write_byte(0xFFu16, 0x80).unwrap();
    cpu.sp = 0xFEu8;
    cpu.pla().unwrap();

    assert_eq!(cpu.sp, 0xFFu8);
    assert!(!cpu.p.z());
    assert!(cpu.p.n());
}

#[test]
fn test_bpl_take_branch_change_page() {
    let (bus, mut cpu) = setup();

    bus.borrow_mut().write_byte(0xFFu16, 0x3u8).unwrap();
    cpu.pc = 0xFE;

    let (_, cycles) = cpu.branch(0x10u8).unwrap();

    assert_eq!(cpu.pc, 0x101u16);
    assert_eq!(cycles, 5);
}

#[test]
fn test_bpl_take_branch() {
    let (bus, mut cpu) = setup();

    bus.borrow_mut().write_byte(0x1u16, 0x3u8).unwrap();
    cpu.pc = 0x0;

    let (_, cycles) = cpu.branch(0x10u8).unwrap();

    assert_eq!(cpu.pc, 0x3u16);
    assert_eq!(cycles, 3);
}

#[test]
fn test_bpl_miss_branch() {
    let (bus, mut cpu) = setup();

    bus.borrow_mut().write_byte(0x1u16, 0x3u8).unwrap();
    cpu.pc = 0x0;
    cpu.p.set_n(true);

    let (_, cycles) = cpu.branch(0x10u8).unwrap();

    assert_eq!(cpu.pc, 0x0u16);
    assert_eq!(cycles, 2);
}
#[test]
fn test_bmi_take_branch() {
    let (bus, mut cpu) = setup();

    bus.borrow_mut().write_byte(0x1u16, 0x3u8).unwrap();
    cpu.pc = 0x0;
    cpu.p.set_n(true);

    let (_, cycles) = cpu.branch(0x30u8).unwrap();

    assert_eq!(cpu.pc, 0x3u16);
    assert_eq!(cycles, 3);
}

#[test]
fn test_bmi_miss_branch() {
    let (bus, mut cpu) = setup();

    bus.borrow_mut().write_byte(0x1u16, 0x3u8).unwrap();
    cpu.pc = 0x0;

    let (_, cycles) = cpu.branch(0x30u8).unwrap();

    assert_eq!(cpu.pc, 0x0u16);
    assert_eq!(cycles, 2);
}

#[test]
fn test_bvc_take_branch() {
    let (bus, mut cpu) = setup();

    bus.borrow_mut().write_byte(0x1u16, 0x3u8).unwrap();
    cpu.pc = 0x0;

    let (_, cycles) = cpu.branch(0x50u8).unwrap();

    assert_eq!(cpu.pc, 0x3u16);
    assert_eq!(cycles, 3);
}

#[test]
fn test_bvc_miss_branch() {
    let (bus, mut cpu) = setup();

    bus.borrow_mut().write_byte(0x1u16, 0x3u8).unwrap();
    cpu.pc = 0x0;
    cpu.p.set_v(true);

    let (_, cycles) = cpu.branch(0x50u8).unwrap();

    assert_eq!(cpu.pc, 0x0u16);
    assert_eq!(cycles, 2);
}
#[test]
fn test_bvs_take_branch() {
    let (bus, mut cpu) = setup();

    bus.borrow_mut().write_byte(0x1u16, 0x3u8).unwrap();
    cpu.pc = 0x0;
    cpu.p.set_v(true);

    let (_, cycles) = cpu.branch(0x70u8).unwrap();

    assert_eq!(cpu.pc, 0x3u16);
    assert_eq!(cycles, 3);
}

#[test]
fn test_bvs_miss_branch() {
    let (bus, mut cpu) = setup();

    bus.borrow_mut().write_byte(0x1u16, 0x3u8).unwrap();
    cpu.pc = 0x0;

    let (_, cycles) = cpu.branch(0x70u8).unwrap();

    assert_eq!(cpu.pc, 0x0u16);
    assert_eq!(cycles, 2);
}

#[test]
fn test_bcc_take_branch() {
    let (bus, mut cpu) = setup();

    bus.borrow_mut().write_byte(0x1u16, 0x3u8).unwrap();
    cpu.pc = 0x0;

    let (_, cycles) = cpu.branch(0x90u8).unwrap();

    assert_eq!(cpu.pc, 0x3u16);
    assert_eq!(cycles, 3);
}

#[test]
fn test_bcc_miss_branch() {
    let (bus, mut cpu) = setup();

    bus.borrow_mut().write_byte(0x1u16, 0x3u8).unwrap();
    cpu.pc = 0x0;
    cpu.p.set_c(true);

    let (_, cycles) = cpu.branch(0x90u8).unwrap();

    assert_eq!(cpu.pc, 0x0u16);
    assert_eq!(cycles, 2);
}
#[test]
fn test_bcs_take_branch() {
    let (bus, mut cpu) = setup();

    bus.borrow_mut().write_byte(0x1u16, 0x3u8).unwrap();
    cpu.pc = 0x0;
    cpu.p.set_c(true);

    let (_, cycles) = cpu.branch(0xB0u8).unwrap();

    assert_eq!(cpu.pc, 0x3u16);
    assert_eq!(cycles, 3);
}

#[test]
fn test_bcs_miss_branch() {
    let (bus, mut cpu) = setup();

    bus.borrow_mut().write_byte(0x1u16, 0x3u8).unwrap();
    cpu.pc = 0x0;

    let (_, cycles) = cpu.branch(0xB0u8).unwrap();

    assert_eq!(cpu.pc, 0x0u16);
    assert_eq!(cycles, 2);
}

#[test]
fn test_bne_take_branch() {
    let (bus, mut cpu) = setup();

    bus.borrow_mut().write_byte(0x1u16, 0x3u8).unwrap();
    cpu.pc = 0x0;

    let (_, cycles) = cpu.branch(0xD0u8).unwrap();

    assert_eq!(cpu.pc, 0x3u16);
    assert_eq!(cycles, 3);
}

#[test]
fn test_bne_miss_branch() {
    let (bus, mut cpu) = setup();

    bus.borrow_mut().write_byte(0x1u16, 0x3u8).unwrap();
    cpu.pc = 0x0;
    cpu.p.set_z(true);

    let (_, cycles) = cpu.branch(0xD0u8).unwrap();

    assert_eq!(cpu.pc, 0x0u16);
    assert_eq!(cycles, 2);
}
#[test]
fn test_beq_take_branch() {
    let (bus, mut cpu) = setup();

    bus.borrow_mut().write_byte(0x1u16, 0x3u8).unwrap();
    cpu.pc = 0x0;
    cpu.p.set_z(true);

    let (_, cycles) = cpu.branch(0xF0u8).unwrap();

    assert_eq!(cpu.pc, 0x3u16);
    assert_eq!(cycles, 3);
}

#[test]
fn test_beq_miss_branch() {
    let (bus, mut cpu) = setup();

    bus.borrow_mut().write_byte(0x1u16, 0x3u8).unwrap();
    cpu.pc = 0x0;

    let (_, cycles) = cpu.branch(0xF0u8).unwrap();

    assert_eq!(cpu.pc, 0x0u16);
    assert_eq!(cycles, 2);
}

#[test]
fn test_negative_branch() {
    let (bus, mut cpu) = setup();

    bus.borrow_mut().write_byte(0x11u16, 0xFEu8).unwrap();
    cpu.pc = 0x10;

    let (_, cycles) = cpu.branch(0x10u8).unwrap();

    assert_eq!(cpu.pc, 0xEu16);
    assert_eq!(cycles, 3);
}

#[test]
fn test_jsr() {
    let (bus, mut cpu) = setup();

    bus.borrow_mut().write_word(0x9u16, 0xDEADu16).unwrap();
    cpu.pc = 8u16;
    cpu.sp = 0xFFu8;

    cpu.jsr().unwrap();

    let result = bus.borrow().read_word(0xFEu16).unwrap();
    assert_eq!(result, 10u16);
    assert_eq!(cpu.sp, 0xFDu8);
    assert_eq!(cpu.pc, 0xDEADu16);
}

#[test]
fn test_bit_zero_page() {
    let (bus, mut cpu) = setup();

    {
        let mut bus = bus.borrow_mut();
        bus.write_byte(0x80u16, 0x80u8).unwrap();
        bus.write_byte(0x1u16, 0x80u8).unwrap();
    }

    cpu.pc = 0x0u16;
    cpu.a = 0x81u8;
    cpu.bit(0x24u8).unwrap();

    assert_eq!(cpu.a, 0x81u8);
    assert_eq!(cpu.p.z(), false);
    assert_eq!(cpu.p.v(), false);
    assert_eq!(cpu.p.n(), true);
}

#[test]
fn test_bit_absolute() {
    let (bus, mut cpu) = setup();

    {
        let mut bus = bus.borrow_mut();
        bus.write_word(0x1u16, 0x715u16).unwrap();
        bus.write_byte(0x715u16, 0x80u8).unwrap();
    }

    cpu.pc = 0x0u16;
    cpu.a = 0x81u8;
    cpu.bit(0x2Cu8).unwrap();

    assert_eq!(cpu.a, 0x81u8);
    assert_eq!(cpu.p.z(), false);
    assert_eq!(cpu.p.v(), false);
    assert_eq!(cpu.p.n(), true);
}

#[test]
fn test_bit_zero() {
    let (bus, mut cpu) = setup();

    {
        let mut bus = bus.borrow_mut();
        bus.write_word(0x1u16, 0x715u16).unwrap();
        bus.write_byte(0x715u16, 0x80u8).unwrap();
    }

    cpu.pc = 0x0u16;
    cpu.a = 0x1u8;
    cpu.bit(0x2Cu8).unwrap();

    assert_eq!(cpu.a, 0x1u8);
    assert_eq!(cpu.p.z(), true);
    assert_eq!(cpu.p.v(), false);
    assert_eq!(cpu.p.n(), false);
}

#[test]
fn test_bit_overflow() {
    let (bus, mut cpu) = setup();

    {
        let mut bus = bus.borrow_mut();
        bus.write_word(0x1u16, 0x715u16).unwrap();
        bus.write_byte(0x715u16, 0x40u8).unwrap();
    }

    cpu.pc = 0x0u16;
    cpu.a = 0x41u8;
    cpu.bit(0x2Cu8).unwrap();

    assert_eq!(cpu.a, 0x41u8);
    assert_eq!(cpu.p.z(), false);
    assert_eq!(cpu.p.v(), true);
    assert_eq!(cpu.p.n(), false);
}

#[test]
fn test_rti() {
    let (_, mut cpu) = setup();

    cpu.push_word(0x500u16).unwrap();
    cpu.push_byte(0x80u8).unwrap();

    cpu.rti().unwrap();

    assert_eq!(cpu.p.0, 0x80u8);
    assert_eq!(cpu.pc, 0x500u16);
}

#[test]
fn test_jmp_absolute() {
    let (bus, mut cpu) = setup();

    bus.borrow_mut().write_word(0x1u16, 0xAABBu16).unwrap();
    cpu.pc = 0;
    cpu.jmp(0x4Cu8).unwrap();

    assert_eq!(cpu.pc, 0xAABBu16);
}

#[test]
fn test_jmp_indirect() {
    let (bus, mut cpu) = setup();

    {
        let mut bus = bus.borrow_mut();
        bus.write_word(0x1u16, 0x716u16).unwrap();
        bus.write_word(0x716u16, 0xAABBu16).unwrap();
    }

    cpu.pc = 0;
    cpu.jmp(0x6Cu8).unwrap();

    assert_eq!(cpu.pc, 0xAABBu16);
}

#[test]
fn test_jmp_indirect_page_bug() {
    let (bus, mut cpu) = setup();

    {
        let mut bus = bus.borrow_mut();
        bus.write_word(0x1u16, 0x6FFu16).unwrap();
        bus.write_byte(0x6FFu16, 0xBBu8).unwrap();
        bus.write_byte(0x600u16, 0xAAu8).unwrap();
    }

    cpu.pc = 0;
    cpu.jmp(0x6Cu8).unwrap();

    assert_eq!(cpu.pc, 0xAABBu16);
}
