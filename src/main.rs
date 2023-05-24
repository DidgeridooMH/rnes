use clap::Parser;
use rnes::core::Nes;
use rnes::window::screen::ScreenBuffer;
use rnes::window::MainWindow;
use std::sync::{Arc, Mutex};
use winit::{event::*, event_loop::EventLoop};

#[derive(Parser, Debug)]
#[command(name = "RNES", author, version, about)]
struct Args {
    #[arg(short, long)]
    rom: String,
    #[arg(long)]
    show_ops: bool,
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

    let screen = Arc::new(Mutex::new(Box::<ScreenBuffer>::default()));

    let nes_screen = screen.clone();
    tokio::spawn(async move {
        let cli = Args::parse();
        let mut nes = Nes::new(&cli.rom, cli.show_ops).unwrap();
        loop {
            if let Err(e) = nes.emulate(&nes_screen) {
                eprintln!("Problem with emulation: {e}");
                break;
            }
        }
    });

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
            let screen = screen.lock().unwrap();
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
