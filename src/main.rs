mod core;
use crate::core::{cpu, Bus};

fn main() {
    let bus = Bus::new();
    let _cpu = cpu::CPU::new(&bus);
}
