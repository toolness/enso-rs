use winapi::um::winuser::VK_BACK;
use std::sync::mpsc::{Receiver, TryRecvError};

use super::events::Event;
use super::windows_util::vkey_to_char;

pub struct UserInterface {
    cmd: String,
}

impl UserInterface {
    pub fn new() -> Self {
        UserInterface { cmd: String::new() }
    }

    pub fn process_event_receiver(&mut self, receiver: &Receiver<Event>) -> bool {
        match receiver.try_recv() {
            Ok(event) => {
                self.process_event(event)
            },
            Err(TryRecvError::Empty) => {
                false
            },
            Err(TryRecvError::Disconnected) => {
                println!("Event send channel was disconnected!");
                true
            }
        }
    }

    pub fn process_event(&mut self, event: Event) -> bool {
        match event {
            Event::QuasimodeStart => {
                println!("Starting quasimode.");
                self.cmd.clear();
            },
            Event::QuasimodeEnd => {
                println!("Ending quasimode.");
                match self.cmd.as_str() {
                    "QUIT" => return true,
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
        return false;
    }
}
