//! Keyboard-to-controller mapping, persisted as `keymappings.toml` next to
//! the executable.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use winit::event::VirtualKeyCode;

pub const FILE_NAME: &str = "keymappings.toml";

/// NES controller buttons, in the order the controller shifts them out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}

impl Button {
    pub const ALL: [Button; 8] = [
        Button::A,
        Button::B,
        Button::Select,
        Button::Start,
        Button::Up,
        Button::Down,
        Button::Left,
        Button::Right,
    ];

    /// Bit for this button in the controller's shift register.
    pub fn bit(self) -> u8 {
        1 << self as u8
    }

    pub fn name(self) -> &'static str {
        match self {
            Button::A => "A",
            Button::B => "B",
            Button::Select => "SELECT",
            Button::Start => "START",
            Button::Up => "UP",
            Button::Down => "DOWN",
            Button::Left => "LEFT",
            Button::Right => "RIGHT",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyMap {
    keys: [Vec<VirtualKeyCode>; 8],
}

impl Default for KeyMap {
    fn default() -> Self {
        use VirtualKeyCode as K;
        Self {
            keys: [
                vec![K::K],
                vec![K::J],
                vec![K::Comma],
                vec![K::Period],
                vec![K::W],
                vec![K::S],
                vec![K::A],
                vec![K::D],
            ],
        }
    }
}

/// How the key map was obtained at startup.
#[derive(Debug, Clone, PartialEq)]
pub enum LoadStatus {
    Loaded,
    /// The file didn't exist and was written with the defaults.
    Created,
    /// The file didn't exist and couldn't be written; defaults are in use.
    CreateFailed(String),
}

impl KeyMap {
    pub fn keys(&self, button: Button) -> &[VirtualKeyCode] {
        &self.keys[button as usize]
    }

    /// Bitmask of every controller button bound to `key`.
    pub fn buttons_for(&self, key: VirtualKeyCode) -> u8 {
        Button::ALL
            .iter()
            .filter(|b| self.keys(**b).contains(&key))
            .fold(0, |mask, b| mask | b.bit())
    }

    /// Binds `key` as the only key for `button`, removing it from any other
    /// button so one key never drives two buttons by accident.
    pub fn bind(&mut self, button: Button, key: VirtualKeyCode) {
        for keys in self.keys.iter_mut() {
            keys.retain(|k| *k != key);
        }
        self.keys[button as usize] = vec![key];
    }

    /// `keymappings.toml` in the executable's directory.
    pub fn default_path() -> Option<PathBuf> {
        let exe = std::env::current_exe().ok()?;
        Some(exe.parent()?.join(FILE_NAME))
    }

    /// Loads the key map from `path`, creating it with the defaults if it
    /// doesn't exist. A file that exists but can't be parsed is an error and
    /// is left untouched.
    pub fn load_or_create(path: &Path) -> Result<(KeyMap, LoadStatus), String> {
        if !path.exists() {
            let map = KeyMap::default();
            let status = match map.save(path) {
                Ok(()) => LoadStatus::Created,
                Err(e) => LoadStatus::CreateFailed(e),
            };
            return Ok((map, status));
        }
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        Ok((KeyMap::from_toml(&text)?, LoadStatus::Loaded))
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        std::fs::write(path, self.to_toml()).map_err(|e| e.to_string())
    }

    pub fn from_toml(text: &str) -> Result<KeyMap, String> {
        let file: KeyMapFile = toml::from_str(text).map_err(|e| e.to_string())?;
        let k = file.keyboard;
        let mut map = KeyMap::default();
        let fields = [k.a, k.b, k.select, k.start, k.up, k.down, k.left, k.right];
        for (slot, field) in map.keys.iter_mut().zip(fields) {
            // Missing buttons keep their default; `[]` leaves one unbound.
            if let Some(keys) = field {
                *slot = keys.into_vec();
            }
        }
        Ok(map)
    }

    pub fn to_toml(&self) -> String {
        let keys = |b: Button| Some(Keys::from_slice(self.keys(b)));
        let file = KeyMapFile {
            keyboard: KeyboardSection {
                a: keys(Button::A),
                b: keys(Button::B),
                select: keys(Button::Select),
                start: keys(Button::Start),
                up: keys(Button::Up),
                down: keys(Button::Down),
                left: keys(Button::Left),
                right: keys(Button::Right),
            },
        };
        let body = toml::to_string(&file).expect("key map serializes");
        format!("{HEADER}\n{body}")
    }
}

const HEADER: &str = "\
# RNES key mappings.
#
# Each NES button takes one key name or a list of them, e.g.
#   a = \"K\"
#   up = [\"W\", \"Up\"]
# Use [] to leave a button unbound. Buttons left out use their default key.
#
# Key names: A-Z, Key0-Key9, Up, Down, Left, Right, Space, Return, Back, Tab,
# Escape, LShift, RShift, LControl, RControl, LAlt, RAlt, Comma, Period,
# Slash, Semicolon, Apostrophe, LBracket, RBracket, Minus, Equals,
# Numpad0-Numpad9, F1-F12, and the other winit VirtualKeyCode names.
#
# This file is rewritten when keys are changed from Options > Key Mapping.
";

#[derive(Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct KeyMapFile {
    #[serde(default)]
    keyboard: KeyboardSection,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct KeyboardSection {
    a: Option<Keys>,
    b: Option<Keys>,
    select: Option<Keys>,
    start: Option<Keys>,
    up: Option<Keys>,
    down: Option<Keys>,
    left: Option<Keys>,
    right: Option<Keys>,
}

/// A single key or a list of keys.
#[derive(Serialize, Deserialize)]
#[serde(untagged, try_from = "RawKeys")]
enum Keys {
    One(VirtualKeyCode),
    Many(Vec<VirtualKeyCode>),
}

/// Keys as written in the file, before the names are validated. Parsing
/// through this gives "unknown key" errors instead of serde's generic
/// untagged-enum message.
#[derive(Deserialize)]
#[serde(untagged)]
enum RawKeys {
    One(String),
    Many(Vec<String>),
}

fn parse_key(name: &str) -> Result<VirtualKeyCode, String> {
    use serde::de::{value::StrDeserializer, IntoDeserializer};
    let deserializer: StrDeserializer<serde::de::value::Error> = name.into_deserializer();
    VirtualKeyCode::deserialize(deserializer).map_err(|_| {
        format!("unknown key \"{name}\" (see the list of key names at the top of the file)")
    })
}

impl TryFrom<RawKeys> for Keys {
    type Error = String;

    fn try_from(raw: RawKeys) -> Result<Self, String> {
        Ok(match raw {
            RawKeys::One(name) => Keys::One(parse_key(&name)?),
            RawKeys::Many(names) => Keys::Many(
                names
                    .iter()
                    .map(|n| parse_key(n))
                    .collect::<Result<_, _>>()?,
            ),
        })
    }
}

impl Keys {
    fn from_slice(keys: &[VirtualKeyCode]) -> Self {
        match keys {
            [key] => Keys::One(*key),
            _ => Keys::Many(keys.to_vec()),
        }
    }

    fn into_vec(self) -> Vec<VirtualKeyCode> {
        match self {
            Keys::One(key) => vec![key],
            Keys::Many(keys) => keys,
        }
    }
}

/// Short, readable name for a key, used in the key mapping overlay.
pub fn key_label(key: VirtualKeyCode) -> String {
    use VirtualKeyCode as K;
    let s = match key {
        K::Comma => ",",
        K::Period => ".",
        K::Slash => "/",
        K::Backslash => "\\",
        K::Semicolon => ";",
        K::Apostrophe => "'",
        K::LBracket => "[",
        K::RBracket => "]",
        K::Minus => "-",
        K::Equals => "=",
        K::Grave => "`",
        K::Return => "Enter",
        K::Back => "Bksp",
        K::Escape => "Esc",
        K::Key0 => "0",
        K::Key1 => "1",
        K::Key2 => "2",
        K::Key3 => "3",
        K::Key4 => "4",
        K::Key5 => "5",
        K::Key6 => "6",
        K::Key7 => "7",
        K::Key8 => "8",
        K::Key9 => "9",
        _ => return format!("{key:?}").replace("Numpad", "Num"),
    };
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use VirtualKeyCode as K;

    #[test]
    fn default_round_trips() {
        let map = KeyMap::default();
        assert_eq!(KeyMap::from_toml(&map.to_toml()).unwrap(), map);
    }

    #[test]
    fn parses_single_and_multiple_keys() {
        let map = KeyMap::from_toml("[keyboard]\na = \"Z\"\nup = [\"W\", \"Up\"]\nb = []\n").unwrap();
        assert_eq!(map.keys(Button::A), &[K::Z]);
        assert_eq!(map.keys(Button::Up), &[K::W, K::Up]);
        assert!(map.keys(Button::B).is_empty());
        // Unspecified buttons fall back to defaults.
        assert_eq!(map.keys(Button::Start), &[K::Period]);
        assert_eq!(map.buttons_for(K::Up), Button::Up.bit());
    }

    #[test]
    fn multi_key_round_trips() {
        let map = KeyMap::from_toml("[keyboard]\nup = [\"W\", \"Up\"]\n").unwrap();
        assert_eq!(KeyMap::from_toml(&map.to_toml()).unwrap(), map);
    }

    #[test]
    fn rejects_unknown_keys_and_buttons() {
        assert!(KeyMap::from_toml("[keyboard]\na = \"NotAKey\"\n").is_err());
        assert!(KeyMap::from_toml("[keyboard]\nstrat = \"P\"\n").is_err());
    }

    #[test]
    fn error_names_the_bad_key() {
        let err = KeyMap::from_toml("[keyboard]\nb = \"NotAKey\"\n").unwrap_err();
        assert!(err.contains(r#"unknown key "NotAKey""#), "{err}");
        let err = KeyMap::from_toml("[keyboard]\nb = [\"J\", \"Nope\"]\n").unwrap_err();
        assert!(err.contains(r#"unknown key "Nope""#), "{err}");
    }

    #[test]
    fn bind_moves_key_between_buttons() {
        let mut map = KeyMap::default();
        map.bind(Button::Start, K::K); // K was A's key
        assert_eq!(map.keys(Button::Start), &[K::K]);
        assert!(map.keys(Button::A).is_empty());
    }

    #[test]
    fn creates_missing_file() {
        let dir = std::env::temp_dir().join(format!("rnes_keymap_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(FILE_NAME);
        let _ = std::fs::remove_file(&path);
        let (map, status) = KeyMap::load_or_create(&path).unwrap();
        assert_eq!(status, LoadStatus::Created);
        assert_eq!(map, KeyMap::default());
        let (again, status) = KeyMap::load_or_create(&path).unwrap();
        assert_eq!(status, LoadStatus::Loaded);
        assert_eq!(again, map);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
