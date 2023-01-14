use std::cell::RefCell;
use std::ptr::null_mut;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::thread;
use winapi::um::processthreadsapi::GetCurrentThreadId;
use winapi::um::winuser::{GetMessageA, PostThreadMessageA, WM_QUIT};

use winapi::um::winuser::{
    CallNextHookEx, SetWindowsHookExA, UnhookWindowsHookEx, KBDLLHOOKSTRUCT, VK_CAPITAL,
    WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use winapi::shared::ntdef::NULL;
use winapi::shared::windef::HHOOK;

use super::event_loop::kick_event_loop;
use super::events::HookEvent;
use super::windows_util;

struct HookState {
    sender: Sender<HookEvent>,
    receiver_thread_id: u32,
    in_quasimode: bool,
}

impl HookState {
    fn process_key(&mut self, wm_type: u32, vk_code: i32) -> bool {
        let is_quasimode_key = vk_code == VK_CAPITAL;

        // Note that WM_SYSKEYUP and WM_SYSKEYDOWN can be set
        // if the alt key is down, even if it's down in combination
        // with other keys.
        let is_key_up = wm_type == WM_KEYUP || wm_type == WM_SYSKEYUP;
        let is_key_down = wm_type == WM_KEYDOWN || wm_type == WM_SYSKEYDOWN;
        let mut force_eat_key = false;

        let possible_event: Option<HookEvent> = if self.in_quasimode {
            if is_quasimode_key {
                if is_key_up {
                    self.in_quasimode = false;
                    Some(HookEvent::QuasimodeEnd)
                } else {
                    // This is likely the quasimode key being auto-repeated.
                    force_eat_key = true;
                    None
                }
            } else if is_key_down {
                Some(HookEvent::Keypress(vk_code))
            } else {
                None
            }
        } else {
            if is_quasimode_key && is_key_down {
                self.in_quasimode = true;
                Some(HookEvent::QuasimodeStart)
            } else {
                None
            }
        };
        match possible_event {
            None => force_eat_key,
            Some(event) => match self.sender.send(event) {
                Ok(()) => {
                    kick_event_loop(self.receiver_thread_id);
                    true
                }
                Err(e) => {
                    println!("Error sending event: {:?}", e);
                    false
                }
            },
        }
    }
}

thread_local! {
    static HOOK_STATE: RefCell<Option<HookState>> = RefCell::new(None);
}

pub struct KeyboardHook {
    join_handle: Option<thread::JoinHandle<()>>,
    thread_id: u32,
}

impl KeyboardHook {
    fn install_in_thread(
        init_sender: Sender<u32>,
        sender: Sender<HookEvent>,
        receiver_thread_id: u32,
    ) {
        let hook_id =
            unsafe { SetWindowsHookExA(WH_KEYBOARD_LL, Some(hook_callback), null_mut(), 0) };
        if hook_id == NULL as HHOOK {
            panic!("SetWindowsHookExA() failed!");
        }
        HOOK_STATE.with(|s| {
            *s.borrow_mut() = Some(HookState {
                sender,
                receiver_thread_id,
                in_quasimode: false,
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

    pub fn install(sender: Sender<HookEvent>, receiver_thread_id: u32) -> Self {
        let (tx, rx) = channel();
        let builder = thread::Builder::new()
            .name("Keyboard hook".into())
            .stack_size(32 * 1024);
        let join_handle = Some(
            builder
                .spawn(move || {
                    Self::install_in_thread(tx, sender, receiver_thread_id);
                })
                .unwrap(),
        );
        let thread_id = rx.recv().unwrap();
        KeyboardHook {
            join_handle,
            thread_id,
        }
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
        handle
            .expect("join_handle should contain a handle!")
            .join()
            .unwrap();
    }
}

// https://msdn.microsoft.com/en-us/library/ms644985%28v=VS.85%29.aspx
unsafe extern "system" fn hook_callback(n_code: i32, w_param: usize, l_param: isize) -> isize {
    if n_code < 0 {
        CallNextHookEx(null_mut(), n_code, w_param, l_param)
    } else {
        let info = l_param as *const KBDLLHOOKSTRUCT;
        let vk_code = (*info).vkCode as i32;
        let wm_type = w_param as u32;
        let eat_key: bool = HOOK_STATE.with(|s| match *s.borrow_mut() {
            None => {
                println!("Expected hook state to exist!");
                false
            }
            Some(ref mut state) => state.process_key(wm_type, vk_code),
        });
        if eat_key {
            // We processed the keystroke, so don't pass it on to the underlying application.
            -1
        } else {
            CallNextHookEx(null_mut(), n_code, w_param, l_param)
        }
    }
}
