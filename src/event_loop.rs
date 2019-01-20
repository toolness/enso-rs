use std::ptr::null_mut;
use winapi::um::winuser::{GetMessageA, DispatchMessageA, PostThreadMessageA, WM_QUIT};
use winapi::um::processthreadsapi::GetCurrentThreadId;

use super::windows_util;

pub struct EventLoop {
    thread_id: u32,
}

impl EventLoop {
    pub fn new() -> Self {
        let thread_id = unsafe { GetCurrentThreadId() };
        EventLoop { thread_id }
    }

    pub fn create_exiter(&self) -> impl FnOnce() -> () {
        let thread_id = self.thread_id;
        move|| {
            if unsafe { PostThreadMessageA(thread_id, WM_QUIT, 0, 0) } == 0 {
                println!("PostThreadMessageA() failed!");
            }
        }
    }

    pub fn run(&self) {
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
                unsafe { DispatchMessageA(&msg); }
            }
        }
    }
}
