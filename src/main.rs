use clap::Parser;
use gilrs::{EventType, Gilrs};
use rnes::{
    core::Nes,
    window::{MainWindow, NATIVE_RESOLUTION},
};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

#[derive(Parser, Debug)]
#[command(name = "RNES", author, version, about)]
struct Args {
    #[arg(short, long)]
    rom: String,
    #[arg(long)]
    show_ops: bool,
    #[arg(long)]
    show_header: bool,
    /// Run without a window for a fixed number of frames and dump PNGs.
    #[arg(long)]
    headless: bool,
    /// Number of frames to run in headless mode.
    #[arg(long, default_value_t = 600)]
    frames: u32,
    /// Headless input script, e.g. "120:S,125:,300:A,305:" (buttons: A B s S U D L R).
    #[arg(long)]
    input: Option<String>,
    /// Headless frames to dump, e.g. "100,200-300/10". Defaults to the last frame.
    #[arg(long)]
    dump: Option<String>,
    /// Headless output directory for PNG dumps.
    #[arg(long, default_value = "headless_out")]
    out: std::path::PathBuf,
    /// Headless: comma separated hex RAM addresses to print every frame.
    #[arg(long, value_delimiter = ',', value_parser = parse_hex)]
    watch: Vec<u16>,
    /// Headless: trace PPU register writes for frames "A-B".
    #[arg(long, value_parser = parse_range)]
    trace: Option<(u32, u32)>,
    /// Headless: also tile all dumped frames into sheet.png.
    #[arg(long)]
    sheet: bool,
    /// Battery save file to load into PRG-RAM ($6000-$7FFF) at startup.
    #[arg(long)]
    sav: Option<std::path::PathBuf>,
}

fn load_sav(nes: &Nes, path: &std::path::Path) -> Result<(), String> {
    let data = std::fs::read(path).map_err(|e| format!("Unable to read save: {e}"))?;
    for (i, &byte) in data.iter().take(0x2000).enumerate() {
        nes.poke(0x6000 + i as u16, byte);
    }
    Ok(())
}

fn parse_range(s: &str) -> Result<(u32, u32), String> {
    let (a, b) = s.split_once('-').unwrap_or((s, s));
    Ok((
        a.parse().map_err(|_| "bad range".to_string())?,
        b.parse().map_err(|_| "bad range".to_string())?,
    ))
}

fn parse_hex(s: &str) -> Result<u16, String> {
    u16::from_str_radix(s.trim_start_matches('$'), 16).map_err(|e| e.to_string())
}

#[tokio::main]
async fn main() {
    let cli = Args::parse();
    if cli.headless {
        let mut nes = match Nes::new(&cli.rom, cli.show_ops, cli.show_header) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        };
        if let Some(sav) = &cli.sav {
            if let Err(e) = load_sav(&nes, sav) {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
        let opts = rnes::headless::HeadlessOptions {
            frames: cli.frames,
            input: cli.input.clone(),
            dump: cli.dump.clone(),
            out_dir: cli.out.clone(),
            watch: cli.watch.clone(),
            sheet: cli.sheet,
            trace: cli.trace,
        };
        if let Err(e) = rnes::headless::run(&mut nes, &opts) {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return;
    }

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
    let mut nes = Nes::new(&cli.rom, cli.show_ops, cli.show_header).unwrap();
    if let Some(sav) = &cli.sav {
        if let Err(e) = load_sav(&nes, sav) {
            eprintln!("{e}");
        }
    }

    let mut gamepad = Gilrs::new().unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } => {
                if window_id == window.window.id() {
                    window.input(event, control_flow);
                    if let WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(keycode),
                                state,
                                ..
                            },
                        ..
                    } = &event
                    {
                        nes.controller.borrow_mut().input_keyboard(keycode, state);
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.window.id() => {
                match window.render(&screen) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("{e:?}");
                    }
                }
            }
            Event::MainEventsCleared => {
                while let Some(gilrs::Event { event, .. }) = gamepad.next_event() {
                    match event {
                        EventType::ButtonPressed(button, ..) => {
                            nes.controller.borrow_mut().gamepad_press(button)
                        }
                        EventType::ButtonReleased(button, ..) => {
                            nes.controller.borrow_mut().gamepad_release(button)
                        }
                        _ => {}
                    }
                }

                const CPU_CYCLES_PER_FRAME: usize = 29780;
                if let Err(e) = nes.emulate(CPU_CYCLES_PER_FRAME, &mut screen, &window) {
                    eprintln!("{e}");
                    *control_flow = ControlFlow::Exit;
                }
                window.window.request_redraw();
            }
            _ => {}
        }
    });
}
