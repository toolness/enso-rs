use std::sync::mpsc::Receiver;

use super::events::Event;

pub fn run(receiver: Receiver<Event>) {
    loop {
        match receiver.recv() {
            Ok(event) => {
                println!("{:?}", event);
                match event {
                    Event::QuasimodeEnd => {
                        break;
                    }
                    _ => {}
                };
            },
            Err(e) => {
                println!("Receive error {:?}", e);
                break;
            }
        }
    }
}
