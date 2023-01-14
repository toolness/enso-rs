// The following will ensure that we won't spawn a Windows console
// in release, but will in debug and testing.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::commands::DefaultCommandsPlugin;

extern crate winapi;

mod autocomplete_map;
mod command;
mod commands;
mod directx;
mod error;
mod event_loop;
mod keyboard_hook;
mod menu;
mod transparent_window;
mod ui;
mod windows_util;

fn run_enso() -> Result<(), Box<error::Error>> {
    use std::sync::mpsc::channel;

    let d3d_device = directx::Direct3DDevice::new()?;
    println!(
        "Created Direct3D device with feature level 0x{:x}.",
        d3d_device.get_feature_level()
    );

    let eloop = event_loop::EventLoop::new();
    let (tx, rx) = channel();

    windows_util::disable_caps_lock()?;

    let keyhook = keyboard_hook::KeyboardHook::install(tx, eloop.get_thread_id());
    let mut ui = ui::UserInterface::new(d3d_device)?;

    ui.add_plugin(DefaultCommandsPlugin::new())?;

    ui.show_message("Welcome to Enso!")?;

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
