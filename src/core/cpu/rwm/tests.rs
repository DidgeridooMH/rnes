use super::*;
use crate::core::Bus;
use std::{cell::RefCell, rc::Rc};

fn setup() -> (Rc<RefCell<Bus>>, CPU) {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);
    (bus, cpu)
}

#[test]
fn test_asl_zero() {
    let mut cpu = setup().1;

    cpu.a = 0;
    cpu.asl(0x0A).unwrap();

    assert_eq!(cpu.a, 0);
    assert!(!cpu.p.c());
    assert!(cpu.p.z());
    assert!(!cpu.p.n());
}

#[test]
fn test_asl_positive() {
    let mut cpu = setup().1;

    cpu.a = 0x2;
    cpu.asl(0x0A).unwrap();

    assert_eq!(cpu.a, 0x4);
    assert!(!cpu.p.c());
    assert!(!cpu.p.z());
    assert!(!cpu.p.n());
}

#[test]
fn test_asl_negative() {
    let mut cpu = setup().1;

    cpu.a = 0x82;
    cpu.asl(0x0A).unwrap();

    assert_eq!(cpu.a, 0x4);
    assert!(cpu.p.c());
    assert!(!cpu.p.z());
    assert!(!cpu.p.n());
}

#[test]
fn test_asl_to_negative() {
    let mut cpu = setup().1;

    cpu.a = 0x40;
    cpu.asl(0x0A).unwrap();

    assert_eq!(cpu.a, 0x80);
    assert!(!cpu.p.c());
    assert!(!cpu.p.z());
    assert!(cpu.p.n());
}

#[test]
fn test_rol_zero() {
    let mut cpu = setup().1;

    cpu.a = 0;
    cpu.rol(0x2A).unwrap();

    assert_eq!(cpu.a, 0);
    assert!(!cpu.p.c());
    assert!(cpu.p.z());
    assert!(!cpu.p.n());
}

#[test]
fn test_rol_positive() {
    let mut cpu = setup().1;

    cpu.a = 0x2;
    cpu.rol(0x2A).unwrap();

    assert_eq!(cpu.a, 0x4);
    assert!(!cpu.p.c());
    assert!(!cpu.p.z());
    assert!(!cpu.p.n());
}

#[test]
fn test_rol_negative() {
    let mut cpu = setup().1;

    cpu.a = 0x82;
    cpu.rol(0x2A).unwrap();

    assert_eq!(cpu.a, 0x4);
    assert!(cpu.p.c());
    assert!(!cpu.p.z());
    assert!(!cpu.p.n());
}

#[test]
fn test_rol_to_negative() {
    let mut cpu = setup().1;

    cpu.a = 0x40;
    cpu.rol(0x2A).unwrap();

    assert_eq!(cpu.a, 0x80);
    assert!(!cpu.p.c());
    assert!(!cpu.p.z());
    assert!(cpu.p.n());
}

#[test]
fn test_rol_from_carry() {
    let mut cpu = setup().1;

    cpu.a = 0x10;
    cpu.p.set_c(true);
    cpu.rol(0x2A).unwrap();

    assert_eq!(cpu.a, 0x21);
    assert!(!cpu.p.c());
    assert!(!cpu.p.z());
    assert!(!cpu.p.n());
}

#[test]
fn test_lsr_zero() {
    let mut cpu = setup().1;

    cpu.a = 0;
    cpu.lsr(0x4A).unwrap();

    assert_eq!(cpu.a, 0);
    assert!(cpu.p.z());
    assert!(!cpu.p.c());
    assert!(!cpu.p.n());
}

#[test]
fn test_lsr_positive() {
    let mut cpu = setup().1;

    cpu.a = 4;
    cpu.lsr(0x4A).unwrap();

    assert_eq!(cpu.a, 2);
    assert!(!cpu.p.z());
    assert!(!cpu.p.c());
    assert!(!cpu.p.n());
}

