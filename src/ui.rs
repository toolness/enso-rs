use direct2d::brush::solid_color::SolidColorBrush;
use direct2d::enums::DrawTextOptions;
use direct2d::math::ColorF;
use direct2d::render_target::RenderTarget;
use directwrite::factory::Factory;
use directwrite::{TextFormat, TextLayout};
use std::sync::mpsc::{Receiver, TryRecvError};
use winapi::um::winuser::VK_BACK;

use super::autocomplete_map::AutocompleteMap;
use super::command::{Command, SimpleCommand};
use super::directx::Direct3DDevice;
use super::error::Error;
use super::events::Event;
use super::menu::Menu;
use super::transparent_window::TransparentWindow;
use super::windows_util::{get_primary_screen_size, send_unicode_keypress, vkey_to_char};

type ColorAlpha = (u32, f32);

const PADDING: f32 = 16.0;
const PADDING_X2: f32 = PADDING * 2.0;
const DEFAULT_BG: ColorAlpha = (0x00_00_00, 0.75);
const DEFAULT_FG: ColorAlpha = (0xFF_FF_FF, 1.0);
const HELP_BG: ColorAlpha = (0x7F_98_45, 0.75);
const HELP_FG: ColorAlpha = DEFAULT_FG;
const AUTOCOMPLETED_FG: ColorAlpha = (0x7F_98_45, 1.0);
const UNSELECTED_INPUT_FG: ColorAlpha = (0xAF_BC_92, 1.0);
const FONT_FAMILY: &'static str = "Georgia";
const FONT_SIZE: f32 = 48.0;
const SMALL_FONT_SIZE: f32 = 24.0;
const MESSAGE_MAXWIDTH_PCT: f32 = 0.5;
const NOCMD_HELP: &'static str =
    "Welcome to Enso! Enter a command, or type \u{201C}help\u{201D} for assistance.";

fn make_simple_brush<T: RenderTarget>(
    target: &mut T,
    color_alpha: ColorAlpha,
) -> Result<SolidColorBrush, Error> {
    let (color, alpha) = color_alpha;
    let brush = SolidColorBrush::create(&target)
        .with_color(ColorF::uint_rgb(color, alpha))
        .build()?;
    Ok(brush)
}

struct Brushes {
    pub default_bg: SolidColorBrush,
    pub default_fg: SolidColorBrush,
    pub help_bg: SolidColorBrush,
    pub help_fg: SolidColorBrush,
    pub autocompleted_fg: SolidColorBrush,
    pub unselected_input_fg: SolidColorBrush,
}

impl Brushes {
    pub fn new<T: RenderTarget>(target: &mut T) -> Result<Self, Error> {
        Ok(Brushes {
            default_bg: make_simple_brush(target, DEFAULT_BG)?,
            default_fg: make_simple_brush(target, DEFAULT_FG)?,
            help_bg: make_simple_brush(target, HELP_BG)?,
            help_fg: make_simple_brush(target, HELP_FG)?,
            autocompleted_fg: make_simple_brush(target, AUTOCOMPLETED_FG)?,
            unselected_input_fg: make_simple_brush(target, UNSELECTED_INPUT_FG)?,
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
            target.fill_rectangle((0.0, 0.0, width, height), &brushes.default_bg);
            target.draw_text_layout(
                (PADDING, PADDING),
                &text_layout,
                &brushes.default_fg,
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
        help_text: &String,
        dw_factory: &Factory,
        text_format: &TextFormat,
        small_text_format: &TextFormat,
    ) -> Result<(), Error> {
        let (screen_width, screen_height) = self.window.get_size();
        let help_layout = TextLayout::create(dw_factory)
            .with_text(help_text)
            .with_font(small_text_format)
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
                &brushes.help_fg,
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
                    &brushes.default_bg,
                );
                // The following can be used to change the color of individual letters:
                // cmd_layout
                //  .set_drawing_effect(&brushes.autocompleted_fg, 0..1)
                //  .unwrap();
                target.draw_text_layout(
                    (PADDING, help_height + PADDING),
                    &cmd_layout,
                    &brushes.default_fg,
                    DrawTextOptions::NONE,
                );
            }
            Ok(())
        })?;
        Ok(())
    }
}

pub struct UserInterface {
    input: String,
    should_quit: bool,
    d3d_device: Direct3DDevice,
    dw_factory: Factory,
    text_format: TextFormat,
    small_text_format: TextFormat,
    quasimode: Option<QuasimodeRenderer>,
    message: Option<TransparentMessageRenderer>,
    commands: AutocompleteMap<Box<dyn Command>>,
}

impl UserInterface {
    pub fn new(d3d_device: Direct3DDevice) -> Result<Self, Error> {
        let dw_factory = Factory::new()?;
        let text_format = TextFormat::create(&dw_factory)
            .with_family(FONT_FAMILY)
            .with_size(FONT_SIZE)
            .build()?;
        let small_text_format = TextFormat::create(&dw_factory)
            .with_family(FONT_FAMILY)
            .with_size(SMALL_FONT_SIZE)
            .build()?;
        let mut ui = UserInterface {
            input: String::new(),
            should_quit: false,
            d3d_device,
            dw_factory,
            text_format,
            small_text_format,
            quasimode: None,
            message: None,
            commands: AutocompleteMap::new(),
        };
        ui.add_builtin_commands();
        Ok(ui)
    }

    fn add_builtin_commands(&mut self) {
        self.add_command(
            SimpleCommand::new("help", |ui| {
                ui.show_message("Sorry, still need to implement help!")
            })
            .into_box(),
        );

        self.add_command(SimpleCommand::new("quit", |ui| ui.quit()).into_box());
    }

    pub fn add_command(&mut self, command: Box<dyn Command>) {
        self.commands.insert(command.name(), command);
    }

    pub fn quit(&mut self) -> Result<(), Error> {
        self.should_quit = true;
        Ok(())
    }

    pub fn show_message<S: Into<String>>(&mut self, text: S) -> Result<(), Error> {
        self.message = Some(TransparentMessageRenderer::new(
            text.into(),
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
                self.input.clear();
                self.quasimode = Some(QuasimodeRenderer::new(&mut self.d3d_device)?);
                redraw_quasimode = true;
            }
            Event::QuasimodeEnd => {
                println!("Ending quasimode.");
                self.quasimode = None;
                if self.input.len() > 0 {
                    let suggs = self.commands.autocomplete(&self.input, 1);
                    let menu = Menu::from(suggs);
                    if let Some(mut sugg) = menu.into_selected_entry() {
                        sugg.value.execute(self)?;
                    } else {
                        println!("Unknown command '{}'.", self.input);
                        self.show_message(format!(
                            "Alas, I am unfamiliar with the \u{201C}{}\u{201D} command.",
                            self.input
                        ))?;
                    }
                }
            }
            Event::Keypress(vk_code) => {
                redraw_quasimode = if vk_code == VK_BACK {
                    match self.input.pop() {
                        None => false,
                        Some(_) => true,
                    }
                } else if let Some(ch) = vkey_to_char(vk_code) {
                    for lch in ch.to_lowercase() {
                        self.input.push(lch);
                    }
                    true
                } else {
                    false
                };
            }
        };
        if redraw_quasimode {
            if let Some(ref mut quasimode) = self.quasimode {
                // Eventually this will be dynamically generated based on the currently matched command.
                let help_text = String::from(NOCMD_HELP);

                quasimode.draw(
                    &self.input,
                    &help_text,
                    &self.dw_factory,
                    &self.text_format,
                    &self.small_text_format,
                )?;
            }
        }
        return Ok(self.should_quit);
    }
}
