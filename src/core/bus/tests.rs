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

#[test]
fn test_register_region() {
    let mut bus = Bus::new();
    let mock = Rc::new(RefCell::new(AddressableMock { data: 0 }));

    bus.register_region(0..10, mock.clone());

    assert!(!bus.regions.is_empty());
}

#[test]
fn test_read_byte() {
    let mut bus = Bus::new();
    let mock = Rc::new(RefCell::new(AddressableMock { data: 0x88u8 }));

    bus.register_region(0..10, mock.clone());

    let result = bus.read_byte(0).unwrap();

    assert_eq!(result, 0x88u8);
}

#[test]
fn test_read_byte_multiple_regions() {
    let mut bus = Bus::new();
    let mock_a = Rc::new(RefCell::new(AddressableMock { data: 0x88u8 }));
    let mock_b = Rc::new(RefCell::new(AddressableMock { data: 0xEFu8 }));

    bus.register_region(0..10, mock_a.clone());
    bus.register_region(10..20, mock_b.clone());

    let result_a = bus.read_byte(0).unwrap();
    let result_b = bus.read_byte(10).unwrap();

    assert_eq!(result_a, 0x88u8);
    assert_eq!(result_b, 0xEFu8);
}

#[test]
fn test_write_byte() {
    let mut bus = Bus::new();
    let mock = Rc::new(RefCell::new(AddressableMock { data: 0 }));

    bus.register_region(0..10, mock.clone());
    bus.write_byte(0u16, 0xFEu8).unwrap();

    let result = mock.borrow().data;
    assert_eq!(result, 0xFEu8);
}

#[test]
fn test_write_byte_multiple_regions() {
    let mut bus = Bus::new();
    let mock_a = Rc::new(RefCell::new(AddressableMock { data: 0 }));
    let mock_b = Rc::new(RefCell::new(AddressableMock { data: 0 }));

    bus.register_region(0..10, mock_a.clone());
    bus.register_region(10..20, mock_b.clone());

    bus.write_byte(0u16, 0xFEu8).unwrap();
    bus.write_byte(15u16, 0xABu8).unwrap();

    let result_a = mock_a.borrow().data;
    let result_b = mock_b.borrow().data;
    assert_eq!(result_a, 0xFEu8);
    assert_eq!(result_b, 0xABu8);
}
