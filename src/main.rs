extern crate winapi;

mod keyboard_hook;
mod event_loop;
mod windows_util;
mod events;

fn main() {
    use std::thread;
    use std::sync::mpsc::channel;
    use winapi::um::winuser::{PostThreadMessageA, WM_QUIT};
    use winapi::um::processthreadsapi::GetCurrentThreadId;

    use self::events::Event;

    let main_thread_id = unsafe { GetCurrentThreadId() };
    let (tx, rx) = channel();

    let keyhook = keyboard_hook::KeyboardHook::install(tx);

    thread::spawn(move|| {
        loop {
            match rx.recv() {
                Ok(event) => {
                    println!("{:?}", event);
                    match event {
                        Event::QuasimodeEnd => {
                            if unsafe { PostThreadMessageA(main_thread_id, WM_QUIT, 0, 0) } == 0 {
                                println!("PostThreadMessageA() failed!");
                            }
                            break;
                        }
                        _ => {}
                    };
                },
                Err(e) => {
                    println!("Receive error {:?}", e);
                    break;
                }
            }
        }
    });

    println!("Installed key hook.");
    println!("Press CAPS LOCK to exit.");

    event_loop::run();

    keyhook.uninstall();

    println!("Farewell.");
}
