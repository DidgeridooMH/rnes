use super::*;

#[test]
fn test_read_byte_first() {
    let mem = InternalRam::new();

    mem.borrow_mut().data[0] = 0x88u8;
    let result = mem.borrow_mut().read_byte(0u16);

    assert_eq!(result, Some(0x88u8));
}

#[test]
fn test_read_byte_last() {
    let mem = InternalRam::new();

    mem.borrow_mut().data[CPU_INTERNAL_RAM_SIZE - 1] = 0x73u8;
    let result = mem
        .borrow_mut()
        .read_byte((CPU_INTERNAL_RAM_SIZE - 1) as u16);

    assert_eq!(result, Some(0x73u8));
}

#[test]
fn test_read_byte_mirrored() {
    let mem = InternalRam::new();

    mem.borrow_mut().data[0] = 0x73u8;
    let result = mem.borrow_mut().read_byte(CPU_INTERNAL_RAM_SIZE as u16);

    assert_eq!(result, Some(0x73u8));
}

#[test]
fn test_write_byte_first() {
    let mem = InternalRam::new();

    mem.borrow_mut().write_byte(0u16, 0x88u8);
    let result = mem.borrow_mut().read_byte(0u16);

    assert_eq!(result, Some(0x88u8));
}

#[test]
fn test_write_byte_last() {
    let mem = InternalRam::new();

    mem.borrow_mut()
        .write_byte((CPU_INTERNAL_RAM_SIZE - 1) as u16, 0x88u8);
    let result = mem
        .borrow_mut()
        .read_byte((CPU_INTERNAL_RAM_SIZE - 1) as u16);

    assert_eq!(result, Some(0x88u8));
}

#[test]
fn test_write_byte_mirrored() {
    let mem = InternalRam::new();

    mem.borrow_mut()
        .write_byte(CPU_INTERNAL_RAM_SIZE as u16, 0x88u8);
    let result = mem.borrow_mut().read_byte(0u16);

    assert_eq!(result, Some(0x88u8));
}
