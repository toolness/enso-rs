use direct2d::brush::solid_color::SolidColorBrush;
use direct2d::enums::DrawTextOptions;
use direct2d::math::ColorF;
use direct2d::render_target::RenderTarget;
use directwrite::factory::Factory;
use directwrite::{TextFormat, TextLayout};
use std::sync::mpsc::{Receiver, TryRecvError};
use winapi::um::winuser::VK_BACK;

use super::directx::Direct3DDevice;
use super::error::Error;
use super::events::Event;
use super::transparent_window::TransparentWindow;
use super::windows_util::{get_primary_screen_size, send_unicode_keypress, vkey_to_char};

const PADDING: f32 = 16.0;
const PADDING_X2: f32 = PADDING * 2.0;
const BG_COLOR: u32 = 0x00_00_00;
const BG_ALPHA: f32 = 0.5;
const TEXT_COLOR: u32 = 0xFF_FF_FF;
const TEXT_ALPHA: f32 = 1.0;
const HELP_BG_COLOR: u32 = 0x7F_98_45;
const HELP_BG_ALPHA: f32 = 0.5;
const FONT_FAMILY: &'static str = "Georgia";
const FONT_SIZE: f32 = 48.0;
const MESSAGE_MAXWIDTH_PCT: f32 = 0.5;
const NOCMD_HELP: &'static str =
    "Welcome to Enso! Enter a command, or type \u{201C}help\u{201D} for assistance.";

struct Brushes {
    pub black: SolidColorBrush,
    pub white: SolidColorBrush,
    pub help_bg: SolidColorBrush,
}

impl Brushes {
    pub fn new<T: RenderTarget>(target: &mut T) -> Result<Self, Error> {
        let black = SolidColorBrush::create(&target)
            .with_color(ColorF::uint_rgb(BG_COLOR, BG_ALPHA))
            .build()?;
        let white = SolidColorBrush::create(&target)
            .with_color(ColorF::uint_rgb(TEXT_COLOR, TEXT_ALPHA))
            .build()?;
        let help_bg = SolidColorBrush::create(&target)
            .with_color(ColorF::uint_rgb(HELP_BG_COLOR, HELP_BG_ALPHA))
            .build()?;
        Ok(Brushes {
            black,
            white,
            help_bg,
        })
    }
}

struct TransparentMessageRenderer {
    window: TransparentWindow,
}

impl TransparentMessageRenderer {
    pub fn new(
        text: String,
        d3d_device: &mut Direct3DDevice,
        dw_factory: &Factory,
        text_format: &TextFormat,
    ) -> Result<Self, Error> {
        let (screen_width, screen_height) = get_primary_screen_size()?;
        let text_layout = TextLayout::create(dw_factory)
            .with_text(text.as_str())
            .with_font(text_format)
            .with_size(screen_width as f32, screen_height as f32)
            .build()?;
        text_layout.set_max_width(screen_width as f32 * MESSAGE_MAXWIDTH_PCT)?;
        text_layout.set_max_height(screen_height as f32)?;
        let metrics = text_layout.get_metrics();
        let (text_width, text_height) = (metrics.width(), metrics.height());
        let width = text_width + PADDING_X2;
        let height = text_height + PADDING_X2;
        let x = screen_width as f32 / 2.0 - width / 2.0;
        let y = screen_height as f32 / 2.0 - height / 2.0;
        let window =
            TransparentWindow::new(d3d_device, x as i32, y as i32, width as u32, height as u32)?;
        let mut result = Self { window };
        result.window.draw_and_update(move |target| {
            let brushes = Brushes::new(target)?;
            target.clear(ColorF::uint_rgb(0, 0.0));
            target.fill_rectangle((0.0, 0.0, width, height), &brushes.black);
            target.draw_text_layout(
                (PADDING, PADDING),
                &text_layout,
                &brushes.white,
                DrawTextOptions::NONE,
            );
            Ok(())
        })?;
        Ok(result)
    }
}

struct QuasimodeRenderer {
    window: TransparentWindow,
}

impl QuasimodeRenderer {
    pub fn new(d3d_device: &mut Direct3DDevice) -> Result<Self, Error> {
        let (width, height) = get_primary_screen_size()?;
        let window = TransparentWindow::new(d3d_device, 0, 0, width, height)?;
        Ok(Self { window })
    }

