use super::*;
use crate::core::Bus;

#[test]
fn test_ora_immediate() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    bus.borrow_mut().write_byte(0x1u16, 0x80u8).unwrap();
    cpu.a = 1u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x09u8).unwrap();

    assert_eq!(cpu.a, 0x81u8);
    assert!(cpu.p.n());
    assert!(!cpu.p.z());
}

#[test]
fn test_ora_zero_flag() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.pc = 0u16;
    cpu.run_alu_op(0x09u8).unwrap();

    assert_eq!(cpu.a, 0x0u8);
    assert!(cpu.p.z());
    assert!(!cpu.p.n());
}

#[test]
fn test_ora_zero_page() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    {
        let mut bus = bus.borrow_mut();
        bus.write_byte(0x1u16, 0x80u8).unwrap();
        bus.write_byte(0x80u16, 0xaau8).unwrap();
    }
    cpu.a = 1u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x05u8).unwrap();

    assert_eq!(cpu.a, 0xabu8);
    assert!(cpu.p.n());
    assert!(!cpu.p.z());
}

#[test]
fn test_ora_zero_page_x() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    {
        let mut bus = bus.borrow_mut();
        bus.write_byte(0x1u16, 0x80u8).unwrap();
        bus.write_byte(0x85u16, 0xaau8).unwrap();
    }
    cpu.a = 1u8;
    cpu.x = 5u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x15u8).unwrap();

    assert_eq!(cpu.a, 0xabu8);
    assert!(cpu.p.n());
    assert!(!cpu.p.z());
}

#[test]
fn test_ora_absolute() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    {
        let mut bus = bus.borrow_mut();
        bus.write_word(0x1u16, 0x100u16).unwrap();
        bus.write_byte(0x100u16, 0xc0u8).unwrap();
    }
    cpu.a = 3u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x0du8).unwrap();

    assert_eq!(cpu.a, 0xc3u8);
    assert!(cpu.p.n());
    assert!(!cpu.p.z());
}

#[test]
fn test_ora_absolute_x() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    {
        let mut bus = bus.borrow_mut();
        bus.write_word(0x1u16, 0x100u16).unwrap();
        bus.write_byte(0x105u16, 0xc0u8).unwrap();
    }
    cpu.a = 3u8;
    cpu.x = 5u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x1du8).unwrap();

    assert_eq!(cpu.a, 0xc3u8);
    assert!(cpu.p.n());
    assert!(!cpu.p.z());
}

#[test]
fn test_ora_absolute_y() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    {
        let mut bus = bus.borrow_mut();
        bus.write_word(0x1u16, 0x100u16).unwrap();
        bus.write_byte(0x105u16, 0xc0u8).unwrap();
    }
    cpu.a = 3u8;
    cpu.y = 5u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x19u8).unwrap();

    assert_eq!(cpu.a, 0xc3u8);
    assert!(cpu.p.n());
    assert!(!cpu.p.z());
}

#[test]
fn test_ora_indirect_x() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    {
        let mut bus = cpu.bus.borrow_mut();
        bus.write_byte(1u16, 0x80u8).unwrap();
        bus.write_word(0x85u16, 0x100u16).unwrap();
        bus.write_byte(0x100u16, 0xc0u8).unwrap();
    }
    cpu.a = 3u8;
    cpu.x = 5u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x01u8).unwrap();

    assert_eq!(cpu.a, 0xc3u8);
    assert!(cpu.p.n());
    assert!(!cpu.p.z());
}

#[test]
fn test_ora_indirect_y() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    {
        let mut bus = cpu.bus.borrow_mut();
        bus.write_byte(1u16, 0x80u8).unwrap();
        bus.write_word(0x80u16, 0x100u16).unwrap();
        bus.write_byte(0x105u16, 0xc0u8).unwrap();
    }
    cpu.a = 3u8;
    cpu.y = 5u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x11u8).unwrap();

    assert_eq!(cpu.a, 0xc3u8);
    assert!(cpu.p.n());
    assert!(!cpu.p.z());
}

#[test]
fn test_and_immediate() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    bus.borrow_mut().write_byte(0x1u16, 0xc0u8).unwrap();
    cpu.a = 0x81u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x29u8).unwrap();

    assert_eq!(cpu.a, 0x80u8);
    assert!(cpu.p.n());
    assert!(!cpu.p.z());
}

#[test]
fn test_and_zero_flag() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.pc = 0u16;
    cpu.run_alu_op(0x29u8).unwrap();

    assert_eq!(cpu.a, 0x0u8);
    assert!(cpu.p.z());
    assert!(!cpu.p.n());
}

