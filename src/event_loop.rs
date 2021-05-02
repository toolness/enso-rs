use std::ptr::null_mut;
use winapi::um::processthreadsapi::GetCurrentThreadId;
use winapi::um::winuser::{DispatchMessageA, GetMessageA, PostThreadMessageA, WM_TIMER, WM_USER};

use super::error::Error;
use super::windows_util;

const WM_USER_KICK_EVENT_LOOP: u32 = WM_USER + 1;

pub struct EventLoop {
    thread_id: u32,
}

pub fn kick_event_loop(thread_id: u32) {
    if unsafe { PostThreadMessageA(thread_id, WM_USER_KICK_EVENT_LOOP, 0, 0) } == 0 {
        println!("PostThreadMessageA() failed to kick event loop!");
    }
}

impl EventLoop {
    pub fn new() -> Self {
        let thread_id = unsafe { GetCurrentThreadId() };
        EventLoop { thread_id }
    }

    pub fn get_thread_id(&self) -> u32 {
        self.thread_id
    }

    pub fn run<F>(&self, mut loop_cb: F) -> Result<(), Error>
    where
        F: FnMut() -> Result<bool, Error>,
    {
        let mut msg = windows_util::create_blank_msg();

        loop {
            let result = unsafe { GetMessageA(&mut msg, null_mut(), 0, 0) };
            if result == 0 {
                // WM_QUIT was received.
                println!("Received WM_QUIT.");
                break;
            } else if result == -1 {
                // An error was received.
                eprintln!("GetMessageA() returned -1.");
                return Err(Error::from_winapi());
            } else {
                if msg.hwnd == null_mut() {
                    match msg.message {
                        WM_USER_KICK_EVENT_LOOP => {
                            // Do nothing, this was just sent to kick us out of GetMessage so
                            // our loop callback can process any incoming events sent through
                            // other safe Rust-based synchronization mechanisms.
                        }
                        WM_TIMER => {
                            // Do nothing. It seems like DirectX or the GDI
                            // or something sends these as a result of our
                            // layered window code, and I'm not sure why.
                        }
                        _ => {
                            println!("Unknown thread message: 0x{:x}", msg.message);
                        }
                    }
                }
                unsafe {
                    DispatchMessageA(&msg);
                }
            }
            if loop_cb()? {
                break;
            }
        }

        Ok(())
    }
}
