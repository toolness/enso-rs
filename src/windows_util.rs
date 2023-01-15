use std::ffi::CStr;
use std::ptr::null_mut;
use winapi::shared::windef::POINT;
use winapi::um::winuser;
use winapi::um::winuser::{
    GetKeyState, GetSystemMetrics, INPUT_u, SendInput, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, MSG, SM_CXSCREEN, SM_CYSCREEN, VK_CAPITAL,
};

use crate::system::{KeyDirection, ModifierKey};

use super::error::Error;

const VK_0: i32 = 0x30;
const VK_9: i32 = 0x39;
const VK_A: i32 = 0x41;
const VK_Z: i32 = 0x5a;

pub fn create_blank_msg() -> MSG {
    MSG {
        hwnd: null_mut(),
        message: 0,
        wParam: 0,
        lParam: 0,
        time: 0,
        pt: POINT { x: 0, y: 0 },
    }
}

/// Converts the given nul-terminated statically allocated string
/// to a pointer capable of being used as a LPCSTR in win32 API calls.
pub fn to_lpcstr(name: &'static [u8]) -> *const i8 {
    CStr::from_bytes_with_nul(name).unwrap().as_ptr()
}

pub fn send_modifier_keypress(key: ModifierKey, direction: KeyDirection) -> Result<(), Error> {
    let vk = match key {
        ModifierKey::Alt => winuser::VK_MENU,
        ModifierKey::Control => winuser::VK_CONTROL,
        ModifierKey::Shift => winuser::VK_SHIFT,
    };
    send_keypress(vk, direction)
}

pub fn send_raw_keypress_for_char(ch: char, direction: KeyDirection) -> Result<bool, Error> {
    let vkey = char_to_vkey(ch);
    if let Some(vk) = vkey {
        println!(
            "Raw keypress: vk: {:x}, direction: {:?}, ch={}",
            vk, direction, ch
        );
        send_keypress(vk, direction)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn send_keypress(vk: i32, direction: KeyDirection) -> Result<(), Error> {
    println!("send_keypress: vk: {:x}, direction: {:?}", vk, direction);
    unsafe {
        let mut u: INPUT_u = Default::default();
        let mut ki = u.ki_mut();
        ki.wVk = vk as u16;
        ki.dwFlags = match direction {
            KeyDirection::Up => KEYEVENTF_KEYUP,
            KeyDirection::Down => 0,
        };
        let mut inp = INPUT {
            type_: INPUT_KEYBOARD,
            u,
        };
        let result = SendInput(1, &mut inp, std::mem::size_of_val(&inp) as i32);
        if result != 1 {
            return Err(Error::WindowsAPIGeneric);
        }
    }
    Ok(())
}

pub fn disable_caps_lock() -> Result<(), Error> {
    let curr_state = unsafe { GetKeyState(VK_CAPITAL) };

    const LOW_ORDER_BIT: i16 = 0x01;
    const HIGH_ORDER_BIT: i16 = 0x80;

    // From the MSDN documentation for `GetKeyState`'s return value:
    //
    //   * If the high-order bit is 1, the key is down; otherwise, it is up.
    //   * If the low-order bit is 1, the key is toggled. A key, such as the
    //     CAPS LOCK key, is toggled if it is turned on.

    let is_down = (curr_state & HIGH_ORDER_BIT) != 0;
    let is_toggled = (curr_state & LOW_ORDER_BIT) != 0;

    if is_toggled {
        if is_down {
            // Looks like the caps lock key is physically pressed
            // down. Let's temporarily release it.
            send_keypress(VK_CAPITAL, KeyDirection::Up)?;
        }

        // Press the caps lock key down.
        send_keypress(VK_CAPITAL, KeyDirection::Down)?;

        // Now release the caps lock key, unless it was already being
        // pressed down.
        if !is_down {
            send_keypress(VK_CAPITAL, KeyDirection::Up)?;
        }

        Ok(())
    } else {
        Ok(())
    }
}

pub fn send_unicode_keypress(value: &str) -> Result<(), Error> {
    // https://stackoverflow.com/a/22308727/2422398
    for ch in value.encode_utf16() {
        unsafe {
            let mut u: INPUT_u = Default::default();
            let mut ki = u.ki_mut();
            ki.wScan = ch;
            ki.dwFlags = KEYEVENTF_UNICODE;
            let mut inp = INPUT {
                type_: INPUT_KEYBOARD,
                u,
            };
            let result = SendInput(1, &mut inp, std::mem::size_of_val(&inp) as i32);
            if result != 1 {
                return Err(Error::WindowsAPIGeneric);
            }
        }
    }
    Ok(())
}

fn get_system_metrics(n_index: i32) -> Result<i32, Error> {
    let result = unsafe { GetSystemMetrics(n_index) };
    if result == 0 {
        Err(Error::WindowsAPIGeneric)
    } else {
        Ok(result)
    }
}

pub fn get_primary_screen_size() -> Result<(u32, u32), Error> {
    let width = get_system_metrics(SM_CXSCREEN)? as u32;
    let height = get_system_metrics(SM_CYSCREEN)? as u32;
    Ok((width, height))
}

#[test]
fn test_get_primary_screen_size() {
    assert!(get_primary_screen_size().is_ok());
}

#[test]
fn test_disable_caps_lock() {
    assert!(disable_caps_lock().is_ok());
}

pub fn char_to_vkey(mut char: char) -> Option<i32> {
    if char.is_ascii_lowercase() {
        char = char.to_ascii_uppercase();
    }
    match char {
        '0'..='9' => Some(char as i32),
        'A'..='Z' => Some(char as i32),
        ' ' => Some(winuser::VK_SPACE),
        '\x08' => Some(winuser::VK_BACK),
        '\x0d' => Some(winuser::VK_RETURN),
        '\x1b' => Some(winuser::VK_ESCAPE),
        _ => {
            if char.is_ascii_graphic() {
                Some(char as i32)
            } else {
                None
            }
        }
    }
}

pub fn vkey_to_char(vk_code: i32) -> Option<char> {
    match vk_code {
        VK_0..=VK_9 | VK_A..=VK_Z => Some(char::from(vk_code as u8)),
        winuser::VK_SPACE => Some(' '),
        _ => None,
    }
}

#[test]
fn test_vkey_to_char() {
    assert_eq!(vkey_to_char(VK_0), Some('0'));
    assert_eq!(vkey_to_char(VK_0 + 3), Some('3'));
    assert_eq!(vkey_to_char(VK_9), Some('9'));
    assert_eq!(vkey_to_char(VK_A), Some('A'));
    assert_eq!(vkey_to_char(VK_A + 3), Some('D'));
    assert_eq!(vkey_to_char(VK_Z), Some('Z'));
    assert_eq!(vkey_to_char(winuser::VK_F1), None);
}

#[test]
fn test_char_to_vkey() {
    assert_eq!(char_to_vkey('0'), Some(VK_0));
    assert_eq!(char_to_vkey('3'), Some(VK_0 + 3));
    assert_eq!(char_to_vkey('9'), Some(VK_9));
    assert_eq!(char_to_vkey('A'), Some(VK_A));
    assert_eq!(char_to_vkey('D'), Some(VK_A + 3));
    assert_eq!(char_to_vkey('c'), Some(VK_A + 2));
    assert_eq!(char_to_vkey('Z'), Some(VK_Z));
    assert_eq!(char_to_vkey(' '), Some(winuser::VK_SPACE));
}
