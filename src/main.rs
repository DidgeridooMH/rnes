use rnes::core::{cpu::CPU, Bus};

fn main() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    loop {
        match cpu.tick() {
            Ok(_cycle_count) => {}
            Err(e) => eprintln!("{}", e),
        }
    }
}
