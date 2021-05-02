use std::ptr::null_mut;
use winapi::shared::windef::POINT;
use winapi::um::winuser;
use winapi::um::winuser::{
    GetSystemMetrics, INPUT_u, SendInput, INPUT, INPUT_KEYBOARD, KEYEVENTF_UNICODE, MSG,
    SM_CXSCREEN, SM_CYSCREEN,
};

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
