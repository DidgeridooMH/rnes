use rnes::core::{cpu, Bus};

fn main() {
    let bus = Bus::new();
    let cpu = cpu::CPU::new(&bus);

    loop {
        match cpu.borrow_mut().tick() {
            Ok(_cycle_count) => {}
            Err(e) => eprintln!("{}", e),
        }
    }
}