#[test]
fn test_lsr_into_carry() {
    let mut cpu = setup().1;

    cpu.a = 1;
    cpu.p.set_c(false);
    cpu.lsr(0x4A).unwrap();

    assert_eq!(cpu.a, 0);
    assert!(cpu.p.z());
    assert!(cpu.p.c());
    assert!(!cpu.p.n());
}

#[test]
fn test_ror_zero() {
    let mut cpu = setup().1;

    cpu.a = 0;
    cpu.ror(0x6A).unwrap();

    assert_eq!(cpu.a, 0);
    assert!(!cpu.p.c());
    assert!(cpu.p.z());
    assert!(!cpu.p.n());
}

#[test]
fn test_ror_positive() {
    let mut cpu = setup().1;

    cpu.a = 0x2;
    cpu.ror(0x6A).unwrap();

    assert_eq!(cpu.a, 0x1);
    assert!(!cpu.p.c());
    assert!(!cpu.p.z());
    assert!(!cpu.p.n());
}

#[test]
fn test_ror_negative() {
    let mut cpu = setup().1;

    cpu.a = 0x2;
    cpu.p.set_c(true);
    cpu.ror(0x6A).unwrap();

    assert_eq!(cpu.a, 0x81);
    assert!(!cpu.p.c());
    assert!(!cpu.p.z());
    assert!(cpu.p.n());
}

#[test]
fn test_rol_into_carry() {
    let mut cpu = setup().1;

    cpu.a = 1;
    cpu.ror(0x6A).unwrap();

    assert_eq!(cpu.a, 0);
    assert!(cpu.p.c());
    assert!(cpu.p.z());
    assert!(!cpu.p.n());
}

#[test]
fn test_stx_zero() {
    let (bus, mut cpu) = setup();

    {
        let mut bus = bus.borrow_mut();
        bus.write_word(1, 0x500).unwrap();
    }

    cpu.pc = 0;
    cpu.x = 0;
    let status = cpu.p;
    cpu.stx(0x8E).unwrap();

    let res = bus.borrow().read_byte(0x500).unwrap();
    assert_eq!(cpu.x, 0);
    assert_eq!(res, 0);
    assert_eq!(status, cpu.p);
}

#[test]
fn test_stx_generic() {
    let (bus, mut cpu) = setup();

    {
        let mut bus = bus.borrow_mut();
        bus.write_word(1, 0x500).unwrap();
    }

    cpu.pc = 0;
    cpu.x = 0xDF;
    let status = cpu.p;
    cpu.stx(0x8E).unwrap();

    let res = bus.borrow().read_byte(0x500).unwrap();
    assert_eq!(cpu.x, 0xDF);
    assert_eq!(res, 0xDF);
    assert_eq!(status, cpu.p);
}

#[test]
fn test_txa_zero() {
    let mut cpu = setup().1;

    cpu.a = 4;
    cpu.x = 0;
    cpu.txa().unwrap();

    assert_eq!(cpu.x, 0);
    assert_eq!(cpu.a, 0);
    assert!(cpu.p.z());
    assert!(!cpu.p.n());
}

#[test]
fn test_txa_positive() {
    let mut cpu = setup().1;

    cpu.a = 0;
    cpu.x = 4;
    cpu.txa().unwrap();

    assert_eq!(cpu.x, 4);
    assert_eq!(cpu.a, 4);
    assert!(!cpu.p.z());
    assert!(!cpu.p.n());
}

#[test]
fn test_txa_negative() {
    let mut cpu = setup().1;

    cpu.a = 0;
    cpu.x = 0x81;
    cpu.txa().unwrap();

    assert_eq!(cpu.x, 0x81);
    assert_eq!(cpu.a, 0x81);
    assert!(!cpu.p.z());
    assert!(cpu.p.n());
}

#[test]
fn test_txs() {
    let mut cpu = setup().1;

    cpu.sp = 0xEF;
    cpu.x = 0x20;
    let status = cpu.p;
    cpu.txs().unwrap();

    assert_eq!(cpu.sp, 0x20);
    assert_eq!(cpu.x, 0x20);
    assert_eq!(status, cpu.p);
}
