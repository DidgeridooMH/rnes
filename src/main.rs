use rnes::core::{Bus, CPU, PPU};
use rnes::rom::{load_rom, RomHeader};
use std::println;
use std::{cell::RefCell, fs, rc::Rc};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "RNES", author, version, about)]
struct Args {
    #[arg(short, long)]
    rom: String,
    #[arg(long)]
    show_ops: bool,
    #[arg(long)]
    rom_header: bool,
}

fn main() {
    let cli = Args::parse();

    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);
    cpu.set_show_ops(cli.show_ops);

    let ppu = Rc::new(RefCell::new(PPU::new()));
    bus.borrow_mut()
        .register_region(0x2000..=0x2007, ppu.clone());

    let rom_file = match fs::read(cli.rom) {
        Ok(f) => f,
        _ => {
            eprintln!("Unable to read rom file.");
            return;
        }
    };

    if cli.rom_header {
        match RomHeader::from_slice(&rom_file[0..16]) {
            Ok(h) => println!("{:?}", h),
            Err(e) => println!("{}", e),
        }
        return;
    }

    if let Err(e) = load_rom(&rom_file, &bus) {
        eprintln!("Error while loading rom: {e}");
        return;
    }

    loop {
        match cpu.tick() {
            Ok(cycle_count) => {
                for _ in 0..(cycle_count * 3) {
                    if ppu.borrow_mut().tick() {
                        cpu.generate_nmi();
                    }
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                break;
            }
        }
    }
}
