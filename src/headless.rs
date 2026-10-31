//! Headless runner: emulates a fixed number of frames with scripted input and
//! dumps selected frames as PNG files. Useful for debugging and regression
//! testing without a window.

use crate::core::Nes;
use crate::window::NATIVE_RESOLUTION;
use std::fs;
use std::path::{Path, PathBuf};

pub struct HeadlessOptions {
    pub frames: u32,
    /// Input script: comma separated `frame:buttons` entries. Buttons are any of
    /// `A B s(elect) S(tart) U D L R`; an empty set releases everything. The
    /// state holds until the next entry. Example: `120:S,125:,300:A,305:`.
    pub input: Option<String>,
    /// Frames to dump: comma separated numbers or ranges `a-b` or `a-b/step`.
    pub dump: Option<String>,
    pub out_dir: PathBuf,
    /// RAM addresses (hex) to print every frame.
    pub watch: Vec<u16>,
    /// Trace PPU register writes during this inclusive frame range.
    pub trace: Option<(u32, u32)>,
    /// Also write all dumped frames tiled into a single `sheet.png`.
    pub sheet: bool,
}

const SHEET_COLUMNS: usize = 6;

fn parse_buttons(s: &str) -> Result<u8, String> {
    let mut b = 0u8;
    for c in s.chars() {
        b |= match c {
            'A' => 1 << 0,
            'B' => 1 << 1,
            's' => 1 << 2,
            'S' => 1 << 3,
            'U' => 1 << 4,
            'D' => 1 << 5,
            'L' => 1 << 6,
            'R' => 1 << 7,
            _ => return Err(format!("Unknown button '{c}'")),
        };
    }
    Ok(b)
}

fn parse_input(script: &str) -> Result<Vec<(u32, u8)>, String> {
    let mut events = Vec::new();
    for entry in script.split(',').map(str::trim).filter(|e| !e.is_empty()) {
        let (frame, buttons) = entry
            .split_once(':')
            .ok_or_else(|| format!("Bad input entry '{entry}'"))?;
        let frame = frame
            .parse::<u32>()
            .map_err(|_| format!("Bad frame in '{entry}'"))?;
        events.push((frame, parse_buttons(buttons)?));
    }
    events.sort_by_key(|e| e.0);
    Ok(events)
}

fn parse_dump(spec: &str) -> Result<Vec<u32>, String> {
    let mut frames = Vec::new();
    for part in spec.split(',').map(str::trim).filter(|e| !e.is_empty()) {
        let (range, step) = match part.split_once('/') {
            Some((r, s)) => (r, s.parse::<u32>().map_err(|_| format!("Bad step '{part}'"))?),
            None => (part, 1),
        };
        match range.split_once('-') {
            Some((a, b)) => {
                let a = a.parse::<u32>().map_err(|_| format!("Bad range '{part}'"))?;
                let b = b.parse::<u32>().map_err(|_| format!("Bad range '{part}'"))?;
                frames.extend((a..=b).step_by(step.max(1) as usize));
            }
            None => frames.push(range.parse().map_err(|_| format!("Bad frame '{part}'"))?),
        }
    }
    frames.sort();
    frames.dedup();
    Ok(frames)
}

