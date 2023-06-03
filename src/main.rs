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

    event_loop.run(move |event, _, control_flow| match event {
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

            if let Err(e) = nes.emulate(29780, &mut screen, &window) {
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
