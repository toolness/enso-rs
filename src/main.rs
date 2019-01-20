extern crate winapi;

mod keyboard_hook;
mod event_loop;
mod windows_util;
mod events;

fn main() {
    use std::thread;
    use std::sync::mpsc::channel;

    use self::events::Event;

    let eloop = event_loop::EventLoop::new();
    let exit_event_loop = eloop.create_exiter();
    let (tx, rx) = channel();

    let keyhook = keyboard_hook::KeyboardHook::install(tx);

    thread::spawn(move|| {
        loop {
            match rx.recv() {
                Ok(event) => {
                    println!("{:?}", event);
                    match event {
                        Event::QuasimodeEnd => {
                            exit_event_loop();
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

    eloop.run();
    keyhook.uninstall();

    println!("Farewell.");
}