pub fn run(nes: &mut Nes, opts: &HeadlessOptions) -> Result<(), String> {
    crate::audio::AUDIO_DISABLED.store(true, std::sync::atomic::Ordering::Relaxed);

    let input = match &opts.input {
        Some(s) => parse_input(s)?,
        None => Vec::new(),
    };
    let dump = match &opts.dump {
        Some(s) => parse_dump(s)?,
        None => vec![opts.frames],
    };
    fs::create_dir_all(&opts.out_dir).map_err(|e| e.to_string())?;

    let mut screen =
        vec![0u32; NATIVE_RESOLUTION.width as usize * NATIVE_RESOLUTION.height as usize];
    let mut next_input = 0;
    let mut next_dump = 0;
    let mut sheet_frames: Vec<Vec<u32>> = Vec::new();

    for frame in 1..=opts.frames {
        while next_input < input.len() && input[next_input].0 <= frame {
            nes.controller.borrow_mut().set_buttons(input[next_input].1);
            next_input += 1;
        }

        if let Some((a, b)) = opts.trace {
            crate::core::TRACE_WRITES
                .store((a..=b).contains(&frame), std::sync::atomic::Ordering::Relaxed);
        }
        nes.run_frame(&mut screen)?;

        if !opts.watch.is_empty() {
            let values: Vec<String> = opts
                .watch
                .iter()
                .map(|&a| format!("{a:04X}={:02X}", nes.peek(a)))
                .collect();
            println!("frame {frame}: {}", values.join(" "));
        }

        while next_dump < dump.len() && dump[next_dump] <= frame {
            if dump[next_dump] == frame {
                let path = opts.out_dir.join(format!("frame_{frame:05}.png"));
                write_png(&path, &screen, NATIVE_RESOLUTION.width, NATIVE_RESOLUTION.height)?;
                if opts.sheet {
                    sheet_frames.push(screen.clone());
                }
            }
            next_dump += 1;
        }
    }

    if opts.sheet && !sheet_frames.is_empty() {
        write_sheet(&opts.out_dir.join("sheet.png"), &sheet_frames)?;
    }

    Ok(())
}

/// Tiles frames into a grid separated by 2px magenta gutters.
fn write_sheet(path: &Path, frames: &[Vec<u32>]) -> Result<(), String> {
    const GUTTER: usize = 2;
    let (fw, fh) = (NATIVE_RESOLUTION.width as usize, NATIVE_RESOLUTION.height as usize);
    let cols = SHEET_COLUMNS.min(frames.len());
    let rows = frames.len().div_ceil(cols);
    let (w, h) = (cols * (fw + GUTTER) - GUTTER, rows * (fh + GUTTER) - GUTTER);
    let mut sheet = vec![0xFFFF00FFu32; w * h];
    for (i, frame) in frames.iter().enumerate() {
        let (ox, oy) = ((i % cols) * (fw + GUTTER), (i / cols) * (fh + GUTTER));
        for y in 0..fh {
            let dst = (oy + y) * w + ox;
            sheet[dst..dst + fw].copy_from_slice(&frame[y * fw..(y + 1) * fw]);
        }
    }
    write_png(path, &sheet, w as u32, h as u32)
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let crc = crc32(&out[start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

/// Writes the screen buffer as an RGB PNG using uncompressed deflate blocks.
/// Pixels are 0xAABBGGRR (the PPU palette format).
pub fn write_png(path: &Path, screen: &[u32], w: u32, h: u32) -> Result<(), String> {
    let mut raw = Vec::with_capacity((w * 3 + 1) as usize * h as usize);
    for y in 0..h {
        raw.push(0); // filter: none
        for x in 0..w {
            // Screen pixels are stored as 0xAABBGGRR (little-endian RGBA bytes).
            let p = screen[(y * w + x) as usize].to_le_bytes();
            raw.extend_from_slice(&p[..3]);
        }
    }

    let mut zlib = vec![0x78, 0x01];
    let blocks: Vec<&[u8]> = raw.chunks(65535).collect();
    for (i, block) in blocks.iter().enumerate() {
        zlib.push((i == blocks.len() - 1) as u8);
        let len = block.len() as u16;
        zlib.extend_from_slice(&len.to_le_bytes());
        zlib.extend_from_slice(&(!len).to_le_bytes());
        zlib.extend_from_slice(block);
    }
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&w.to_be_bytes());
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    chunk(&mut png, b"IHDR", &ihdr);
    chunk(&mut png, b"IDAT", &zlib);
    chunk(&mut png, b"IEND", &[]);

    fs::write(path, png).map_err(|e| e.to_string())
}
