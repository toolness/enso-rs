use std::ptr::null_mut;
use winapi::um::winuser;
use winapi::um::winuser::{MSG};
use winapi::shared::windef::POINT;

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
