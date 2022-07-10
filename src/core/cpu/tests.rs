use super::*;

#[test]
fn test_nz_flag_zero() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.set_nz_flags(0u8);

    assert_eq!(cpu.p.z(), true);
    assert_eq!(cpu.p.n(), false);
}

#[test]
fn test_nz_flag_positive() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.set_nz_flags(12u8);

    assert_eq!(cpu.p.z(), false);
    assert_eq!(cpu.p.n(), false);
}

#[test]
fn test_nz_flag_negative() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.set_nz_flags(0x81u8);

    assert_eq!(cpu.p.z(), false);
    assert_eq!(cpu.p.n(), true);
}

#[test]
fn test_get_address_immediate() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.pc = 0u16;
    let address = cpu.get_address(AddressMode::Immediate).unwrap();
    assert_eq!(address, (1u16, false));
}

#[test]
fn test_get_address_zero_page() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.pc = 0u16;
    bus.borrow_mut().write_byte(1u16, 0x88u8).unwrap();

    let address = cpu.get_address(AddressMode::ZeroPage).unwrap();
    assert_eq!(address, (0x88u16, false));
}

#[test]
fn test_get_address_zero_page_x() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.pc = 0u16;
    cpu.x = 5;
    bus.borrow_mut().write_byte(1u16, 0x88u8).unwrap();

    let address = cpu.get_address(AddressMode::ZeroPageX).unwrap();
    assert_eq!(address, (0x88u16 + 5u16, false));
}

#[test]
fn test_get_address_zero_page_x_page_cross() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.pc = 0u16;
    cpu.x = 2;
    bus.borrow_mut().write_byte(1u16, 0xFEu8).unwrap();

    let address = cpu.get_address(AddressMode::ZeroPageX).unwrap();
    assert_eq!(address, (0u16, false));
}

#[test]
fn test_get_address_zero_page_y() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.pc = 0u16;
    cpu.y = 5;
    bus.borrow_mut().write_byte(1u16, 0x88u8).unwrap();

    let address = cpu.get_address(AddressMode::ZeroPageY).unwrap();
    assert_eq!(address, (0x88u16 + 5u16, false));
}

#[test]
fn test_get_address_zero_page_y_page_cross() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.pc = 0u16;
    cpu.y = 2;
    bus.borrow_mut().write_byte(1u16, 0xFEu8).unwrap();

    let address = cpu.get_address(AddressMode::ZeroPageY).unwrap();
    assert_eq!(address, (0u16, false));
}

#[test]
fn test_get_address_absolute() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.pc = 0u16;
    bus.borrow_mut().write_word(1u16, 0xBBAAu16).unwrap();

    let address = cpu.get_address(AddressMode::Absolute).unwrap();
    assert_eq!(address, (0xBBAAu16, false));
}

#[test]
fn test_get_address_absolute_x() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.pc = 0u16;
    cpu.x = 5u8;
    bus.borrow_mut().write_word(1u16, 0xBBAAu16).unwrap();

    let address = cpu.get_address(AddressMode::AbsoluteX).unwrap();
    assert_eq!(address, (0xBBAAu16 + 5u16, false));
}

#[test]
fn test_get_address_absolute_x_page_cross() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.pc = 0u16;
    cpu.x = 0x2u8;
    bus.borrow_mut().write_word(1u16, 0xBBFFu16).unwrap();

    let address = cpu.get_address(AddressMode::AbsoluteX).unwrap();
    assert_eq!(address, (0xBBFFu16 + 0x2u16, true));
}

#[test]
fn test_get_address_absolute_y() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.pc = 0u16;
    cpu.y = 5u8;
    bus.borrow_mut().write_word(1u16, 0xBBAAu16).unwrap();

    let address = cpu.get_address(AddressMode::AbsoluteY).unwrap();
    assert_eq!(address, (0xBBAAu16 + 5u16, false));
}

#[test]
fn test_get_address_absolute_y_page_cross() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.pc = 0u16;
    cpu.y = 0x2u8;
    bus.borrow_mut().write_word(1u16, 0xBBFFu16).unwrap();

    let address = cpu.get_address(AddressMode::AbsoluteY).unwrap();
    assert_eq!(address, (0xBBFFu16 + 0x2u16, true));
}

#[test]
fn test_get_address_indirect_x() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.pc = 0u16;
    cpu.x = 5u8;
    bus.borrow_mut().write_word(0x10u16, 0x1234u16).unwrap();
    bus.borrow_mut().write_byte(1u16, 0x10u8 - 0x5u8).unwrap();

    let address = cpu.get_address(AddressMode::IndirectX).unwrap();
    assert_eq!(address, (0x1234u16, false));
}

#[test]
fn test_get_address_indirect_x_page_cross() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.pc = 0u16;
    cpu.x = 1u8;
    bus.borrow_mut().write_byte(0x0u16, 0x12u8).unwrap();
    bus.borrow_mut().write_byte(0xFFu16, 0x34u8).unwrap();
    bus.borrow_mut().write_byte(1u16, 0xFEu8).unwrap();

    let address = cpu.get_address(AddressMode::IndirectX).unwrap();
    assert_eq!(address, (0x1234u16, false));
}

#[test]
fn test_get_address_indirect_y() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.pc = 0u16;
    cpu.y = 5u8;
    bus.borrow_mut().write_word(0x10u16, 0x1234u16).unwrap();
    bus.borrow_mut().write_byte(1u16, 0x10u8).unwrap();

    let address = cpu.get_address(AddressMode::IndirectY).unwrap();
    assert_eq!(address, (0x1234u16 + 5u16, false));
}

#[test]
fn test_get_address_indirect_y_page_cross() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.pc = 0u16;
    cpu.y = 0x2u8;
    bus.borrow_mut().write_word(0x10u16, 0x12FFu16).unwrap();
    bus.borrow_mut().write_byte(1u16, 0x10u8).unwrap();

    let address = cpu.get_address(AddressMode::IndirectY).unwrap();
    assert_eq!(address, (0x12FFu16 + 0x2u16, true));
}

#[test]
fn test_push_byte() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.sp = 0xFFu8;
    cpu.push_byte(0x80u8).unwrap();

    let result = bus.borrow_mut().read_byte(0xFF).unwrap();
    assert_eq!(cpu.sp, 0xFEu8);
    assert_eq!(result, 0x80u8);
}

#[test]
fn test_push_word() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.sp = 0xFFu8;
    cpu.push_word(0xBEEFu16).unwrap();

    let result = bus.borrow().read_word(0xFEu16).unwrap();
    assert_eq!(cpu.sp, 0xFDu8);
    assert_eq!(result, 0xBEEFu16);
}

#[test]
fn test_pop_byte() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.sp = 0xFEu8;
    bus.borrow_mut().write_byte(0xFFu16, 0x8Eu8).unwrap();

    let result = cpu.pop_byte().unwrap();
    assert_eq!(result, 0x8Eu8);
    assert_eq!(cpu.sp, 0xFFu8);
}

#[test]
fn test_pop_word() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.sp = 0xFFu8;
    cpu.push_word(0xBEEFu16).unwrap();

    let result = cpu.pop_word().unwrap();
    assert_eq!(result, 0xBEEFu16);
    assert_eq!(cpu.sp, 0xFFu8);
}
