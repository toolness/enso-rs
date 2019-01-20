use std::sync::mpsc::Receiver;
use winapi::um::winuser::VK_BACK;

use super::events::Event;
use super::windows_util::vkey_to_char;

pub fn run(receiver: Receiver<Event>) {
    let mut cmd = String::new();
    loop {
        match receiver.recv() {
            Ok(event) => {
                match event {
                    Event::QuasimodeStart => {
                        println!("Starting quasimode.");
                        cmd.clear();
                    },
                    Event::QuasimodeEnd => {
                        println!("Ending quasimode.");
                        match cmd.as_str() {
                            "QUIT" => break,
                            "" => {},
                            _ => {
                                println!("Unknown command '{}'.", cmd);
                            }
                        }
                    },
                    Event::Keypress(vk_code) => {
                        let changed = if vk_code == VK_BACK {
                            match cmd.pop() {
                                None => false,
                                Some(_) => true
                            }
                        } else if let Some(ch) = vkey_to_char(vk_code) {
                            cmd.push(ch);
                            true
                        } else {
                            false
                        };
                        if changed {
                            println!("Command so far: {}", cmd);
                        }
                    },
                };
            },
            Err(e) => {
                println!("Receive error {:?}", e);
                break;
            }
        }
    }
}
