use std::ptr::null_mut;
use std::cell::RefCell;
use std::sync::mpsc::Sender;
use std::thread;
use std::sync::mpsc::channel;
use winapi::um::winuser::{GetMessageA, PostThreadMessageA, WM_QUIT};
use winapi::um::processthreadsapi::GetCurrentThreadId;

use winapi::um::winuser::{
    SetWindowsHookExA,
    UnhookWindowsHookEx,
    CallNextHookEx,
    KBDLLHOOKSTRUCT,
    VK_CAPITAL,
    WM_KEYUP,
    WM_KEYDOWN,
    WM_SYSKEYUP,
    WM_SYSKEYDOWN,
    WH_KEYBOARD_LL
};

use winapi::shared::windef::HHOOK;
use winapi::shared::ntdef::NULL;

use super::events::{Event};
use super::windows_util;

struct HookState {
    sender: Sender<Event>,
    in_quasimode: bool
}

thread_local! {
    static HOOK_STATE: RefCell<Option<HookState>> = RefCell::new(None);
}

pub struct KeyboardHook {
    join_handle: Option<thread::JoinHandle<()>>,
    thread_id: u32,
}

impl KeyboardHook {
    fn install_in_thread(init_sender: Sender<u32>, sender: Sender<Event>) {
        let hook_id = unsafe {
            SetWindowsHookExA(WH_KEYBOARD_LL, Some(hook_callback), null_mut(), 0)
        };
        if hook_id == NULL as HHOOK {
            panic!("SetWindowsHookExA() failed!");
        }
        HOOK_STATE.with(|s| {
            *s.borrow_mut() = Some(HookState {
                sender,
                in_quasimode: false
            });
        });
        init_sender.send(unsafe { GetCurrentThreadId() }).unwrap();
        Self::run_event_loop(hook_id);
    }

    fn run_event_loop(hook_id: HHOOK) {
        let mut msg = windows_util::create_blank_msg();

        loop {
            let result = unsafe { GetMessageA(&mut msg, null_mut(), 0, 0) };
            if result == 0 {
                // WM_QUIT was received.
                println!("Keyboard hook thread received WM_QUIT.");
                break;
            } else if result == -1 {
                // An error was received.
                println!("Keyboard hook thread received error.");
                break;
            } else {
                println!("Unexpected message in keyboard hook!");
            }
        }

        if unsafe { UnhookWindowsHookEx(hook_id) } == 0 {
            panic!("UnhookWindowsHookEx failed!");
        }
        HOOK_STATE.with(|s| {
            if s.borrow().is_none() {
                panic!("Assertion failure, expected hook state to exist!");
            }
            *s.borrow_mut() = None;
        });
    }

    pub fn install(sender: Sender<Event>) -> Self {
        let (tx, rx) = channel();
        let builder = thread::Builder::new()
            .name("Keyboard hook".into())
            .stack_size(32 * 1024);
        let join_handle = Some(builder.spawn(move|| {
            Self::install_in_thread(tx, sender);
        }).unwrap());
        let thread_id = rx.recv().unwrap();
        KeyboardHook { join_handle, thread_id }
    }

    pub fn uninstall(self) {
        // Do nothing. The fact that this consumes self will run our drop implementation.
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        println!("Uninstalling keyhook.");

        if unsafe { PostThreadMessageA(self.thread_id, WM_QUIT, 0, 0) } == 0 {
            println!("PostThreadMessageA() failed!");
        }

        let mut handle = None;
        std::mem::swap(&mut handle, &mut self.join_handle);
        handle.expect("join_handle should contain a handle!").join().unwrap();
    }
}

// https://msdn.microsoft.com/en-us/library/ms644985%28v=VS.85%29.aspx
unsafe extern "system" fn hook_callback(n_code: i32, w_param: usize, l_param: isize) -> isize {
    if n_code < 0 {
        CallNextHookEx(null_mut(), n_code, w_param, l_param)
    } else {
        let info = l_param as *const KBDLLHOOKSTRUCT;
        let vk_code = (*info).vkCode as i32;
        let eat_key: bool = HOOK_STATE.with(|s| match *s.borrow_mut() {
            None => {
                println!("Expected hook state to exist!");
                false
            },
            Some(ref mut state) => {
                let is_quasimode_key = vk_code == VK_CAPITAL;
                let wm_type = w_param as u32;

                // Note that WM_SYSKEYUP and WM_SYSKEYDOWN can be set
                // if the alt key is down, even if it's down in combination
                // with other keys.
                let is_key_up = wm_type == WM_KEYUP || wm_type == WM_SYSKEYUP;
                let is_key_down = wm_type == WM_KEYDOWN || wm_type == WM_SYSKEYDOWN;
                let mut force_eat_key = false;

                let possible_event: Option<Event> = if state.in_quasimode {
                    if is_quasimode_key {
                        if is_key_up {
                            state.in_quasimode = false;
                            Some(Event::QuasimodeEnd)
                        } else {
                            // This is likely the quasimode key being auto-repeated.
                            force_eat_key = true;
                            None
                        }
                    } else if is_key_down {
                        Some(Event::Keypress(vk_code))
                    } else {
                        None
                    }
                } else {
                    if is_quasimode_key && is_key_down {
                        state.in_quasimode = true;
                        Some(Event::QuasimodeStart)
                    } else {
                        None
                    }
                };
                match possible_event {
                    None => force_eat_key,
                    Some(event) => {
                        match state.sender.send(event) {
                            Ok(()) => true,
                            Err(e) => {
                                println!("Error sending event: {:?}", e);
                                false
                            }
                        }
                    }
                }
            }
        });
        if eat_key {
            // We processed the keystroke, so don't pass it on to the underlying application.
            -1
        } else {
            CallNextHookEx(null_mut(), n_code, w_param, l_param)
        }
    }
}
