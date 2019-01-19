use winapi::um::winuser::WM_USER;

use winapi::um::processthreadsapi::GetCurrentThreadId;
use winapi::um::winuser::{
    PostThreadMessageA,
};
use winapi::um::winuser::{
    WM_KEYDOWN,
    WM_KEYUP,
    MSG
};
use winapi::shared::minwindef::{
    UINT,
    WPARAM,
    LPARAM
};

const WM_USER_KEYPRESS: UINT = WM_USER + 1;

pub enum KeypressType {
    KeyDown,
    KeyUp
}

impl KeypressType {
    pub fn from_w_param(w_param: WPARAM) -> Option<Self> {
        match w_param as u32 {
            WM_KEYDOWN => Some(KeypressType::KeyDown),
            WM_KEYUP => Some(KeypressType::KeyUp),
            _ => None,
        }
    }
}

pub enum Event {
    Keypress(KeypressType, i32),
}

impl Event {
    pub fn queue(&self) {
        match self {
            Event::Keypress(keypress_type, vkey) => {
                let w_param = match keypress_type {
                    KeypressType::KeyDown => WM_KEYDOWN,
                    KeypressType::KeyUp => WM_KEYUP
                };
                post_thread_message(WM_USER_KEYPRESS, w_param as WPARAM, *vkey as LPARAM);
            }
        }
    }

    pub fn from_message(msg: &MSG) -> Option<Self> {
        match msg.message {
            WM_USER_KEYPRESS => {
                match KeypressType::from_w_param(msg.wParam) {
                    Some(keypress_type) => Some(Event::Keypress(keypress_type, msg.lParam as i32)),
                    None => {
                        println!("Warning: WM_USER_KEYPRESS with invalid wParam: {}", msg.wParam);
                        None
                    }
                }
            },
            _ => None
        }
    }
}

fn post_thread_message(msg: UINT, w_param: WPARAM, l_param: LPARAM) {
    if unsafe { PostThreadMessageA(GetCurrentThreadId(), msg, w_param, l_param) == 0 } {
        println!("Warning: PostThreadMessageA() failed!");
    }
}
