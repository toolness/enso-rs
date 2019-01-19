use std::ptr::null_mut;
use winapi::um::winuser::{GetMessageA};

use super::windows_util;

pub fn run() {
    let mut msg = windows_util::create_blank_msg();

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
}
