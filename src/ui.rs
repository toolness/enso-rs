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

const PADDING: f32 = 16.0;
const BG_COLOR: u32 = 0x00_00_00;
const BG_ALPHA: f32 = 0.5;
const TEXT_COLOR: u32 = 0xFF_FF_FF;
const TEXT_ALPHA: f32 = 1.0;
const FONT_FAMILY: &'static str = "Georgia";
const FONT_SIZE: f32 = 48.0;

pub struct QuasimodeRenderer {
    window: TransparentWindow
}

impl QuasimodeRenderer {
    pub fn new(d3d_device: &mut Direct3DDevice) -> Result<Self, Error> {
        let (width, height) = get_primary_screen_size()?;
        let window = TransparentWindow::new(d3d_device, 0, 0, width, height)?;
        Ok(Self { window })
    }

    pub fn draw(&mut self, cmd: &String, dw_factory: &Factory, text_format: &TextFormat) -> Result<(), Error> {
        let (screen_width, screen_height) = self.window.get_size();
        let text_layout = TextLayout::create(dw_factory)
            .with_text(cmd)
            .with_font(text_format)
            .with_size(screen_width as f32, screen_height as f32)
            .build()?;
        let metrics = text_layout.get_metrics();
        let (text_width, text_height) = (metrics.width(), metrics.height());
        self.window.draw_and_update(move|target| {
            let black_brush = SolidColorBrush::create(&target)
                .with_color(ColorF::uint_rgb(BG_COLOR, BG_ALPHA))
                .build()?;
            let white_brush = SolidColorBrush::create(&target)
                .with_color(ColorF::uint_rgb(TEXT_COLOR, TEXT_ALPHA))
                .build()?;
            target.clear(ColorF::uint_rgb(0, 0.0));
            if cmd.len() > 0 {
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
        Ok(())
    }
}

pub struct UserInterface {
    cmd: String,
    d3d_device: Direct3DDevice,
    dw_factory: Factory,
    text_format: TextFormat,
    quasimode: Option<QuasimodeRenderer>
}

impl UserInterface {
    pub fn new(d3d_device: Direct3DDevice) -> Result<Self, Error> {
        let dw_factory = Factory::new()?;
        let text_format = TextFormat::create(&dw_factory)
            .with_family(FONT_FAMILY)
            .with_size(FONT_SIZE)
            .build()?;
        Ok(UserInterface {
            cmd: String::new(),
            d3d_device,
            dw_factory,
            text_format,
            quasimode: None
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

    pub fn process_event(&mut self, event: Event) -> Result<bool, Error> {
        let mut redraw_quasimode = false;
        match event {
            Event::QuasimodeStart => {
                println!("Starting quasimode.");
                self.cmd.clear();
                self.quasimode = Some(QuasimodeRenderer::new(&mut self.d3d_device)?);
                redraw_quasimode = true;
            },
            Event::QuasimodeEnd => {
                println!("Ending quasimode.");
                self.quasimode = None;
                match self.cmd.as_str() {
                    "quit" => return Ok(true),
                    "" => {},
                    _ => {
                        println!("Unknown command '{}'.", self.cmd);
                    }
                }
            },
            Event::Keypress(vk_code) => {
                redraw_quasimode = if vk_code == VK_BACK {
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
            },
        };
        if redraw_quasimode {
            if let Some(ref mut quasimode) = self.quasimode {
                quasimode.draw(&self.cmd, &self.dw_factory, &self.text_format)?;
            }
        }
        return Ok(false);
    }
}
