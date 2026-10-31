//! Software-drawn key mapping overlay: an NES controller with each button's
//! bound keys called out. Drawing is independent of any window so it can be
//! rendered headlessly.

use crate::keymap::{key_label, Button, KeyMap};
use font8x8::{UnicodeFonts, BASIC_FONTS};

pub const CANVAS_WIDTH: usize = 560;
pub const CANVAS_HEIGHT: usize = 240;

pub const BACKGROUND: u32 = 0x1b1d26;
const BODY: u32 = 0xd4d4d4;
const BODY_EDGE: u32 = 0x7d7d7d;
const FACE: u32 = 0x1c1c1c;
const DPAD: u32 = 0x0b0b0b;
const DPAD_EDGE: u32 = 0x4a4a4a;
const PANEL: u32 = 0xc9c9c9;
const STRIPE: u32 = 0x8f8f8f;
const RED: u32 = 0xc8102e;
const RED_RIM: u32 = 0x7e0a1d;
const PILL: u32 = 0x111111;
const HIGHLIGHT: u32 = 0xffd75e;
const HOVER: u32 = 0xffe9a8;
const PRESSED: u32 = 0x5ee08a;
const LABEL_BG: u32 = 0xf2f2f2;
const LABEL_TEXT: u32 = 0x1b1d26;
const LABEL_EMPTY: u32 = 0x8a8a8a;
const LEADER: u32 = 0x9aa0b4;
const TITLE: u32 = 0xe8e8e8;
const STATUS: u32 = 0xa0a4b8;

