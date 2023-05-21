use clap::Parser;
use rnes::core::{Bus, CPU, PPU};
use rnes::rom::{load_rom, RomHeader};
use rnes::window::screen::{Pixel, ScreenBuffer};
use rnes::window::{MainWindow, BYTES_PER_PIXEL, NATIVE_RESOLUTION};
use std::println;
use std::{cell::RefCell, fs, rc::Rc};
use winit::event_loop::EventLoop;
use winit::{event::*, event_loop::ControlFlow};

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

/*
fn start_emulation() {
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

match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.window.id() => {
                if window.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        _ => {}
                    }
                }
            }

*/

fn update_screen(screen: &mut ScreenBuffer) {
    for y in 0..NATIVE_RESOLUTION.height as usize {
        for x in 0..NATIVE_RESOLUTION.width as usize {
            let p = &mut screen.buffer[y * NATIVE_RESOLUTION.width as usize + x];
            if p.r == 255 {
                p.r = 0;
            }
            p.r += 1;
            p.a = 0xFF;
        }
    }
}

#[tokio::main]
async fn main() {
    let event_loop = EventLoop::new();
    let mut window = match MainWindow::new(&event_loop).await {
        Ok(w) => w,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    let mut screen = Box::<ScreenBuffer>::default();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } => {
            if window_id == window.window.id() {
                window.input(event, control_flow);
            }
        }
        Event::RedrawRequested(window_id) if window_id == window.window.id() => {
            // TODO: window update
            update_screen(&mut screen);
            match window.render(&screen.buffer) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{:?}", e);
                }
            }
        }
        Event::MainEventsCleared => {
            window.window.request_redraw();
        }
        _ => {}
    });
}
