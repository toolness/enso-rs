extern crate winapi;

mod keyboard_hook;
mod event_loop;
mod windows_util;
mod events;
mod ui;
mod transparent_window;
mod directx;
mod error;

fn run_enso() -> Result<(), Box<error::Error>> {
    use std::sync::mpsc::{channel};

    let d3d_device = directx::Direct3DDevice::new()?;
    println!("Created Direct3D device with feature level 0x{:x}.", d3d_device.get_feature_level());

    let eloop = event_loop::EventLoop::new();
    let (tx, rx) = channel();

    let keyhook = keyboard_hook::KeyboardHook::install(tx, eloop.get_thread_id());
    let mut ui = ui::UserInterface::new(d3d_device);

    println!("Starting Enso.");
    println!("To exit, hold down CAPS LOCK and type 'QUIT'.");

    eloop.run(|| ui.process_event_receiver(&rx))?;

    keyhook.uninstall();

    println!("Farewell.");

    Ok(())
}

fn main() {
    std::process::exit(match run_enso() {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("Error: {}", err);
            1
        }
    });
}
