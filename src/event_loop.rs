use std::ptr::null_mut;
use std::sync::Once;
use winapi::shared::{minwindef, windef};
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::processthreadsapi::GetCurrentThreadId;
use winapi::um::winuser;
use winapi::um::winuser::{
    DispatchMessageA, GetMessageA, PostThreadMessageA, WM_CLOSE, WM_TIMER, WM_USER,
};

use super::error::Error;
use super::windows_util::{create_blank_msg, to_lpcstr};

const WM_USER_KICK_EVENT_LOOP: u32 = WM_USER + 1;

static mut WINDOW_CLASS: Result<minwindef::ATOM, minwindef::DWORD> = Ok(0);
static INIT_WINDOW_CLASS: Once = Once::new();
static WINDOW_CLASS_NAME: &'static [u8] = b"EnsoEventLoopWindow\0";

pub struct EventLoop {
    thread_id: u32,
}

pub fn kick_event_loop(thread_id: u32) {
    if unsafe { PostThreadMessageA(thread_id, WM_USER_KICK_EVENT_LOOP, 0, 0) } == 0 {
        println!("PostThreadMessageA() failed to kick event loop!");
    }
}

impl EventLoop {
    fn end_other_event_loop_processes() -> Result<(), Error> {
        unsafe {
            let hwnd = winuser::FindWindowA(to_lpcstr(WINDOW_CLASS_NAME), null_mut());

            if hwnd != null_mut() {
                println!(
                    "Existing event loop found with HWND {:?}. Closing it.",
                    hwnd
                );
                let result = winuser::PostMessageA(hwnd, WM_CLOSE, 0, 0);
                if result == 0 {
                    println!(
                        "An error occurred when ending the existing event loop ({}).",
                        Error::get_last_windows_api_error()
                    );
                }
            }
        }

        Ok(())
    }

    fn create_window_class() -> Result<minwindef::ATOM, Error> {
        INIT_WINDOW_CLASS.call_once(|| {
            let info = winuser::WNDCLASSEXA {
                cbSize: std::mem::size_of::<winuser::WNDCLASSEXA>() as u32,
                style: 0,
                lpfnWndProc: Some(winuser::DefWindowProcA),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: unsafe { GetModuleHandleA(null_mut()) },
                hIcon: null_mut(),
                hCursor: null_mut(),
                hbrBackground: null_mut(),
                lpszMenuName: null_mut(),
                lpszClassName: to_lpcstr(WINDOW_CLASS_NAME),
                hIconSm: null_mut(),
            };

            let window_class = unsafe { winuser::RegisterClassExA(&info) };

            unsafe {
                WINDOW_CLASS = if window_class == 0 {
                    Err(Error::get_last_windows_api_error())
                } else {
                    Ok(window_class)
                };
            }
        });
        let result = unsafe { WINDOW_CLASS };
        match result {
            Ok(atom) => Ok(atom),
            Err(code) => Err(Error::WindowsAPI(code)),
        }
    }

    fn create_window() -> Result<windef::HWND, Error> {
        Self::create_window_class()?;
        let window = unsafe {
            winuser::CreateWindowExA(
                0,                            /* dwExStyle    */
                to_lpcstr(WINDOW_CLASS_NAME), /* lpClassName  */
                null_mut(),                   /* lpWindowName */
                0,                            /* dwStyle      */
                0,                            /* x            */
                0,                            /* y            */
                0,                            /* nWidth       */
                0,                            /* nHeight      */
                null_mut(),                   /* hWndParent   */
                null_mut(),                   /* hMenu        */
                GetModuleHandleA(null_mut()), /* hInstance    */
                null_mut(),                   /* lpParam      */
            )
        };

        if window == null_mut() {
            return Err(Error::from_winapi());
        }

        Ok(window)
    }

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
        Self::end_other_event_loop_processes()?;
        let hwnd = Self::create_window()?;
        let mut msg = create_blank_msg();

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
                } else if msg.hwnd == hwnd && msg.message == WM_CLOSE {
                    println!("Event loop window received WM_CLOSE, exiting.");
                    break;
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
