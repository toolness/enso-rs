extern crate winapi;

mod keyboard_hook;
mod event_loop;
mod windows_util;
mod events;
mod ui_thread;
mod transparent_window;

fn main() {
    use std::thread;
    use std::sync::mpsc::channel;

    let eloop = event_loop::EventLoop::new();
    let exit_event_loop = eloop.create_exiter();
    let window = transparent_window::TransparentWindow::new(100, 100);
    let (tx, rx) = channel();

    let keyhook = keyboard_hook::KeyboardHook::install(tx);

    thread::spawn(move|| {
        ui_thread::run(rx);
        exit_event_loop();
    });

    println!("Starting Enso.");
    println!("To exit, hold down CAPS LOCK and type 'QUIT'.");

    eloop.run();
    keyhook.uninstall();
    window.close();

    println!("Farewell.");
}
