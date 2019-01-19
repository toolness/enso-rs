extern crate winapi;

use winapi::um::winuser::{
    SetWindowsHookExA,
    UnhookWindowsHookEx,
    CallNextHookEx,
    GetMessageA,
    PostThreadMessageA,
    KBDLLHOOKSTRUCT,
    MSG,
    VK_CAPITAL,
    WM_KEYUP,
    WM_QUIT,
    WH_KEYBOARD_LL
};

use winapi::shared::windef::POINT;

use winapi::um::processthreadsapi::{
    GetCurrentProcessId,
    GetCurrentThreadId
};

use std::ptr::null_mut;

// https://msdn.microsoft.com/en-us/library/ms644985%28v=VS.85%29.aspx
unsafe extern "system" fn hook_callback(n_code: i32, w_param: usize, l_param: isize) -> isize {
    if n_code < 0 {
        CallNextHookEx(null_mut(), n_code, w_param, l_param)
    } else {
        let info = l_param as *const KBDLLHOOKSTRUCT;
        let vk_code = (*info).vkCode as i32;
        println!("In hook_callback() with n_code={}, w_param={}, processId={}, vkCode={}",
                 n_code, w_param, GetCurrentProcessId(), vk_code);
        if vk_code == VK_CAPITAL {
            if w_param == WM_KEYUP as usize {
                PostThreadMessageA(GetCurrentThreadId(), WM_QUIT, 0, 0);
            }
            // We processed the keystroke, so don't pass it on to the underlying application.
            return -1;
        }
        CallNextHookEx(null_mut(), n_code, w_param, l_param)
    }
}

fn main() {
    let hook_id;
    unsafe {
        println!("Hello, I have process ID {}.", GetCurrentProcessId());
        hook_id = SetWindowsHookExA(WH_KEYBOARD_LL, Some(hook_callback), null_mut(), 0);
    }

    println!("Installed key hook with ID {:?}.", hook_id);
    println!("Press CAPS LOCK to exit.");

    let mut msg = MSG {
        hwnd: null_mut(),
        message: 0,
        wParam: 0,
        lParam: 0,
        time: 0,
        pt: POINT { x: 0, y: 0 }
    };

    loop {
        let result = unsafe { GetMessageA(&mut msg, null_mut(), 0, 0) };
        if result == 0 {
            // WM_QUIT was received.
            println!("Received WM_QUIT.");
            break;
        } else if result == -1 {
            println!("Received error.");
            // An error was received.
        } else {
            println!("Got a message {}", msg.message);
        }
    }

    let unhook_result;
    unsafe {
        unhook_result = UnhookWindowsHookEx(hook_id);
    }
    println!("unhook_result is {}", unhook_result);
    println!("Farewell.");
}
