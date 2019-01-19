use std::ptr::null_mut;
use winapi::um::winuser::{GetMessageA, VK_CAPITAL};

use super::windows_util;
use super::events::{Event, KeypressType};

pub fn run() {
    let mut msg = windows_util::create_blank_msg();

    loop {
        let result = unsafe { GetMessageA(&mut msg, null_mut(), 0, 0) };
        if result == 0 {
            // WM_QUIT was received.
            println!("Received WM_QUIT.");
            break;
        } else if result == -1 {
            // An error was received.
            println!("Received error.");
        } else {
            match Event::from_message(&msg) {
                Some(Event::Keypress(keypress_type, vkey)) => {
                    match keypress_type {
                        KeypressType::KeyUp => {
                            if vkey == VK_CAPITAL {
                                // TODO: Eventually we shouldn't do this.
                                break;
                            }
                        },
                        KeypressType::KeyDown => {}
                    }
                },
                None => println!("Got a message {}", msg.message)
            }
        }
    }
}
