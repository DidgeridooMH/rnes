use super::*;

#[test]
fn test_nz_flag_zero() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    cpu.borrow_mut().set_nz_flags(0u8);

    assert_eq!(cpu.borrow().p.z(), true);
    assert_eq!(cpu.borrow().p.n(), false);
}

#[test]
fn test_nz_flag_positive() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    cpu.borrow_mut().set_nz_flags(12u8);

    assert_eq!(cpu.borrow().p.z(), false);
    assert_eq!(cpu.borrow().p.n(), false);
}

#[test]
fn test_nz_flag_negative() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    cpu.borrow_mut().set_nz_flags(0x81u8);

    assert_eq!(cpu.borrow().p.z(), false);
    assert_eq!(cpu.borrow().p.n(), true);
}

#[test]
fn test_get_address_immediate() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    cpu.borrow_mut().pc = 0u16;
    let address = cpu.borrow_mut().get_address(AddressMode::Immediate);
    assert_eq!(address, 1u16);
}

#[test]
fn test_get_address_zero_page() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    cpu.borrow_mut().pc = 0u16;
    cpu.borrow_mut()
        .bus
        .borrow_mut()
        .write_byte(1u16, 0x88u8)
        .unwrap();

    let address = cpu.borrow_mut().get_address(AddressMode::ZeroPage);
    assert_eq!(address, 0x88u16);
}

#[test]
fn test_get_address_zero_page_x() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    cpu.borrow_mut().pc = 0u16;
    cpu.borrow_mut().x = 5;
    cpu.borrow_mut()
        .bus
        .borrow_mut()
        .write_byte(1u16, 0x88u8)
        .unwrap();

    let address = cpu.borrow_mut().get_address(AddressMode::ZeroPageX);
    assert_eq!(address, 0x88u16 + 5u16);
}

#[test]
fn test_get_address_zero_page_x_page_cross() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    cpu.borrow_mut().pc = 0u16;
    cpu.borrow_mut().x = 2;
    cpu.borrow_mut()
        .bus
        .borrow_mut()
        .write_byte(1u16, 0xFEu8)
        .unwrap();

    let address = cpu.borrow_mut().get_address(AddressMode::ZeroPageX);
    assert_eq!(address, 0u16);
}

#[test]
fn test_get_address_zero_page_y() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    cpu.borrow_mut().pc = 0u16;
    cpu.borrow_mut().y = 5;
    cpu.borrow_mut()
        .bus
        .borrow_mut()
        .write_byte(1u16, 0x88u8)
        .unwrap();

    let address = cpu.borrow_mut().get_address(AddressMode::ZeroPageY);
    assert_eq!(address, 0x88u16 + 5u16);
}

#[test]
fn test_get_address_zero_page_y_page_cross() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    cpu.borrow_mut().pc = 0u16;
    cpu.borrow_mut().y = 2;
    cpu.borrow_mut()
        .bus
        .borrow_mut()
        .write_byte(1u16, 0xFEu8)
        .unwrap();

    let address = cpu.borrow_mut().get_address(AddressMode::ZeroPageY);
    assert_eq!(address, 0u16);
}

#[test]
fn test_get_address_absolute() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    cpu.borrow_mut().pc = 0u16;
    cpu.borrow_mut()
        .bus
        .borrow_mut()
        .write_word(1u16, 0xBBAAu16)
        .unwrap();

    let address = cpu.borrow_mut().get_address(AddressMode::Absolute);
    assert_eq!(address, 0xBBAAu16);
}

#[test]
fn test_get_address_absolute_x() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    cpu.borrow_mut().pc = 0u16;
    cpu.borrow_mut().x = 5u8;
    cpu.borrow_mut()
        .bus
        .borrow_mut()
        .write_word(1u16, 0xBBAAu16)
        .unwrap();

    let address = cpu.borrow_mut().get_address(AddressMode::AbsoluteX);
    assert_eq!(address, 0xBBAAu16 + 5u16);
}

#[test]
fn test_get_address_absolute_y() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    cpu.borrow_mut().pc = 0u16;
    cpu.borrow_mut().y = 5u8;
    cpu.borrow_mut()
        .bus
        .borrow_mut()
        .write_word(1u16, 0xBBAAu16)
        .unwrap();

    let address = cpu.borrow_mut().get_address(AddressMode::AbsoluteY);
    assert_eq!(address, 0xBBAAu16 + 5u16);
}

#[test]
fn test_get_address_indirect_x() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    cpu.borrow_mut().pc = 0u16;
    cpu.borrow_mut().x = 5u8;
    cpu.borrow_mut()
        .bus
        .borrow_mut()
        .write_word(0x10u16, 0x1234u16)
        .unwrap();
    cpu.borrow_mut()
        .bus
        .borrow_mut()
        .write_byte(1u16, 0x10u8 - 0x5u8)
        .unwrap();

    let address = cpu.borrow_mut().get_address(AddressMode::IndirectX);
    assert_eq!(address, 0x1234u16);
}

#[test]
fn test_get_address_indirect_x_page_cross() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    cpu.borrow_mut().pc = 0u16;
    cpu.borrow_mut().x = 1u8;
    cpu.borrow_mut()
        .bus
        .borrow_mut()
        .write_byte(0x0u16, 0x12u8)
        .unwrap();
    cpu.borrow_mut()
        .bus
        .borrow_mut()
        .write_byte(0xFFu16, 0x34u8)
        .unwrap();
    cpu.borrow_mut()
        .bus
        .borrow_mut()
        .write_byte(1u16, 0xFEu8)
        .unwrap();

    let address = cpu.borrow_mut().get_address(AddressMode::IndirectX);
    assert_eq!(address, 0x1234u16);
}

#[test]
fn test_get_address_indirect_y() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    cpu.borrow_mut().pc = 0u16;
    cpu.borrow_mut().y = 5u8;
    cpu.borrow_mut()
        .bus
        .borrow_mut()
        .write_word(0x10u16, 0x1234u16)
        .unwrap();
    cpu.borrow_mut()
        .bus
        .borrow_mut()
        .write_byte(1u16, 0x10u8)
        .unwrap();

    let address = cpu.borrow_mut().get_address(AddressMode::IndirectY);
    assert_eq!(address, 0x1234u16 + 5u16);
}
