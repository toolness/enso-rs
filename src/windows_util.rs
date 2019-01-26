use std::ptr::null_mut;
use winapi::um::winuser;
use winapi::um::winuser::{MSG, SM_CXSCREEN, SM_CYSCREEN, GetSystemMetrics};
use winapi::shared::windef::POINT;

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
        pt: POINT { x: 0, y: 0 }
    }
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
        VK_0...VK_9 | VK_A...VK_Z => Some(char::from(vk_code as u8)),
        winuser::VK_SPACE => Some(' '),
        _ => None
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
