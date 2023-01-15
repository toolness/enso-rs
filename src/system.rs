use std::path::PathBuf;

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

#[derive(Debug)]
pub enum ModifierKey {
    Shift,
    Alt,
    Control,
}

pub fn press_modifier_key(key: ModifierKey, direction: KeyDirection) -> Result<(), Error> {
    crate::windows_util::send_modifier_keypress(key, direction)
}

/// Press the key that corresponds to the given ASCII character. Use this
/// if you are simulating a hotkey combination, etc.
///
/// Returns true if the key was pressed, or false if the key was not an ASCII key
/// that could be pressed.
///
/// TODO: Consider returning an error in the case of a non-ASCII key.
pub fn press_key(ch: char, direction: KeyDirection) -> Result<bool, Error> {
    crate::windows_util::send_raw_keypress_for_char(ch, direction)
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
    }

    Ok(home_dir)
}