#[test]
fn test_and_zero_page() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    {
        let mut bus = bus.borrow_mut();
        bus.write_byte(0x1u16, 0x80u8).unwrap();
        bus.write_byte(0x80u16, 0xc0u8).unwrap();
    }
    cpu.a = 0x81u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x25u8).unwrap();

    assert_eq!(cpu.a, 0x80u8);
    assert!(cpu.p.n());
    assert!(!cpu.p.z());
}

#[test]
fn test_and_zero_page_x() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    {
        let mut bus = bus.borrow_mut();
        bus.write_byte(0x1u16, 0x80u8).unwrap();
        bus.write_byte(0x85u16, 0xc0u8).unwrap();
    }
    cpu.a = 0x81u8;
    cpu.x = 5u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x35u8).unwrap();

    assert_eq!(cpu.a, 0x80u8);
    assert!(cpu.p.n());
    assert!(!cpu.p.z());
}

#[test]
fn test_and_absolute() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    {
        let mut bus = bus.borrow_mut();
        bus.write_word(0x1u16, 0x100u16).unwrap();
        bus.write_byte(0x100u16, 0xc0u8).unwrap();
    }
    cpu.a = 0x83u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x2du8).unwrap();

    assert_eq!(cpu.a, 0x80u8);
    assert!(cpu.p.n());
    assert!(!cpu.p.z());
}

#[test]
fn test_and_absolute_x() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    {
        let mut bus = bus.borrow_mut();
        bus.write_word(0x1u16, 0x100u16).unwrap();
        bus.write_byte(0x105u16, 0xc0u8).unwrap();
    }
    cpu.a = 0x81u8;
    cpu.x = 5u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x3du8).unwrap();

    assert_eq!(cpu.a, 0x80u8);
    assert!(cpu.p.n());
    assert!(!cpu.p.z());
}

#[test]
fn test_and_absolute_y() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    {
        let mut bus = bus.borrow_mut();
        bus.write_word(0x1u16, 0x100u16).unwrap();
        bus.write_byte(0x105u16, 0xc0u8).unwrap();
    }
    cpu.a = 0x81u8;
    cpu.y = 5u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x39u8).unwrap();

    assert_eq!(cpu.a, 0x80u8);
    assert!(cpu.p.n());
    assert!(!cpu.p.z());
}

#[test]
fn test_and_indirect_x() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    {
        let mut bus = cpu.bus.borrow_mut();
        bus.write_byte(1u16, 0x80u8).unwrap();
        bus.write_word(0x85u16, 0x100u16).unwrap();
        bus.write_byte(0x100u16, 0xc0u8).unwrap();
    }
    cpu.a = 0x81u8;
    cpu.x = 5u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x21u8).unwrap();

    assert_eq!(cpu.a, 0x80u8);
    assert!(cpu.p.n());
    assert!(!cpu.p.z());
}

#[test]
fn test_and_indirect_y() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    {
        let mut bus = bus.borrow_mut();
        bus.write_byte(1u16, 0x80u8).unwrap();
        bus.write_word(0x80u16, 0x100u16).unwrap();
        bus.write_byte(0x105u16, 0xc0u8).unwrap();
    }
    cpu.a = 0x81u8;
    cpu.y = 5u8;
    cpu.pc = 0u16;
    cpu.run_alu_op(0x31u8).unwrap();

    assert_eq!(cpu.a, 0x80u8);
    assert!(cpu.p.n());
    assert!(!cpu.p.z());
}

#[test]
fn test_eor_zero() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.a = 0x80u8;
    cpu.eor(0x80u8);

    assert_eq!(cpu.a, 0x0u8);
    assert!(!cpu.p.n());
    assert!(cpu.p.z());
}

#[test]
fn test_eor_positive() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.a = 0x1u8;
    cpu.eor(0x2u8);

    assert_eq!(cpu.a, 0x3u8);
    assert!(!cpu.p.n());
    assert!(!cpu.p.z());
}

#[test]
fn test_eor_negative() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.a = 0x80u8;
    cpu.eor(0u8);

    assert_eq!(cpu.a, 0x80u8);
    assert!(cpu.p.n());
    assert!(!cpu.p.z());
}

#[test]
fn test_adc_zero() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.a = 0u8;
    cpu.adc(0u8);

    assert_eq!(cpu.a, 0u8);
    assert!(!cpu.p.c());
    assert!(!cpu.p.n());
    assert!(cpu.p.z());
    assert!(!cpu.p.v());
}

#[test]
fn test_adc_positive() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.a = 3u8;
    cpu.adc(4u8);

    assert_eq!(cpu.a, 7u8);
    assert!(!cpu.p.c());
    assert!(!cpu.p.n());
    assert!(!cpu.p.z());
    assert!(!cpu.p.v());
}

#[test]
fn test_adc_negative() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.a = 0x80u8;
    cpu.adc(0x1u8);

    assert_eq!(cpu.a, 0x81u8);
    assert!(!cpu.p.c());
    assert!(cpu.p.n());
    assert!(!cpu.p.z());
    assert!(!cpu.p.v());
}

