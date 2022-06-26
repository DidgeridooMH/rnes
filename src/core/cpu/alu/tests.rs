use super::*;
use crate::core::Bus;

#[test]
fn test_ora_immediate() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    let mut cpu = cpu.borrow_mut();
    cpu.bus.borrow_mut().write_byte(0x1u16, 0x80u8).unwrap();
    cpu.a = 1u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x09u8).unwrap();

    assert_eq!(cpu.a, 0x81u8);
    assert_eq!(cpu.p.n(), true);
    assert_eq!(cpu.p.z(), false);
}

#[test]
fn test_ora_zero_flag() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    let mut cpu = cpu.borrow_mut();
    cpu.pc = 0u16;
    cpu.run_alu_op(0x09u8).unwrap();

    assert_eq!(cpu.a, 0x0u8);
    assert_eq!(cpu.p.z(), true);
    assert_eq!(cpu.p.n(), false);
}

#[test]
fn test_ora_zero_page() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    let mut cpu = cpu.borrow_mut();
    cpu.bus.borrow_mut().write_byte(0x1u16, 0x80u8).unwrap();
    cpu.bus.borrow_mut().write_byte(0x80u16, 0xAAu8).unwrap();
    cpu.a = 1u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x05u8).unwrap();

    assert_eq!(cpu.a, 0xABu8);
    assert_eq!(cpu.p.n(), true);
    assert_eq!(cpu.p.z(), false);
}

#[test]
fn test_ora_zero_page_x() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    let mut cpu = cpu.borrow_mut();
    cpu.bus.borrow_mut().write_byte(0x1u16, 0x80u8).unwrap();
    cpu.bus.borrow_mut().write_byte(0x85u16, 0xAAu8).unwrap();
    cpu.a = 1u8;
    cpu.x = 5u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x15u8).unwrap();

    assert_eq!(cpu.a, 0xABu8);
    assert_eq!(cpu.p.n(), true);
    assert_eq!(cpu.p.z(), false);
}

#[test]
fn test_ora_absolute() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    let mut cpu = cpu.borrow_mut();
    cpu.bus.borrow_mut().write_word(0x1u16, 0x100u16).unwrap();
    cpu.bus.borrow_mut().write_byte(0x100u16, 0xC0u8).unwrap();
    cpu.a = 3u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x0Du8).unwrap();

    assert_eq!(cpu.a, 0xC3u8);
    assert_eq!(cpu.p.n(), true);
    assert_eq!(cpu.p.z(), false);
}

#[test]
fn test_ora_absolute_x() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    let mut cpu = cpu.borrow_mut();
    cpu.bus.borrow_mut().write_word(0x1u16, 0x100u16).unwrap();
    cpu.bus.borrow_mut().write_byte(0x105u16, 0xC0u8).unwrap();
    cpu.a = 3u8;
    cpu.x = 5u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x1Du8).unwrap();

    assert_eq!(cpu.a, 0xC3u8);
    assert_eq!(cpu.p.n(), true);
    assert_eq!(cpu.p.z(), false);
}

#[test]
fn test_ora_absolute_y() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    let mut cpu = cpu.borrow_mut();
    cpu.bus.borrow_mut().write_word(0x1u16, 0x100u16).unwrap();
    cpu.bus.borrow_mut().write_byte(0x105u16, 0xC0u8).unwrap();
    cpu.a = 3u8;
    cpu.y = 5u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x19u8).unwrap();

    assert_eq!(cpu.a, 0xC3u8);
    assert_eq!(cpu.p.n(), true);
    assert_eq!(cpu.p.z(), false);
}

#[test]
fn test_ora_indirect_x() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    let mut cpu = cpu.borrow_mut();
    {
        let mut bus = cpu.bus.borrow_mut();
        bus.write_byte(1u16, 0x80u8).unwrap();
        bus.write_word(0x85u16, 0x100u16).unwrap();
        bus.write_byte(0x100u16, 0xC0u8).unwrap();
    }
    cpu.a = 3u8;
    cpu.x = 5u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x01u8).unwrap();

    assert_eq!(cpu.a, 0xC3u8);
    assert_eq!(cpu.p.n(), true);
    assert_eq!(cpu.p.z(), false);
}

#[test]
fn test_ora_indirect_y() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    let mut cpu = cpu.borrow_mut();
    {
        let mut bus = cpu.bus.borrow_mut();
        bus.write_byte(1u16, 0x80u8).unwrap();
        bus.write_word(0x80u16, 0x100u16).unwrap();
        bus.write_byte(0x105u16, 0xC0u8).unwrap();
    }
    cpu.a = 3u8;
    cpu.y = 5u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x11u8).unwrap();

    assert_eq!(cpu.a, 0xC3u8);
    assert_eq!(cpu.p.n(), true);
    assert_eq!(cpu.p.z(), false);
}