    pub fn draw(
        &mut self,
        cmd: &String,
        dw_factory: &Factory,
        text_format: &TextFormat,
    ) -> Result<(), Error> {
        let (screen_width, screen_height) = self.window.get_size();

        // Eventually this will be dynamically generated based on the currently matched command.
        let help_text = NOCMD_HELP;

        let help_layout = TextLayout::create(dw_factory)
            .with_text(help_text)
            .with_font(text_format)
            .with_size(screen_width as f32, screen_height as f32)
            .build()?;
        let cmd_layout = TextLayout::create(dw_factory)
            .with_text(cmd)
            .with_font(text_format)
            .with_size(screen_width as f32, screen_height as f32)
            .build()?;
        self.window.draw_and_update(move |target| {
            let brushes = Brushes::new(target)?;
            target.clear(ColorF::uint_rgb(0, 0.0));
            let help_met = help_layout.get_metrics();
            let (help_width, help_height) = (
                help_met.width() + PADDING_X2,
                help_met.height() + PADDING_X2,
            );
            target.fill_rectangle((0.0, 0.0, help_width, help_height), &brushes.help_bg);
            target.draw_text_layout(
                (PADDING, PADDING),
                &help_layout,
                &brushes.white,
                DrawTextOptions::NONE,
            );
            if cmd.len() > 0 {
                let cmd_met = cmd_layout.get_metrics();
                target.fill_rectangle(
                    (
                        0.0,
                        help_height,
                        cmd_met.width() + PADDING_X2,
                        help_height + cmd_met.height() + PADDING_X2,
                    ),
                    &brushes.black,
                );
                target.draw_text_layout(
                    (PADDING, help_height + PADDING),
                    &cmd_layout,
                    &brushes.white,
                    DrawTextOptions::NONE,
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
    quasimode: Option<QuasimodeRenderer>,
    message: Option<TransparentMessageRenderer>,
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
            quasimode: None,
            message: None,
        })
    }

    pub fn show_message(&mut self, text: &str) -> Result<(), Error> {
        self.message = Some(TransparentMessageRenderer::new(
            String::from(text),
            &mut self.d3d_device,
            &self.dw_factory,
            &self.text_format,
        )?);
        Ok(())
    }

    pub fn type_char(&mut self, ch: &str) -> Result<(), Error> {
        send_unicode_keypress(ch)
    }

    pub fn process_event_receiver(&mut self, receiver: &Receiver<Event>) -> Result<bool, Error> {
        loop {
            match receiver.try_recv() {
                Ok(event) => {
                    if self.process_event(event)? {
                        return Ok(true);
                    }
                }
                Err(TryRecvError::Empty) => {
                    return Ok(false);
                }
                Err(TryRecvError::Disconnected) => {
                    return Err(Error::Other(Box::new(TryRecvError::Disconnected)));
                }
            }
        }
    }

    pub fn process_event(&mut self, event: Event) -> Result<bool, Error> {
        let mut redraw_quasimode = false;
        if self.message.is_some() {
            match event {
                Event::QuasimodeStart | Event::QuasimodeEnd => self.message = None,
                _ => {}
            }
        }
        match event {
            Event::QuasimodeStart => {
                println!("Starting quasimode.");
                self.cmd.clear();
                self.quasimode = Some(QuasimodeRenderer::new(&mut self.d3d_device)?);
                redraw_quasimode = true;
            }
            Event::QuasimodeEnd => {
                println!("Ending quasimode.");
                self.quasimode = None;
                match self.cmd.as_str() {
                    "quit" => return Ok(true),
                    "tada" => self.type_char("🎉")?,
                    "help" => {
                        self.show_message("Sorry, still need to implement help!")?;
                    }
                    "" => {}
                    _ => {
                        println!("Unknown command '{}'.", self.cmd);
                        let msg = format!(
                            "Alas, I am unfamiliar with the \u{201C}{}\u{201D} command.",
                            self.cmd
                        );
                        self.show_message(msg.as_str())?;
                    }
                }
            }
            Event::Keypress(vk_code) => {
                redraw_quasimode = if vk_code == VK_BACK {
                    match self.cmd.pop() {
                        None => false,
                        Some(_) => true,
                    }
                } else if let Some(ch) = vkey_to_char(vk_code) {
                    for lch in ch.to_lowercase() {
                        self.cmd.push(lch);
                    }
                    true
                } else {
                    false
                };
            }
        };
        if redraw_quasimode {
            if let Some(ref mut quasimode) = self.quasimode {
                quasimode.draw(&self.cmd, &self.dw_factory, &self.text_format)?;
            }
        }
        return Ok(false);
    }
}
