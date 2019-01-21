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
    use direct2d::render_target::RenderTarget;

    let eloop = event_loop::EventLoop::new();
    let mut window = transparent_window::TransparentWindow::new(20, 20, 100, 100);
    window.draw_and_update(|target| {
        target.clear(0xFF_FF_FF);
    });
    let (tx, rx) = channel();

    let keyhook = keyboard_hook::KeyboardHook::install(tx, eloop.get_thread_id());
    let mut ui = ui::UserInterface::new();

    println!("Starting Enso.");
    println!("To exit, hold down CAPS LOCK and type 'QUIT'.");

    eloop.run(|| ui.process_event_receiver(&rx));

    keyhook.uninstall();
    window.close();

    println!("Farewell.");
}
