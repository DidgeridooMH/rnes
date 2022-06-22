use std::cell::RefCell;
use std::rc::Rc;

mod core;
use crate::core::{Bus, CPU};

fn main() {
    let bus = Rc::new(RefCell::new(Bus::new()));
    let _cpu = CPU::new(&bus);
}
