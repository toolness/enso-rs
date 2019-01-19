extern crate winapi;

mod keyboard_hook;
mod event_loop;
mod windows_util;
mod events;

fn main() {
    let keyhook = keyboard_hook::KeyboardHook::install();

    println!("Installed key hook.");
    println!("Press CAPS LOCK to exit.");

    event_loop::run();

    keyhook.uninstall();

    println!("Farewell.");
}