#[test]
fn test_adc_carry() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.a = 0xffu8;
    cpu.adc(0x2u8);

    assert_eq!(cpu.a, 0x1u8);
    assert!(cpu.p.c());
    assert!(!cpu.p.n());
    assert!(!cpu.p.z());
    assert!(!cpu.p.v());
}

#[test]
fn test_adc_overflow_pton() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.a = 0x7fu8;
    cpu.adc(0x01u8);

    assert_eq!(cpu.a, 0x80u8);
    assert!(!cpu.p.c());
    assert!(cpu.p.n());
    assert!(!cpu.p.z());
    assert!(cpu.p.v());
}

#[test]
fn test_adc_overflow_ntop() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.a = 0x81u8;
    cpu.adc(0x80u8);

    assert_eq!(cpu.a, 0x1u8);
    assert!(cpu.p.c());
    assert!(!cpu.p.n());
    assert!(!cpu.p.z());
    assert!(cpu.p.v());
}

#[test]
fn test_sta() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    bus.borrow_mut().write_byte(0x1u16, 0x80u8).unwrap();

    cpu.a = 0x81u8;
    cpu.pc = 0x0u16;
    cpu.run_alu_op(0x85u8).unwrap();

    let result = bus.borrow_mut().read_byte(0x80u16).unwrap();
    assert_eq!(result, 0x81u8);
}

#[test]
fn test_lda_zero() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.a = 0x80u8;
    cpu.lda(0x00u8);

    assert_eq!(cpu.a, 0x00u8);
    assert!(cpu.p.z());
    assert!(!cpu.p.n());
}

#[test]
fn test_lda_positive() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.a = 0x80u8;
    cpu.lda(0x40u8);

    assert_eq!(cpu.a, 0x40u8);
    assert!(!cpu.p.z());
    assert!(!cpu.p.n());
}

#[test]
fn test_lda_negative() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.a = 0x80u8;
    cpu.lda(0xefu8);

    assert_eq!(cpu.a, 0xefu8);
    assert!(!cpu.p.z());
    assert!(cpu.p.n());
}

#[test]
fn test_cmp_zero() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.a = 0x20u8;
    cpu.cmp(0x20u8);

    assert_eq!(cpu.a, 0x20u8);
    assert!(cpu.p.z());
    assert!(!cpu.p.n());
    assert!(cpu.p.c());
}

#[test]
fn test_cmp_negative() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.a = 0xe0u8;
    cpu.cmp(0x10u8);

    assert_eq!(cpu.a, 0xe0u8);
    assert!(!cpu.p.z());
    assert!(cpu.p.n());
    assert!(cpu.p.c());
}

#[test]
fn test_cmp_carry() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.a = 0x30u8;
    cpu.cmp(0x20u8);

    assert_eq!(cpu.a, 0x30u8);
    assert!(!cpu.p.z());
    assert!(!cpu.p.n());
    assert!(cpu.p.c());
}

#[test]
fn test_cmp_negative_no_carry() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.a = 0x20u8;
    cpu.cmp(0x30u8);

    assert_eq!(cpu.a, 0x20u8);
    assert!(!cpu.p.z());
    assert!(cpu.p.n());
    assert!(!cpu.p.c());
}

#[test]
fn test_sbc_no_flags() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.a = 0x30u8;
    cpu.sbc(0x10u8);

    assert_eq!(cpu.a, 0x20u8);
    assert!(!cpu.p.z());
    assert!(!cpu.p.n());
    assert!(cpu.p.c());
    assert!(!cpu.p.v());
}

#[test]
fn test_sbc_zero() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.a = 0x10u8;
    cpu.p.set_c(true);
    cpu.sbc(0x10u8);

    assert_eq!(cpu.a, 0x00u8);
    assert!(cpu.p.z());
    assert!(!cpu.p.n());
    assert!(!cpu.p.v());
    assert!(cpu.p.c());
}

#[test]
fn test_sbc_overflow_positive() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.a = 0x7fu8;
    cpu.p.set_c(true);
    cpu.sbc(0xffu8);

    assert_eq!(cpu.a, 0x80u8);
    assert!(!cpu.p.z());
    assert!(cpu.p.n());
    assert!(cpu.p.v());
    assert!(!cpu.p.c());
}

#[test]
fn test_sbc_overflow_negative() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    cpu.a = 0xfeu8;
    cpu.p.set_c(true);
    cpu.sbc(0x7fu8);

    assert_eq!(cpu.a, 0x7fu8);
    assert!(!cpu.p.z());
    assert!(!cpu.p.n());
    assert!(cpu.p.v());
    assert!(!cpu.p.c());
}
