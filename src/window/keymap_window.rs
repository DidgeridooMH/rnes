//! The Options > Key Mapping window. Shows the controller overlay, lets the
//! user click a button and press a key to rebind it, and lights up buttons
//! while their keys are held.

use super::keymap_view::{Canvas, KeyMapView, Target, BACKGROUND, CANVAS_HEIGHT, CANVAS_WIDTH};
use crate::keymap::{Button, KeyMap};
use std::num::NonZeroU32;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, MouseButton, VirtualKeyCode, WindowEvent},
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowBuilder},
};

const IDLE_STATUS: &str = "Click a button to rebind it. Hold keys to test.";

pub struct KeyMapWindow {
    // Fields drop in declaration order: the surface and context must go
    // before the window whose handle they hold.
    surface: softbuffer::Surface,
    _context: softbuffer::Context,
    pub window: Window,
    canvas: Canvas,
    view: KeyMapView,
    cursor: Option<(f64, f64)>,
}

/// What the key mapping window asks the app to do after an event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyMapEvent {
    None,
    /// The key map was changed and should be saved.
    Changed,
    Close,
}

impl KeyMapWindow {
    pub fn new(target: &EventLoopWindowTarget<()>) -> Result<Self, String> {
        let window = WindowBuilder::new()
            .with_title("RNES - Key Mapping")
            .with_inner_size(PhysicalSize::new(
                CANVAS_WIDTH as u32 * 2,
                CANVAS_HEIGHT as u32 * 2,
            ))
            .with_min_inner_size(PhysicalSize::new(CANVAS_WIDTH as u32, CANVAS_HEIGHT as u32))
            .build(target)
            .map_err(|e| e.to_string())?;
        // SAFETY: the surface and context are stored alongside the window and
        // dropped before it (see the field order on `KeyMapWindow`).
        let context = unsafe { softbuffer::Context::new(&window) }.map_err(|e| e.to_string())?;
        let surface =
            unsafe { softbuffer::Surface::new(&context, &window) }.map_err(|e| e.to_string())?;

        Ok(Self {
            window,
            surface,
            _context: context,
            canvas: Canvas::new(CANVAS_WIDTH, CANVAS_HEIGHT),
            view: {
                let mut view = KeyMapView::default();
                view.status = IDLE_STATUS.into();
                view
            },
            cursor: None,
        })
    }

    pub fn is_listening(&self) -> bool {
        self.view.listening.is_some()
    }

    /// Advances the "press a key" blink. Returns true if a redraw is needed.
    pub fn tick(&mut self, now_ms: u128) -> bool {
        if self.view.listening.is_none() {
            return false;
        }
        let blink = (now_ms / 400) % 2 == 0;
        let changed = blink != self.view.blink;
        self.view.blink = blink;
        changed
    }

    pub fn set_status(&mut self, status: impl Into<String>) {
        self.view.status = status.into();
        self.window.request_redraw();
    }

    /// Integer scale and offset used to fit the canvas in the window.
    fn layout(&self) -> (usize, usize, usize, usize, usize) {
        let size = self.window.inner_size();
        let (w, h) = (size.width.max(1) as usize, size.height.max(1) as usize);
        let scale = (w / CANVAS_WIDTH).min(h / CANVAS_HEIGHT).max(1);
        let ox = w.saturating_sub(CANVAS_WIDTH * scale) / 2;
        let oy = h.saturating_sub(CANVAS_HEIGHT * scale) / 2;
        (w, h, scale, ox, oy)
    }

    fn target_at_cursor(&self) -> Option<Target> {
        let (x, y) = self.cursor?;
        let (_, _, scale, ox, oy) = self.layout();
        let cx = (x as i32 - ox as i32).div_euclid(scale as i32);
        let cy = (y as i32 - oy as i32).div_euclid(scale as i32);
        self.view.target_at(cx, cy)
    }

