use std::ptr::null_mut;

use winapi::um::winuser::{
    SetWindowsHookExA,
    UnhookWindowsHookEx,
    CallNextHookEx,
    KBDLLHOOKSTRUCT,
    VK_CAPITAL,
    WM_KEYUP,
    WH_KEYBOARD_LL
};

use winapi::shared::windef::HHOOK;
use winapi::shared::ntdef::NULL;

use super::events::{Event, KeypressType};

pub struct KeyboardHook {
    hook_id: HHOOK,
}

impl KeyboardHook {
    pub fn install() -> Self {
        let hook_id = unsafe {
            SetWindowsHookExA(WH_KEYBOARD_LL, Some(hook_callback), null_mut(), 0)
        };
        if hook_id == NULL as HHOOK {
            panic!("SetWindowsHookExA() failed!");
        }
        KeyboardHook { hook_id }
    }

    pub fn uninstall(self) {
        // Do nothing. The fact that this consumes self will run our drop implementation.
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        println!("Uninstalling keyhook.");
        if unsafe { UnhookWindowsHookEx(self.hook_id) } == 0 {
            panic!("UnhookWindowsHookEx failed!");
        }
    }
}

// https://msdn.microsoft.com/en-us/library/ms644985%28v=VS.85%29.aspx
unsafe extern "system" fn hook_callback(n_code: i32, w_param: usize, l_param: isize) -> isize {
    if n_code < 0 {
        CallNextHookEx(null_mut(), n_code, w_param, l_param)
    } else {
        let info = l_param as *const KBDLLHOOKSTRUCT;
        let vk_code = (*info).vkCode as i32;
        println!("In hook_callback() with n_code={}, w_param={}, vkCode={}",
                 n_code, w_param, vk_code);
        if vk_code == VK_CAPITAL {
            if w_param == WM_KEYUP as usize {
                Event::Keypress(KeypressType::from_w_param(w_param).unwrap(), vk_code).queue();
            }
            // We processed the keystroke, so don't pass it on to the underlying application.
            return -1;
        }
        CallNextHookEx(null_mut(), n_code, w_param, l_param)
    }
}