#[test]
fn test_and_immediate() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    let mut cpu = cpu.borrow_mut();
    cpu.bus.borrow_mut().write_byte(0x1u16, 0xC0u8).unwrap();
    cpu.a = 0x81u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x29u8).unwrap();

    assert_eq!(cpu.a, 0x80u8);
    assert_eq!(cpu.p.n(), true);
    assert_eq!(cpu.p.z(), false);
}

#[test]
fn test_and_zero_flag() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    let mut cpu = cpu.borrow_mut();
    cpu.pc = 0u16;
    cpu.run_alu_op(0x29u8).unwrap();

    assert_eq!(cpu.a, 0x0u8);
    assert_eq!(cpu.p.z(), true);
    assert_eq!(cpu.p.n(), false);
}

#[test]
fn test_and_zero_page() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    let mut cpu = cpu.borrow_mut();
    cpu.bus.borrow_mut().write_byte(0x1u16, 0x80u8).unwrap();
    cpu.bus.borrow_mut().write_byte(0x80u16, 0xC0u8).unwrap();
    cpu.a = 0x81u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x25u8).unwrap();

    assert_eq!(cpu.a, 0x80u8);
    assert_eq!(cpu.p.n(), true);
    assert_eq!(cpu.p.z(), false);
}

#[test]
fn test_and_zero_page_x() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    let mut cpu = cpu.borrow_mut();
    cpu.bus.borrow_mut().write_byte(0x1u16, 0x80u8).unwrap();
    cpu.bus.borrow_mut().write_byte(0x85u16, 0xC0u8).unwrap();
    cpu.a = 0x81u8;
    cpu.x = 5u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x35u8).unwrap();

    assert_eq!(cpu.a, 0x80u8);
    assert_eq!(cpu.p.n(), true);
    assert_eq!(cpu.p.z(), false);
}

#[test]
fn test_and_absolute() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    let mut cpu = cpu.borrow_mut();
    cpu.bus.borrow_mut().write_word(0x1u16, 0x100u16).unwrap();
    cpu.bus.borrow_mut().write_byte(0x100u16, 0xC0u8).unwrap();
    cpu.a = 0x83u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x2Du8).unwrap();

    assert_eq!(cpu.a, 0x80u8);
    assert_eq!(cpu.p.n(), true);
    assert_eq!(cpu.p.z(), false);
}

#[test]
fn test_and_absolute_x() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    let mut cpu = cpu.borrow_mut();
    cpu.bus.borrow_mut().write_word(0x1u16, 0x100u16).unwrap();
    cpu.bus.borrow_mut().write_byte(0x105u16, 0xC0u8).unwrap();
    cpu.a = 0x81u8;
    cpu.x = 5u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x3Du8).unwrap();

    assert_eq!(cpu.a, 0x80u8);
    assert_eq!(cpu.p.n(), true);
    assert_eq!(cpu.p.z(), false);
}

#[test]
fn test_and_absolute_y() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    let mut cpu = cpu.borrow_mut();
    cpu.bus.borrow_mut().write_word(0x1u16, 0x100u16).unwrap();
    cpu.bus.borrow_mut().write_byte(0x105u16, 0xC0u8).unwrap();
    cpu.a = 0x81u8;
    cpu.y = 5u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x39u8).unwrap();

    assert_eq!(cpu.a, 0x80u8);
    assert_eq!(cpu.p.n(), true);
    assert_eq!(cpu.p.z(), false);
}

#[test]
fn test_and_indirect_x() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    let mut cpu = cpu.borrow_mut();
    {
        let mut bus = cpu.bus.borrow_mut();
        bus.write_byte(1u16, 0x80u8).unwrap();
        bus.write_word(0x85u16, 0x100u16).unwrap();
        bus.write_byte(0x100u16, 0xC0u8).unwrap();
    }
    cpu.a = 0x81u8;
    cpu.x = 5u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x21u8).unwrap();

    assert_eq!(cpu.a, 0x80u8);
    assert_eq!(cpu.p.n(), true);
    assert_eq!(cpu.p.z(), false);
}

#[test]
fn test_and_indirect_y() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    let mut cpu = cpu.borrow_mut();
    {
        let mut bus = cpu.bus.borrow_mut();
        bus.write_byte(1u16, 0x80u8).unwrap();
        bus.write_word(0x80u16, 0x100u16).unwrap();
        bus.write_byte(0x105u16, 0xC0u8).unwrap();
    }
    cpu.a = 0x81u8;
    cpu.y = 5u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x31u8).unwrap();

    assert_eq!(cpu.a, 0x80u8);
    assert_eq!(cpu.p.n(), true);
    assert_eq!(cpu.p.z(), false);
}
