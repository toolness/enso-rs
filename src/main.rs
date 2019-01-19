extern crate winapi;

use winapi::um::winuser::{
    SetWindowsHookExA,
    UnhookWindowsHookEx,
    CallNextHookEx,
    GetMessageA,
    MSG,
    WH_KEYBOARD_LL,
};

use winapi::shared::windef::POINT;
use winapi::um::processthreadsapi::GetCurrentProcessId;

use std::ptr::null_mut;

// https://msdn.microsoft.com/en-us/library/ms644985%28v=VS.85%29.aspx
unsafe extern "system" fn hook_callback(n_code: i32, w_param: usize, l_param: isize) -> isize {
    if n_code < 0 {
        CallNextHookEx(null_mut(), n_code, w_param, l_param)
    } else {
        println!("In hook_callback() with n_code={}, w_param={}, processId={}",
                 n_code, w_param, GetCurrentProcessId());
        // TODO: Process the keystroke.
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
    println!("Press CTRL-C to exit.");

    let mut msg = MSG {
        hwnd: null_mut(),
        message: 0,
        wParam: 0,
        lParam: 0,
        time: 0,
        pt: POINT { x: 0, y: 0 }
    };

    loop {
        unsafe {
            let result = GetMessageA(&mut msg, null_mut(), 0, 0);
            if result == 0 {
                // WM_QUIT was received.
                println!("Received WM_QUIT.");
            } else if result == -1 {
                println!("Received error.");
                // An error was received.
            } else {
                println!("Got a message {}", msg.message);
            }
        }
        break;
    }

    let unhook_result;
    unsafe {
        unhook_result = UnhookWindowsHookEx(hook_id);
    }
    println!("unhook_result is {}", unhook_result);
}
