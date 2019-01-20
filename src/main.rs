extern crate winapi;

mod keyboard_hook;
mod event_loop;
mod windows_util;
mod events;
mod ui_thread;

fn main() {
    use std::thread;
    use std::sync::mpsc::channel;

    let eloop = event_loop::EventLoop::new();
    let exit_event_loop = eloop.create_exiter();
    let (tx, rx) = channel();

    let keyhook = keyboard_hook::KeyboardHook::install(tx);

    thread::spawn(move|| {
        ui_thread::run(rx);
        exit_event_loop();
    });

    println!("Installed key hook.");
    println!("Press CAPS LOCK to exit.");

    eloop.run();
    keyhook.uninstall();

    println!("Farewell.");
}
