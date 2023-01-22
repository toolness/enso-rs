use std::ffi::CStr;
use std::ptr::null_mut;
use winapi::shared::windef::POINT;
use winapi::um::winuser;
use winapi::um::winuser::{
    GetKeyState, GetSystemMetrics, INPUT_u, SendInput, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, MSG, SM_CXSCREEN, SM_CYSCREEN, VK_CAPITAL,
};

use crate::system::{KeyDirection, VirtualKey};

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

pub fn send_virtual_keypress(key: VirtualKey, direction: KeyDirection) -> Result<(), Error> {
    let vk: i32 = match key {
        VirtualKey::Shift => 0x10,
        VirtualKey::Alt => 0x12,
        VirtualKey::Control => 0x11,
        VirtualKey::Escape => 0x1B,
        VirtualKey::Space => 0x20,
        VirtualKey::Enter => 0x0D,
        VirtualKey::F1 => 0x70,
        VirtualKey::F2 => 0x71,
        VirtualKey::F3 => 0x72,
        VirtualKey::F4 => 0x73,
        VirtualKey::F5 => 0x74,
        VirtualKey::F6 => 0x75,
        VirtualKey::F7 => 0x76,
        VirtualKey::F8 => 0x77,
        VirtualKey::F9 => 0x78,
        VirtualKey::F10 => 0x79,
        VirtualKey::F11 => 0x7A,
        VirtualKey::F12 => 0x7B,
        VirtualKey::Alphanumeric(a) => u8::from(a) as i32,
    };
    send_keypress(vk, direction)
}

fn send_keypress(vk: i32, direction: KeyDirection) -> Result<(), Error> {
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

pub fn vkey_to_char(vk_code: i32) -> Option<char> {
    // TODO: These virtual key codes are actually just ASCII codes, we could
    // probably accomplish this better with e.g. `std::char::is_ascii_control`.
    // or something.
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
