use std::time::{Duration, Instant};

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

    let mut gamepad = Gilrs::new().unwrap();

    let mut last_frame = Instant::now();
    let mut accumulator = Duration::ZERO;
    let frame_time = Duration::from_nanos(16_666_667);

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

                // let now = Instant::now();
                // let delta = now - last_frame;
                // last_frame = now;
                // accumulator += delta;

                // while accumulator >= frame_time {
                const CPU_CYCLES_PER_FRAME: usize = 29780;
                if let Err(e) = nes.emulate(CPU_CYCLES_PER_FRAME, &mut screen, &window) {
                    eprintln!("{e}");
                    *control_flow = ControlFlow::Exit;
                }
                // accumulator -= frame_time;
                // }
                window.window.request_redraw();
            }
            _ => {}
        }
    });
}
