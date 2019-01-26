use winapi::um::winuser::VK_BACK;
use std::sync::mpsc::{Receiver, TryRecvError};
use direct2d::render_target::RenderTarget;
use directwrite::factory::Factory;
use directwrite::{TextFormat, TextLayout};
use direct2d::math::ColorF;
use direct2d::brush::solid_color::SolidColorBrush;
use direct2d::enums::DrawTextOptions;

use super::events::Event;
use super::windows_util::{vkey_to_char, get_primary_screen_size};
use super::transparent_window::TransparentWindow;
use super::directx::Direct3DDevice;
use super::error::Error;

const PADDING: f32 = 8.0;

pub struct UserInterface {
    cmd: String,
    d3d_device: Direct3DDevice,
    dw_factory: Factory,
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
            dw_factory,
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
        let is_cmd_empty = self.cmd.len() == 0;
        if let Some(ref mut window) = self.window {
            let (screen_width, screen_height) = window.get_size();
            let text_layout = TextLayout::create(&self.dw_factory)
                .with_text(self.cmd.as_str())
                .with_font(&self.text_format)
                .with_size(screen_width as f32, screen_height as f32)
                .build()?;
            let metrics = text_layout.get_metrics();
            let (text_width, text_height) = (metrics.width(), metrics.height());
            window.draw_and_update(move|target| {
                let black_brush = SolidColorBrush::create(&target)
                    .with_color(ColorF::uint_rgb(0, 0.5))
                    .build()?;
                let white_brush = SolidColorBrush::create(&target)
                    .with_color(0xFF_FF_FF)
                    .build()?;
                target.clear(ColorF::uint_rgb(0, 0.0));
                if !is_cmd_empty {
                    let pad = PADDING * 2.0;
                    target.fill_rectangle(
                        (0.0, 0.0, text_width + pad, text_height + pad),
                        &black_brush
                    );
                    target.draw_text_layout(
                        (PADDING, PADDING),
                        &text_layout,
                        &white_brush,
                        DrawTextOptions::NONE
                    );
                }
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
                let (width, height) = get_primary_screen_size()?;
                let window = TransparentWindow::new(&mut self.d3d_device, 0, 0, width, height)?;

                self.window = Some(window);
                self.draw_quasimode()?;
            },
            Event::QuasimodeEnd => {
                println!("Ending quasimode.");
                self.window = None;
                match self.cmd.as_str() {
                    "quit" => return Ok(true),
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
                    for lch in ch.to_lowercase() {
                        self.cmd.push(lch);
                    }
                    true
                } else {
                    false
                };
                if changed {
                    self.draw_quasimode()?;
                }
            },
        };
        return Ok(false);
    }
}
