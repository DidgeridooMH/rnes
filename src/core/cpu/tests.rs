use super::*;

#[test]
fn test_read_byte_first() {
    let bus = Rc::new(RefCell::new(Bus::new()));
    let cpu = CPU::new(&bus);

    cpu.borrow_mut().internal_ram[0] = 0x88u8;
    let result = cpu.borrow().read_byte(0u16);

    assert_eq!(result, 0x88u8);
}

#[test]
fn test_read_byte_last() {
    let bus = Rc::new(RefCell::new(Bus::new()));
    let cpu = CPU::new(&bus);

    cpu.borrow_mut().internal_ram[CPU_INTERNAL_RAM_SIZE - 1] = 0x73u8;
    let result = cpu.borrow().read_byte((CPU_INTERNAL_RAM_SIZE - 1) as u16);

    assert_eq!(result, 0x73u8);
}

#[test]
fn test_read_byte_mirrored() {
    let bus = Rc::new(RefCell::new(Bus::new()));
    let cpu = CPU::new(&bus);

    cpu.borrow_mut().internal_ram[0] = 0x73u8;
    let result = cpu.borrow().read_byte(CPU_INTERNAL_RAM_SIZE as u16);

    assert_eq!(result, 0x73u8);
}

#[test]
fn test_write_byte_first() {
    let bus = Rc::new(RefCell::new(Bus::new()));
    let cpu = CPU::new(&bus);

    cpu.borrow_mut().write_byte(0u16, 0x88u8);
    let result = cpu.borrow().read_byte(0u16);

    assert_eq!(result, 0x88u8);
}

#[test]
fn test_write_byte_last() {
    let bus = Rc::new(RefCell::new(Bus::new()));
    let cpu = CPU::new(&bus);

    cpu.borrow_mut()
        .write_byte((CPU_INTERNAL_RAM_SIZE - 1) as u16, 0x88u8);
    let result = cpu.borrow().read_byte((CPU_INTERNAL_RAM_SIZE - 1) as u16);

    assert_eq!(result, 0x88u8);
}

#[test]
fn test_write_byte_mirrored() {
    let bus = Rc::new(RefCell::new(Bus::new()));
    let cpu = CPU::new(&bus);

    cpu.borrow_mut()
        .write_byte(CPU_INTERNAL_RAM_SIZE as u16, 0x88u8);
    let result = cpu.borrow().read_byte(0u16);

    assert_eq!(result, 0x88u8);
}
