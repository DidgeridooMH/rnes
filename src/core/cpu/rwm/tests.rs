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
