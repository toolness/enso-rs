use winapi::um::winuser::VK_BACK;
use std::sync::mpsc::{Receiver, TryRecvError};
use direct2d::render_target::RenderTarget;

use super::events::Event;
use super::windows_util::vkey_to_char;
use super::transparent_window::TransparentWindow;
use super::directx::Direct3DDevice;
use super::error::Error;

pub struct UserInterface {
    cmd: String,
    d3d_device: Direct3DDevice,
    window: Option<TransparentWindow>
}

impl UserInterface {
    pub fn new(d3d_device: Direct3DDevice) -> Self {
        UserInterface { cmd: String::new(), d3d_device, window: None }
    }

    pub fn process_event_receiver(&mut self, receiver: &Receiver<Event>) -> Result<bool, Error> {
        match receiver.try_recv() {
            Ok(event) => {
                self.process_event(event)
            },
            Err(TryRecvError::Empty) => {
                Ok(false)
            },
            Err(TryRecvError::Disconnected) => {
                Err(Error::Other(Box::new(TryRecvError::Disconnected)))
            }
        }
    }

    pub fn process_event(&mut self, event: Event) -> Result<bool, Error> {
        match event {
            Event::QuasimodeStart => {
                println!("Starting quasimode.");
                self.cmd.clear();
                let mut window = TransparentWindow::new(&mut self.d3d_device, 20, 20, 100, 100);
                window.draw_and_update(|target| {
                    target.clear(0xFF_FF_FF);
                })?;
                self.window = Some(window);
            },
            Event::QuasimodeEnd => {
                println!("Ending quasimode.");
                self.window = None;
                match self.cmd.as_str() {
                    "QUIT" => return Ok(true),
                    "" => {},
                    _ => {
                        println!("Unknown command '{}'.", self.cmd);
                    }
                }
            },
            Event::Keypress(vk_code) => {
                let changed = if vk_code == VK_BACK {
                    match self.cmd.pop() {
                        None => false,
                        Some(_) => true
                    }
                } else if let Some(ch) = vkey_to_char(vk_code) {
                    self.cmd.push(ch);
                    true
                } else {
                    false
                };
                if changed {
                    println!("Command so far: {}", self.cmd);
                }
            },
        };
        return Ok(false);
    }
}
