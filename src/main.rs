extern crate winapi;

mod keyboard_hook;
mod event_loop;
mod windows_util;
mod events;
mod ui;
mod transparent_window;
mod directx;

fn main() {
    use std::sync::mpsc::{channel};

    let eloop = event_loop::EventLoop::new();
    let (tx, rx) = channel();

    let keyhook = keyboard_hook::KeyboardHook::install(tx, eloop.get_thread_id());
    let mut ui = ui::UserInterface::new();

    println!("Starting Enso.");
    println!("To exit, hold down CAPS LOCK and type 'QUIT'.");

    eloop.run(|| ui.process_event_receiver(&rx));

    keyhook.uninstall();

    println!("Farewell.");
}
