use rnes::core::{cpu::CPU, Bus};

fn main() {
    let bus = Bus::new();
    let cpu = CPU::new(&bus);

    loop {
        match cpu.borrow_mut().tick() {
            Ok(_cycle_count) => {}
            Err(e) => eprintln!("{}", e),
        }
    }
}