    pub fn handle_event(&mut self, event: &WindowEvent, map: &mut KeyMap) -> KeyMapEvent {
        match event {
            WindowEvent::CloseRequested => return KeyMapEvent::Close,
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                self.window.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = Some((position.x, position.y));
                self.update_hover();
            }
            WindowEvent::CursorLeft { .. } => {
                self.cursor = None;
                self.update_hover();
            }
            WindowEvent::Focused(false) => {
                self.view.pressed = 0;
                self.window.request_redraw();
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => match self.target_at_cursor() {
                Some(Target::Button(button)) => {
                    self.view.listening = Some(button);
                    self.view.blink = true;
                    self.view.status =
                        format!("Press a key for {}. Esc cancels.", button.name());
                    self.window.request_redraw();
                }
                Some(Target::Reset) => {
                    *map = KeyMap::default();
                    self.view.listening = None;
                    self.view.status = "Restored the default keys.".into();
                    self.window.request_redraw();
                    return KeyMapEvent::Changed;
                }
                None => {
                    if self.view.listening.take().is_some() {
                        self.view.status = IDLE_STATUS.into();
                        self.window.request_redraw();
                    }
                }
            },
            WindowEvent::KeyboardInput { input, .. } => {
                let Some(key) = input.virtual_keycode else {
                    return KeyMapEvent::None;
                };
                let pressed = input.state == ElementState::Pressed;
                if let Some(button) = self.view.listening {
                    if !pressed {
                        return KeyMapEvent::None;
                    }
                    self.view.listening = None;
                    self.window.request_redraw();
                    if key == VirtualKeyCode::Escape {
                        self.view.status = IDLE_STATUS.into();
                        return KeyMapEvent::None;
                    }
                    map.bind(button, key);
                    self.view.status = format!(
                        "{} is now {}.",
                        button.name(),
                        crate::keymap::key_label(key)
                    );
                    return KeyMapEvent::Changed;
                }
                if key == VirtualKeyCode::Escape && pressed {
                    return KeyMapEvent::Close;
                }
                let mask = map.buttons_for(key);
                if mask != 0 {
                    if pressed {
                        self.view.pressed |= mask;
                    } else {
                        self.view.pressed &= !mask;
                    }
                    self.window.request_redraw();
                }
            }
            _ => {}
        }
        KeyMapEvent::None
    }

    fn update_hover(&mut self) {
        let hovered = self.target_at_cursor();
        if hovered != self.view.hovered {
            self.view.hovered = hovered;
            self.window.request_redraw();
        }
    }

    pub fn render(&mut self, map: &KeyMap) {
        self.view.draw(map, &mut self.canvas);

        let (w, h, scale, ox, oy) = self.layout();
        let (Some(nw), Some(nh)) = (NonZeroU32::new(w as u32), NonZeroU32::new(h as u32)) else {
            return;
        };
        if let Err(e) = self.surface.resize(nw, nh) {
            eprintln!("Key mapping window resize failed: {e}");
            return;
        }
        let mut buffer = match self.surface.buffer_mut() {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Key mapping window draw failed: {e}");
                return;
            }
        };
        buffer.fill(BACKGROUND);
        for cy in 0..CANVAS_HEIGHT {
            let src = &self.canvas.pixels[cy * CANVAS_WIDTH..(cy + 1) * CANVAS_WIDTH];
            for sy in 0..scale {
                let y = oy + cy * scale + sy;
                if y >= h {
                    break;
                }
                let row = &mut buffer[y * w..(y + 1) * w];
                for (cx, &pixel) in src.iter().enumerate() {
                    let x0 = ox + cx * scale;
                    if x0 >= w {
                        break;
                    }
                    let x1 = (x0 + scale).min(w);
                    row[x0..x1].fill(pixel);
                }
            }
        }
        if let Err(e) = buffer.present() {
            eprintln!("Key mapping window present failed: {e}");
        }
    }
}

/// Renders the overlay in a few states, stacked vertically, for previewing
/// without a window. Pixels are 0xRRGGBB.
pub fn preview(map: &KeyMap) -> Canvas {
    let states: [fn(&mut KeyMapView); 3] = [
        |v| v.status = IDLE_STATUS.into(),
        |v| {
            v.hovered = Some(Target::Button(Button::A));
            v.pressed = Button::Up.bit() | Button::B.bit();
            v.status = "Hover A, holding Up and B".into();
        },
        |v| {
            v.listening = Some(Button::Start);
            v.blink = true;
            v.hovered = Some(Target::Reset);
            v.status = "Press a key for START. Esc cancels.".into();
        },
    ];
    let mut out = Canvas::new(CANVAS_WIDTH, CANVAS_HEIGHT * states.len());
    for (i, setup) in states.iter().enumerate() {
        let mut view = KeyMapView::default();
        setup(&mut view);
        let mut canvas = Canvas::new(CANVAS_WIDTH, CANVAS_HEIGHT);
        view.draw(map, &mut canvas);
        let start = i * CANVAS_WIDTH * CANVAS_HEIGHT;
        out.pixels[start..start + canvas.pixels.len()].copy_from_slice(&canvas.pixels);
    }
    out
}
