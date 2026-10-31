use clap::Parser;
use gilrs::{EventType, Gilrs};
use rnes::{
    core::Nes,
    window::{AppMenu, MainWindow, MenuAction, NATIVE_RESOLUTION},
};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

#[derive(Parser, Debug)]
#[command(name = "RNES", author, version, about)]
struct Args {
    /// ROM to load at startup. If omitted, the emulator opens with no ROM
    /// loaded and one can be chosen from File > Open ROM.
    #[arg(short, long)]
    rom: Option<PathBuf>,
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
    out: PathBuf,
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
    sav: Option<PathBuf>,
}

fn load_sav(nes: &Nes, path: &Path) -> Result<(), String> {
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

/// Creates an emulator for `path`, loading the `--sav` file if one was given.
fn open_rom(path: &Path, cli: &Args) -> Result<Nes, String> {
    let nes = Nes::new(&path.to_string_lossy(), cli.show_ops, cli.show_header)?;
    if let Some(sav) = &cli.sav {
        load_sav(&nes, sav)?;
    }
    Ok(nes)
}

fn show_error(window: &MainWindow, message: &str) {
    eprintln!("{message}");
    rfd::MessageDialog::new()
        .set_parent(&window.dialog_parent())
        .set_level(rfd::MessageLevel::Error)
        .set_title("RNES")
        .set_description(message)
        .show();
}

/// Window-side state: the running emulator (if a ROM is loaded) and the UI
/// that reflects it.
struct App {
    nes: Option<Nes>,
    screen: Vec<u32>,
    window: MainWindow,
    menu: Option<AppMenu>,
}

impl App {
    fn load(&mut self, path: &Path, cli: &Args) {
        // Keep the current game running if the new ROM fails to load.
        match open_rom(path, cli) {
            Ok(nes) => {
                self.unload();
                self.nes = Some(nes);
                let name = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.display().to_string());
                self.window.set_rom_name(Some(name));
                if let Some(menu) = &self.menu {
                    menu.set_rom_loaded(true);
                }
            }
            Err(e) => show_error(&self.window, &format!("Unable to open {}:\n{e}", path.display())),
        }
    }

    fn unload(&mut self) {
        self.nes = None;
        self.screen.fill(0xFF000000);
        self.window.set_rom_name(None);
        if let Some(menu) = &self.menu {
            menu.set_rom_loaded(false);
        }
    }

    fn pick_and_load(&mut self, cli: &Args) {
        let file = rfd::FileDialog::new()
            .set_title("Open ROM")
            .set_parent(&self.window.dialog_parent())
            .add_filter("NES ROM", &["nes"])
            .add_filter("All files", &["*"])
            .pick_file();
        if let Some(path) = file {
            self.load(&path, cli);
        }
    }
}

fn run_headless(cli: &Args) -> ! {
    let Some(rom) = &cli.rom else {
        eprintln!("--headless requires --rom");
        std::process::exit(1);
    };
    let mut nes = match open_rom(rom, cli) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
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
    std::process::exit(0);
}

#[tokio::main]
async fn main() {
    let cli = Args::parse();
    if cli.headless {
        run_headless(&cli);
    }

    let event_loop = EventLoop::new();
    let window = match MainWindow::new(&event_loop).await {
        Ok(w) => w,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    let menu = match AppMenu::new(&window.window) {
        Ok(menu) => {
            // The menu bar takes space from the client area; restore it.
            window.reset_size();
            Some(menu)
        }
        Err(e) => {
            eprintln!("Unable to create menu bar: {e}");
            None
        }
    };

    let mut app = App {
        nes: None,
        screen: vec![
            0xFF000000u32;
            NATIVE_RESOLUTION.width as usize * NATIVE_RESOLUTION.height as usize
        ],
        window,
        menu,
    };
    app.unload();
    if let Some(rom) = cli.rom.clone() {
        app.load(&rom, &cli);
    }

    let mut gamepad = Gilrs::new().unwrap();
    let mut modifiers = ModifiersState::empty();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = if app.nes.is_some() {
            // Emulation is paced by audio playback.
            ControlFlow::Poll
        } else {
            ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(16))
        };

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == app.window.window.id() => {
                app.window.input(event, control_flow);
                match event {
                    WindowEvent::ModifiersChanged(state) => modifiers = *state,
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(keycode),
                                state,
                                ..
                            },
                        ..
                    } => {
                        let shortcut = if cfg!(target_os = "macos") {
                            modifiers.logo()
                        } else {
                            modifiers.ctrl()
                        };
                        if shortcut
                            && *keycode == VirtualKeyCode::O
                            && *state == ElementState::Pressed
                        {
                            app.pick_and_load(&cli);
                        } else if let Some(nes) = &app.nes {
                            nes.controller.borrow_mut().input_keyboard(keycode, state);
                        }
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == app.window.window.id() => {
                if let Err(e) = app.window.render(&app.screen) {
                    eprintln!("{e:?}");
                }
            }
            Event::MainEventsCleared => {
                match app.menu.as_ref().and_then(|m| m.poll()) {
                    Some(MenuAction::OpenRom) => app.pick_and_load(&cli),
                    Some(MenuAction::CloseRom) => app.unload(),
                    Some(MenuAction::Exit) => *control_flow = ControlFlow::Exit,
                    None => {}
                }

                while let Some(gilrs::Event { event, .. }) = gamepad.next_event() {
                    let Some(nes) = &app.nes else { continue };
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

                if let Some(nes) = &mut app.nes {
                    const CPU_CYCLES_PER_FRAME: usize = 29780;
                    if let Err(e) = nes.emulate(CPU_CYCLES_PER_FRAME, &mut app.screen, &app.window)
                    {
                        app.unload();
                        show_error(&app.window, &format!("Emulation stopped:\n{e}"));
                    }
                }
                app.window.window.request_redraw();
            }
            _ => {}
        }
    });
}
