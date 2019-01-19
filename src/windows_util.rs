use std::ptr::null_mut;
use winapi::um::winuser::{MSG};
use winapi::shared::windef::POINT;

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