/// A 0xRRGGBB pixel buffer with a few drawing primitives.
pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height],
        }
    }

    fn put(&mut self, x: i32, y: i32, color: u32) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.pixels[y as usize * self.width + x as usize] = color;
        }
    }

    fn fill(&mut self, color: u32) {
        self.pixels.fill(color);
    }

    fn rect(&mut self, r: Rect, color: u32) {
        for y in r.y..r.y + r.h {
            for x in r.x..r.x + r.w {
                self.put(x, y, color);
            }
        }
    }

    fn rounded(&mut self, r: Rect, radius: i32, color: u32) {
        let radius = radius.min((r.w - 1) / 2).min((r.h - 1) / 2).max(0);
        for y in r.y..r.y + r.h {
            for x in r.x..r.x + r.w {
                // Distance from the nearest corner circle center.
                let cx = x.clamp(r.x + radius, r.x + r.w - 1 - radius);
                let cy = y.clamp(r.y + radius, r.y + r.h - 1 - radius);
                let (dx, dy) = (x - cx, y - cy);
                if dx * dx + dy * dy <= radius * radius {
                    self.put(x, y, color);
                }
            }
        }
    }

    /// Rounded rectangle with a border of `width` pixels.
    fn panel(&mut self, r: Rect, radius: i32, border: u32, fill: u32, width: i32) {
        self.rounded(r, radius, border);
        self.rounded(r.inset(width), (radius - width).max(0), fill);
    }

    fn circle(&mut self, cx: i32, cy: i32, radius: i32, color: u32) {
        for y in -radius..=radius {
            for x in -radius..=radius {
                if x * x + y * y <= radius * radius + radius / 2 {
                    self.put(cx + x, cy + y, color);
                }
            }
        }
    }

    fn hline(&mut self, x0: i32, x1: i32, y: i32, color: u32) {
        for x in x0.min(x1)..=x0.max(x1) {
            self.put(x, y, color);
        }
    }

    fn vline(&mut self, x: i32, y0: i32, y1: i32, color: u32) {
        for y in y0.min(y1)..=y0.max(y1) {
            self.put(x, y, color);
        }
    }

    /// Small triangle pointing in `dir` (0=up, 1=down, 2=left, 3=right).
    fn arrow(&mut self, cx: i32, cy: i32, dir: u8, color: u32) {
        for i in 0..4 {
            for j in -i..=i {
                let (x, y) = match dir {
                    0 => (cx + j, cy - 2 + i),
                    1 => (cx + j, cy + 2 - i),
                    2 => (cx - 2 + i, cy + j),
                    _ => (cx + 2 - i, cy + j),
                };
                self.put(x, y, color);
            }
        }
    }

    fn text(&mut self, x: i32, y: i32, text: &str, color: u32, scale: i32) {
        for (i, c) in text.chars().enumerate() {
            let glyph = BASIC_FONTS.get(c).or_else(|| BASIC_FONTS.get('?')).unwrap();
            let gx = x + i as i32 * 8 * scale;
            for (row, bits) in glyph.iter().enumerate() {
                for col in 0..8 {
                    if bits & (1 << col) != 0 {
                        for sy in 0..scale {
                            for sx in 0..scale {
                                self.put(
                                    gx + col * scale + sx,
                                    y + row as i32 * scale + sy,
                                    color,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

fn text_width(text: &str, scale: i32) -> i32 {
    text.chars().count() as i32 * 8 * scale
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    fn inset(self, n: i32) -> Self {
        Rect::new(self.x + n, self.y + n, self.w - 2 * n, self.h - 2 * n)
    }

    fn grow(self, n: i32) -> Self {
        self.inset(-n)
    }

    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.w && y < self.y + self.h
    }
}

/// Something in the overlay that can be clicked.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Target {
    Button(Button),
    Reset,
}

// Controller geometry.
const BODY_RECT: Rect = Rect::new(100, 60, 360, 120);
const FACE_RECT: Rect = Rect::new(112, 78, 336, 90);
const DPAD_X: i32 = 166;
const DPAD_Y: i32 = 123;
const ARM: i32 = 18;
const SELECT_X: i32 = 246;
const START_X: i32 = 314;
const B_X: i32 = 374;
const A_X: i32 = 418;
const AB_Y: i32 = 122;
const LABEL_H: i32 = 14;
const TOP_ROW_Y: i32 = 30;
const BOTTOM_ROW_Y: i32 = 198;
const RESET_RECT: Rect = Rect::new(CANVAS_WIDTH as i32 - 84, 216, 76, 18);

fn dpad_arm(button: Button) -> Rect {
    let (cx, cy) = (DPAD_X - ARM / 2, DPAD_Y - ARM / 2);
    match button {
        Button::Up => Rect::new(cx, cy - ARM, ARM, ARM),
        Button::Down => Rect::new(cx, cy + ARM, ARM, ARM),
        Button::Left => Rect::new(cx - ARM, cy, ARM, ARM),
        _ => Rect::new(cx + ARM, cy, ARM, ARM),
    }
}

/// Clickable area of a button on the controller itself.
fn button_area(button: Button) -> Rect {
    match button {
        Button::Up | Button::Down | Button::Left | Button::Right => dpad_arm(button).grow(2),
        Button::Select => Rect::new(SELECT_X - 26, 116, 52, 36),
        Button::Start => Rect::new(START_X - 26, 116, 52, 36),
        Button::B => Rect::new(B_X - 18, AB_Y - 18, 36, 44),
        Button::A => Rect::new(A_X - 18, AB_Y - 18, 36, 44),
    }
}

/// Where a leader line starts for a label in the top or bottom row.
fn row_anchor(button: Button) -> (i32, i32) {
    match button {
        Button::Up => (DPAD_X, DPAD_Y - ARM * 3 / 2),
        Button::Down => (DPAD_X, DPAD_Y + ARM * 3 / 2),
        Button::Left => (DPAD_X - ARM * 3 / 2 + 3, DPAD_Y - ARM / 2),
        Button::Right => (DPAD_X + ARM * 3 / 2 - 3, DPAD_Y + ARM / 2),
        Button::Select => (SELECT_X, 117),
        Button::Start => (START_X, 117),
        Button::B => (B_X, AB_Y + 34),
        Button::A => (A_X, AB_Y + 34),
    }
}

/// Where a leader line starts for a label in a side column.
fn column_anchor(button: Button) -> (i32, i32) {
    match button {
        Button::Left => (DPAD_X - ARM * 3 / 2, DPAD_Y),
        _ => (A_X + 13, AB_Y),
    }
}

#[derive(Default)]
pub struct KeyMapView {
    pub hovered: Option<Target>,
    /// Button waiting for a new key.
    pub listening: Option<Button>,
    /// Buttons currently held (lit up so mappings can be tested).
    pub pressed: u8,
    /// Toggled over time while listening.
    pub blink: bool,
    pub status: String,
    hit_areas: Vec<(Rect, Target)>,
}

impl KeyMapView {
    pub fn target_at(&self, x: i32, y: i32) -> Option<Target> {
        self.hit_areas
            .iter()
            .find(|(r, _)| r.contains(x, y))
            .map(|(_, t)| *t)
    }

    /// Fill color override for a button, if it is highlighted.
    fn accent(&self, button: Button) -> Option<u32> {
        if self.listening == Some(button) {
            Some(if self.blink { HIGHLIGHT } else { HOVER })
        } else if self.pressed & button.bit() != 0 {
            Some(PRESSED)
        } else if self.hovered == Some(Target::Button(button)) {
            Some(HOVER)
        } else {
            None
        }
    }

    pub fn draw(&mut self, map: &KeyMap, canvas: &mut Canvas) {
        self.hit_areas.clear();
        canvas.fill(BACKGROUND);

        let title = "KEY MAPPING";
        canvas.text(
            (CANVAS_WIDTH as i32 - text_width(title, 2)) / 2,
            6,
            title,
            TITLE,
            2,
        );

        self.draw_controller(canvas);
        self.draw_labels(map, canvas);
        for button in Button::ALL {
            self.hit_areas.push((button_area(button), Target::Button(button)));
        }

        // Footer: status line and reset button.
        canvas.text(8, 221, &self.status, STATUS, 1);
        let reset_hover = self.hovered == Some(Target::Reset);
        canvas.panel(
            RESET_RECT,
            3,
            LEADER,
            if reset_hover { HOVER } else { LABEL_BG },
            1,
        );
        let label = "DEFAULTS";
        canvas.text(
            RESET_RECT.x + (RESET_RECT.w - text_width(label, 1)) / 2,
            RESET_RECT.y + 5,
            label,
            LABEL_TEXT,
            1,
        );
        self.hit_areas.push((RESET_RECT, Target::Reset));
    }

    fn draw_controller(&self, canvas: &mut Canvas) {
        canvas.panel(BODY_RECT, 8, BODY_EDGE, BODY, 2);
        canvas.rounded(FACE_RECT, 3, FACE);

        // D-pad: outline, cross, then per-arm highlights and arrows.
        let horizontal = Rect::new(DPAD_X - ARM * 3 / 2, DPAD_Y - ARM / 2, ARM * 3, ARM);
        let vertical = Rect::new(DPAD_X - ARM / 2, DPAD_Y - ARM * 3 / 2, ARM, ARM * 3);
        canvas.rounded(horizontal.grow(1), 3, DPAD_EDGE);
        canvas.rounded(vertical.grow(1), 3, DPAD_EDGE);
        canvas.rounded(horizontal, 2, DPAD);
        canvas.rounded(vertical, 2, DPAD);
        for (dir, button) in [Button::Up, Button::Down, Button::Left, Button::Right]
            .into_iter()
            .enumerate()
        {
            let arm = dpad_arm(button);
            if let Some(color) = self.accent(button) {
                canvas.rect(arm.inset(1), color);
            }
            let arrow = if self.accent(button).is_some() { DPAD } else { DPAD_EDGE };
            canvas.arrow(arm.x + ARM / 2, arm.y + ARM / 2, dir as u8, arrow);
        }
        canvas.circle(DPAD_X, DPAD_Y, 4, 0x161616);

        // Center stripes and the Select/Start panel.
        canvas.rect(Rect::new(216, 88, 128, 6), STRIPE);
        canvas.rect(Rect::new(216, 100, 128, 6), STRIPE);
        canvas.rounded(Rect::new(216, 114, 128, 46), 4, PANEL);
        for (button, x) in [(Button::Select, SELECT_X), (Button::Start, START_X)] {
            let name = button.name();
            canvas.text(x - text_width(name, 1) / 2, 120, name, RED, 1);
            let pill = Rect::new(x - 14, 137, 28, 10);
            canvas.rounded(pill.grow(1), 5, 0x6a6a6a);
            canvas.rounded(pill, 5, self.accent(button).unwrap_or(PILL));
        }

        // B and A buttons.
        for (button, x) in [(Button::B, B_X), (Button::A, A_X)] {
            canvas.rounded(Rect::new(x - 16, AB_Y - 16, 32, 32), 4, PANEL);
            canvas.circle(x, AB_Y, 12, RED_RIM);
            canvas.circle(x, AB_Y, 11, self.accent(button).unwrap_or(RED));
            canvas.text(x - 4, AB_Y + 22, button.name(), RED, 1);
        }
    }

    fn label_text(&self, map: &KeyMap, button: Button) -> (String, u32) {
        if self.listening == Some(button) {
            return ("press a key".into(), LABEL_TEXT);
        }
        let keys = map.keys(button);
        if keys.is_empty() {
            ("---".into(), LABEL_EMPTY)
        } else {
            let names: Vec<String> = keys.iter().map(|k| key_label(*k)).collect();
            (names.join(" / "), LABEL_TEXT)
        }
    }

    fn label_fill(&self, button: Button) -> u32 {
        if self.listening == Some(button) {
            if self.blink {
                HIGHLIGHT
            } else {
                LABEL_BG
            }
        } else if self.hovered == Some(Target::Button(button)) {
            HOVER
        } else {
            LABEL_BG
        }
    }

    fn leader_color(&self, button: Button) -> u32 {
        if self.listening == Some(button) || self.hovered == Some(Target::Button(button)) {
            HIGHLIGHT
        } else {
            LEADER
        }
    }

    fn draw_label(&mut self, canvas: &mut Canvas, button: Button, rect: Rect, text: &str, color: u32) {
        canvas.panel(rect, 3, LEADER, self.label_fill(button), 1);
        canvas.text(rect.x + 5, rect.y + 3, text, color, 1);
        self.hit_areas.push((rect, Target::Button(button)));
    }

    fn draw_labels(&mut self, map: &KeyMap, canvas: &mut Canvas) {
        let width_of = |view: &Self, b: Button| text_width(&view.label_text(map, b).0, 1) + 10;

        // Left and A normally sit in side columns; if their text is too wide
        // for the margin they join the top/bottom rows instead.
        let left_fits = width_of(self, Button::Left) <= BODY_RECT.x - 12;
        let a_fits = width_of(self, Button::A) <= CANVAS_WIDTH as i32 - (BODY_RECT.x + BODY_RECT.w) - 12;

        let mut top = vec![Button::Up, Button::Select, Button::Start];
        if !left_fits {
            top.insert(0, Button::Left);
        }
        let mut bottom = vec![Button::Down, Button::Right, Button::B];
        if !a_fits {
            bottom.push(Button::A);
        }

        // Rows: boxes spread out so they never overlap, with elbowed leader
        // lines back to their buttons.
        let rows = [
            (top, TOP_ROW_Y, BODY_RECT.y - 4, -3),
            (bottom, BOTTOM_ROW_Y, BODY_RECT.y + BODY_RECT.h + 3, 3),
        ];
        for (buttons, row_y, elbow_base, elbow_step) in rows {
            let labels: Vec<(Button, String, u32, i32)> = buttons
                .iter()
                .map(|&b| {
                    let (text, color) = self.label_text(map, b);
                    let width = text_width(&text, 1) + 10;
                    (b, text, color, width)
                })
                .collect();
            let centers = spread(
                &labels
                    .iter()
                    .map(|(b, _, _, w)| (row_anchor(*b).0, *w))
                    .collect::<Vec<_>>(),
            );
            for (i, ((button, text, color, width), cx)) in
                labels.into_iter().zip(centers).enumerate()
            {
                // Stagger elbows so neighbouring leaders never share a line.
                let elbow_y = elbow_base + elbow_step * i as i32;
                let (ax, ay) = row_anchor(button);
                let box_edge = if row_y < BODY_RECT.y { row_y + LABEL_H } else { row_y };
                let leader = self.leader_color(button);
                canvas.vline(ax, ay, elbow_y, leader);
                canvas.hline(ax, cx, elbow_y, leader);
                canvas.vline(cx, elbow_y, box_edge, leader);
                let rect = Rect::new(cx - width / 2, row_y, width, LABEL_H);
                self.draw_label(canvas, button, rect, &text, color);
            }
        }

        // Side columns.
        for (button, left_side, fits) in [(Button::Left, true, left_fits), (Button::A, false, a_fits)] {
            if !fits {
                continue;
            }
            let (text, color) = self.label_text(map, button);
            let width = text_width(&text, 1) + 10;
            let (ax, ay) = column_anchor(button);
            let x = if left_side {
                BODY_RECT.x - 8 - width
            } else {
                BODY_RECT.x + BODY_RECT.w + 8
            };
            let rect = Rect::new(x, ay - LABEL_H / 2, width, LABEL_H);
            let edge = if left_side { rect.x + rect.w } else { rect.x };
            canvas.hline(ax, edge, ay, self.leader_color(button));
            self.draw_label(canvas, button, rect, &text, color);
        }
    }
}

/// Positions boxes of the given widths as close to their desired centers as
/// possible without overlapping, keeping them on the canvas.
fn spread(items: &[(i32, i32)]) -> Vec<i32> {
    const GAP: i32 = 6;
    let mut centers: Vec<i32> = items.iter().map(|(c, _)| *c).collect();
    for i in 1..items.len() {
        let min = centers[i - 1] + (items[i - 1].1 + items[i].1) / 2 + GAP;
        centers[i] = centers[i].max(min);
    }
    // Pull back from the right edge if needed.
    let mut limit = CANVAS_WIDTH as i32 - 4;
    for i in (0..items.len()).rev() {
        let max = limit - items[i].1 / 2;
        centers[i] = centers[i].min(max);
        limit = centers[i] - items[i].1 / 2 - GAP;
    }
    centers
}
