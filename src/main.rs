use clap::Parser;
use rnes::window::MainWindow;
use rnes::{core::Nes, window::NATIVE_RESOLUTION};
use winit::event_loop::ControlFlow;
use winit::{event::*, event_loop::EventLoop};

#[derive(Parser, Debug)]
#[command(name = "RNES", author, version, about)]
struct Args {
    #[arg(short, long)]
    rom: String,
    #[arg(long)]
    show_ops: bool,
    #[arg(long)]
    show_header: bool,
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

    let mut screen =
        vec![0u32; NATIVE_RESOLUTION.width as usize * NATIVE_RESOLUTION.height as usize];
    let cli = Args::parse();
    let mut nes = Nes::new(&cli.rom, cli.show_ops, cli.show_header).unwrap();

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
            // TODO: may need to use PPU cycles to be more accurate. Drift may occur.
            if let Err(e) = nes.emulate(29780, &mut screen) {
                eprintln!("{e}");
                *control_flow = ControlFlow::Exit;
            }
            match window.render(&screen) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{e:?}");
                }
            }
        }
        Event::MainEventsCleared => {
            window.window.request_redraw();
        }
        _ => {}
    });
}
