use rnes::core::{cpu::CPU, Bus};
use rnes::rom::load_rom;
use std::fs;

fn main() {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);

    let rom_file = match fs::read("roms/dk.nes") {
        Ok(f) => f,
        _ => {
            eprintln!("Unable to read rom file.");
            return;
        }
    };
    if let Err(e) = load_rom(&rom_file, &bus) {
        eprintln!("Error while loading rom: {e}");
        return;
    }

    loop {
        match cpu.tick() {
            Ok(cycle_count) => {
                println!("Tick {cycle_count}");
            }
            Err(e) => {
                eprintln!("{}", e);
                break;
            }
        }
    }
}
