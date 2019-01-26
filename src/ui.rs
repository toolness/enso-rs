use winapi::um::winuser::VK_BACK;
use std::sync::mpsc::{Receiver, TryRecvError};
use direct2d::render_target::RenderTarget;
use directwrite::factory::Factory;
use directwrite::TextFormat;
use direct2d::math::{RectF, ColorF};
use direct2d::brush::solid_color::SolidColorBrush;
use direct2d::enums::DrawTextOptions;

use super::events::Event;
use super::windows_util::vkey_to_char;
use super::transparent_window::TransparentWindow;
use super::directx::Direct3DDevice;
use super::error::Error;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

pub struct UserInterface {
    cmd: String,
    d3d_device: Direct3DDevice,
    text_format: TextFormat,
    window: Option<TransparentWindow>
}

impl UserInterface {
    pub fn new(d3d_device: Direct3DDevice) -> Result<Self, Error> {
        let dw_factory = Factory::new()?;
        let text_format = TextFormat::create(&dw_factory)
            .with_family("Georgia")
            .with_size(36.0)
            .build()?;
        Ok(UserInterface {
            cmd: String::new(),
            d3d_device,
            text_format,
            window: None
        })
    }

    pub fn process_event_receiver(&mut self, receiver: &Receiver<Event>) -> Result<bool, Error> {
        loop {
            match receiver.try_recv() {
                Ok(event) => {
                    if self.process_event(event)? {
                        return Ok(true);
                    }
                },
                Err(TryRecvError::Empty) => {
                    return Ok(false);
                },
                Err(TryRecvError::Disconnected) => {
                    return Err(Error::Other(Box::new(TryRecvError::Disconnected)));
                }
            }
        }
    }

    fn draw_quasimode(&mut self) -> Result<(), Error> {
        let text = self.cmd.as_str();
        let text_format = &self.text_format;
        if let Some(ref mut window) = self.window {
            window.draw_and_update(|target| {
                let rect = RectF::new(0.0, 0.0, WIDTH as f32, HEIGHT as f32);
                let brush = SolidColorBrush::create(&target)
                    .with_color(0xFF_FF_FF)
                    .build()?;
                target.clear(ColorF::uint_rgb(0, 0.8));
                target.draw_text(
                    text,
                    &text_format,
                    rect,
                    &brush,
                    DrawTextOptions::NONE
                );
                Ok(())
            })?;
        }
        Ok(())
    }

    pub fn process_event(&mut self, event: Event) -> Result<bool, Error> {
        match event {
            Event::QuasimodeStart => {
                println!("Starting quasimode.");
                self.cmd.clear();
                let window = TransparentWindow::new(&mut self.d3d_device, 0, 0, WIDTH, HEIGHT)?;

                self.window = Some(window);
                self.draw_quasimode()?;
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
                    self.draw_quasimode()?;
                    println!("Command so far: {}", self.cmd);
                }
            },
        };
        return Ok(false);
    }
}
