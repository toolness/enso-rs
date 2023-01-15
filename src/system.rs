use std::{convert::TryFrom, path::PathBuf};

/// This module is intened to provide an OS-independent way to access system functionality
/// that platform-independent commands can use.
///
/// Right now it only supports Windows, but that's because Enso only really supports
/// Windows currently.
use crate::error::Error;

#[derive(Debug)]
pub enum KeyDirection {
    Up,
    Down,
}

#[derive(Debug, Copy, Clone)]
pub enum VirtualKey {
    Shift,
    Alt,
    Control,
    Escape,
    Space,
    Enter,
    Graphic(GraphicKey),
}

impl TryFrom<&str> for VirtualKey {
    type Error = Error;

    fn try_from(key: &str) -> Result<Self, Self::Error> {
        if key.len() == 1 {
            let key = key.chars().next().unwrap();
            if let Some(key) = GraphicKey::new(key) {
                Ok(VirtualKey::Graphic(key))
            } else {
                Err(Error::new(format!("Unsupported virtual key: {}", key)))
            }
        } else {
            match key.to_ascii_lowercase().as_str() {
                "shift" => Ok(VirtualKey::Shift),
                "alt" => Ok(VirtualKey::Alt),
                "control" | "ctrl" => Ok(VirtualKey::Control),
                "escape" => Ok(VirtualKey::Escape),
                "space" => Ok(VirtualKey::Space),
                "enter" => Ok(VirtualKey::Enter),
                _ => Err(Error::new(format!("Unsupported virtual key: {}", key))),
            }
        }
    }
}

impl From<VirtualKey> for u8 {
    fn from(vk: VirtualKey) -> Self {
        match vk {
            VirtualKey::Shift => 0x10,
            VirtualKey::Alt => 0x12,
            VirtualKey::Control => 0x11,
            VirtualKey::Escape => 0x1B,
            VirtualKey::Space => 0x20,
            VirtualKey::Enter => 0x0D,
            VirtualKey::Graphic(g) => g.into(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
/// Encapsulates a virtual key that represents an ASCII graphic code. See
/// `std::char::is_ascii_graphic` for more information:
///
///    https://doc.rust-lang.org/std/primitive.char.html#method.is_ascii_graphic
pub struct GraphicKey {
    ch: char,
}

impl GraphicKey {
    pub fn new(ch: char) -> Option<Self> {
        if ch.is_ascii_lowercase() {
            Some(GraphicKey {
                ch: ch.to_ascii_uppercase(),
            })
        } else if ch.is_ascii_graphic() {
            Some(GraphicKey { ch })
        } else {
            None
        }
    }

    pub fn char(&self) -> char {
        self.ch
    }
}

impl From<GraphicKey> for char {
    fn from(gk: GraphicKey) -> Self {
        gk.ch
    }
}

impl From<GraphicKey> for u8 {
    fn from(gk: GraphicKey) -> Self {
        gk.ch as u8
    }
}

/// Press the given key. Use this if you are simulating a hotkey combination, etc.
/// This will take into account the current modifier keys, so e.g. pressing 'c' will
/// only end up uppercase if the shift key is down.
pub fn press_key(ch: VirtualKey, direction: KeyDirection) -> Result<(), Error> {
    crate::windows_util::send_virtual_keypress(ch, direction)
}

/// Insert the given unicode character into the current application. This doesn't
/// take into account the current modifier keys or anything.
pub fn type_char(ch: &str) -> Result<(), Error> {
    crate::windows_util::send_unicode_keypress(ch)
}

/// Returns Enso's home directory for the current user, usually found at
/// `~/.enso`.  Creates the directory if it doesn't exist.
pub fn get_enso_home_dir() -> Result<PathBuf, Error> {
    let mut home_dir =
        dirs::home_dir().ok_or_else(|| Error::new("Could not find home directory"))?;

    home_dir.push(".enso");

    if !home_dir.exists() {
        println!("Creating Enso home directory at {:?}.", home_dir);
        std::fs::create_dir_all(home_dir.clone())?;
    }

    Ok(home_dir)
}
