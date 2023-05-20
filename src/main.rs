use rnes::core::{cpu::CPU, Bus};
use rnes::rom::load_rom;
use std::{fs, cell::RefCell, rc::Rc};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "RNES", author, version, about)]
struct Args {
    #[arg(short, long)]
    rom: String,
    #[arg(long)]
    show_ops: bool
}

struct PPUMock;
use rnes::core::Addressable;
impl Addressable for PPUMock {
    fn read_byte(&self, _address: u16) -> u8 { 
        0xFF
    }
    fn write_byte(&mut self, _address: u16, _data: u8) { }
}

fn main() {
    let cli = Args::parse();

    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);
    cpu.set_show_ops(cli.show_ops);

    let rom_file = match fs::read(cli.rom) {
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

    bus.borrow_mut().register_region(0x2000..=0x2007, Rc::new(RefCell::new(PPUMock {})));

    loop {
        match cpu.tick() {
            Ok(_cycle_count) => {
            }
            Err(e) => {
                eprintln!("{}", e);
                break;
            }
        }
    }
}
