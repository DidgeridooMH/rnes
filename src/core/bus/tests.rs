use super::*;

struct AddressableMock {
    data: u8,
}

impl Addressable for AddressableMock {
    fn read_byte(&self, _address: u16) -> u8 {
        return self.data;
    }

    fn write_byte(&mut self, _address: u16, data: u8) {
        self.data = data;
    }
}

struct MultiAddressableMock {
    data: [u8; 2],
}

impl Addressable for MultiAddressableMock {
    fn read_byte(&self, address: u16) -> u8 {
        return self.data[(address % 2) as usize];
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        self.data[(address % 2) as usize] = data;
    }
}

#[test]
fn test_register_region() {
    let bus = Bus::new();
    let mock = Rc::new(RefCell::new(AddressableMock { data: 0 }));

    bus.borrow_mut().register_region(0..10, mock.clone());

    assert!(!bus.borrow_mut().regions.is_empty());
}

#[test]
fn test_read_byte() {
    let bus = Bus::new();
    let mock = Rc::new(RefCell::new(AddressableMock { data: 0x88u8 }));

    bus.borrow_mut().register_region(0..10, mock.clone());

    let result = bus.borrow_mut().read_byte(0).unwrap();

    assert_eq!(result, 0x88u8);
}

#[test]
fn test_read_byte_multiple_regions() {
    let bus = Bus::new();
    let mock_a = Rc::new(RefCell::new(AddressableMock { data: 0x88u8 }));
    let mock_b = Rc::new(RefCell::new(AddressableMock { data: 0xEFu8 }));

    bus.borrow_mut().register_region(0..10, mock_a.clone());
    bus.borrow_mut().register_region(10..20, mock_b.clone());

    let result_a = bus.borrow_mut().read_byte(0).unwrap();
    let result_b = bus.borrow_mut().read_byte(10).unwrap();

    assert_eq!(result_a, 0x88u8);
    assert_eq!(result_b, 0xEFu8);
}

#[test]
fn test_write_byte() {
    let bus = Bus::new();
    let mock = Rc::new(RefCell::new(AddressableMock { data: 0 }));

    bus.borrow_mut().register_region(0..10, mock.clone());
    bus.borrow_mut().write_byte(0u16, 0xFEu8).unwrap();

    let result = mock.borrow().data;
    assert_eq!(result, 0xFEu8);
}

#[test]
fn test_write_byte_multiple_regions() {
    let bus = Bus::new();
    let mock_a = Rc::new(RefCell::new(AddressableMock { data: 0 }));
    let mock_b = Rc::new(RefCell::new(AddressableMock { data: 0 }));

    bus.borrow_mut().register_region(0..10, mock_a.clone());
    bus.borrow_mut().register_region(10..20, mock_b.clone());

    bus.borrow_mut().write_byte(0u16, 0xFEu8).unwrap();
    bus.borrow_mut().write_byte(15u16, 0xABu8).unwrap();

    let result_a = mock_a.borrow().data;
    let result_b = mock_b.borrow().data;
    assert_eq!(result_a, 0xFEu8);
    assert_eq!(result_b, 0xABu8);
}

#[test]
fn test_read_word() {
    let bus = Bus::new();
    let mock = Rc::new(RefCell::new(AddressableMock { data: 0x88u8 }));

    bus.borrow_mut().register_region(0..0x10, mock.clone());

    let result = bus.borrow_mut().read_word(0).unwrap();

    assert_eq!(result, 0x8888u16);
}

#[test]
fn test_read_word_cross_boundary() {
    let bus = Bus::new();
    let mock_a = Rc::new(RefCell::new(AddressableMock { data: 0x88u8 }));
    let mock_b = Rc::new(RefCell::new(AddressableMock { data: 0x77u8 }));

    bus.borrow_mut().register_region(0..0x10, mock_a.clone());
    bus.borrow_mut().register_region(0x10..0x20, mock_b.clone());

    let result = bus.borrow_mut().read_word(0xF).unwrap();

    assert_eq!(result, 0x7788u16);
}

#[test]
fn test_read_word_bug() {
    let bus = Bus::new();
    let mock_a = Rc::new(RefCell::new(AddressableMock { data: 0x88u8 }));
    let mock_b = Rc::new(RefCell::new(AddressableMock { data: 0x77u8 }));

    bus.borrow_mut().register_region(0..0x100, mock_a.clone());
    bus.borrow_mut()
        .register_region(0x100..0x200, mock_b.clone());

    let result = bus.borrow_mut().read_word_bug(0xF).unwrap();

    assert_eq!(result, 0x8888u16);
}

#[test]
fn test_write_word() {
    let bus = Bus::new();
    let mock = Rc::new(RefCell::new(MultiAddressableMock { data: [0, 0] }));

    bus.borrow_mut().register_region(0..10, mock.clone());
    bus.borrow_mut().write_word(0u16, 0xDEADu16).unwrap();

    let res_low = mock.borrow().data[0];
    let res_high = mock.borrow().data[1];
    assert_eq!(res_low, 0xADu8);
    assert_eq!(res_high, 0xDEu8);
}

#[test]
fn test_write_word_cross_boundary() {
    let bus = Bus::new();
    let mock_a = Rc::new(RefCell::new(AddressableMock { data: 0x88u8 }));
    let mock_b = Rc::new(RefCell::new(AddressableMock { data: 0x77u8 }));

    bus.borrow_mut().register_region(0..0x100, mock_a.clone());
    bus.borrow_mut()
        .register_region(0x100..0x200, mock_b.clone());

    bus.borrow_mut().write_word(0xFFu16, 0x1122u16).unwrap();

    assert_eq!(mock_a.borrow().data, 0x22u8);
    assert_eq!(mock_b.borrow().data, 0x11u8);
}
