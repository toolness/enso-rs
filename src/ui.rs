use direct2d::brush::solid_color::SolidColorBrush;
use direct2d::enums::DrawTextOptions;
use direct2d::math::ColorF;
use direct2d::render_target::RenderTarget;
use directwrite::factory::Factory;
use directwrite::{TextFormat, TextLayout};
use std::convert::TryFrom;
use std::ops::Range;
use std::sync::mpsc::{Receiver, TryRecvError};
use winapi::um::winuser::{VK_BACK, VK_DOWN, VK_UP};

use crate::command::SimpleCommand;
use crate::windows_util::{send_modifier_keypress, send_raw_keypress_for_char};

use super::autocomplete_map::{AutocompleteMap, AutocompleteSuggestion};
use super::command::Command;
use super::directx::Direct3DDevice;
use super::error::Error;
use super::keyboard_hook::HookEvent;
use super::menu::Menu;
use super::transparent_window::TransparentWindow;
use super::windows_util::{get_primary_screen_size, send_unicode_keypress, vkey_to_char};

type ColorAlpha = (u32, f32);

const MAX_SUGGESTIONS: usize = 5;
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
const NOCMD_HELP: &'static str = "No command matches your input.";
const EMPTY_INPUT_HELP: &'static str =
    "Welcome to Enso! Enter a command, or type \u{201C}help\u{201D} for assistance.";

pub enum KeyDirection {
    Up,
    Down,
}

pub enum ModifierKey {
    Shift,
    Alt,
    Control,
}

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
        input: &String,
        optional_menu: &Option<Menu<AutocompleteSuggestion<Box<dyn Command>>>>,
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
        let mut menu_layouts: Vec<(TextLayout, bool, Vec<Range<usize>>)> = vec![];
        if let Some(menu) = optional_menu {
            for (sugg, is_selected) in menu.iter() {
                let menu_layout = TextLayout::create(dw_factory)
                    .with_text(sugg.name.as_ref())
                    .with_font(text_format)
                    .with_size(screen_width as f32, screen_height as f32)
                    .build()?;
                menu_layouts.push((menu_layout, is_selected, sugg.matches.clone()));
            }
        } else if input.len() > 0 {
            let cmd_layout = TextLayout::create(dw_factory)
                .with_text(input)
                .with_font(text_format)
                .with_size(screen_width as f32, screen_height as f32)
                .build()?;
            menu_layouts.push((cmd_layout, true, vec![0..input.len()]));
        }
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
            let mut y = help_height;
            for (menu_layout, is_selected, matches) in menu_layouts {
                let menu_met = menu_layout.get_metrics();
                let new_y = y + menu_met.height() + PADDING_X2;
                target.fill_rectangle(
                    (0.0, y, menu_met.width() + PADDING_X2, new_y),
                    &brushes.default_bg,
                );
                for input_match in matches {
                    let brush = if is_selected {
                        &brushes.default_fg
                    } else {
                        &brushes.unselected_input_fg
                    };
                    let u32_match = (input_match.start as u32)..(input_match.end as u32);
                    menu_layout
                        .set_drawing_effect(brush, u32_match)
                        .expect("setting brush for input highlight should work");
                }
                target.draw_text_layout(
                    (PADDING, y + PADDING),
                    &menu_layout,
                    &brushes.autocompleted_fg,
                    DrawTextOptions::NONE,
                );
                y = new_y;
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
    menu: Option<Menu<AutocompleteSuggestion<Box<dyn Command>>>>,
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
        let ui = UserInterface {
            input: String::new(),
            should_quit: false,
            d3d_device,
            dw_factory,
            text_format,
            small_text_format,
            quasimode: None,
            message: None,
            menu: None,
            commands: AutocompleteMap::new(),
        };
        Ok(ui)
    }

    pub fn add_simple_command(
        &mut self,
        name: &str,
        callback: impl FnMut(&mut UserInterface) -> Result<(), Error> + Clone + 'static,
    ) {
        self.add_command(Box::new(SimpleCommand::new(name, callback)));
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

    pub fn press_modifier_key(
        &mut self,
        key: ModifierKey,
        direction: KeyDirection,
    ) -> Result<(), Error> {
        send_modifier_keypress(key, direction)
    }

    /// Press the key that corresponds to the given ASCII character. Use this
    /// if you are simulating a hotkey combination, etc.
    pub fn press_key(&mut self, ch: char, direction: KeyDirection) -> Result<bool, Error> {
        send_raw_keypress_for_char(ch, direction)
    }

    /// Insert the given unicode character into the current application. This doesn't
    /// take into account the current modifier keys or anything.
    pub fn type_char(&mut self, ch: &str) -> Result<(), Error> {
        send_unicode_keypress(ch)
    }

    pub fn process_event_receiver(
        &mut self,
        receiver: &Receiver<HookEvent>,
    ) -> Result<bool, Error> {
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

    pub fn process_event(&mut self, event: HookEvent) -> Result<bool, Error> {
        let mut redraw_quasimode = false;
        if self.message.is_some() {
            match event {
                HookEvent::QuasimodeStart | HookEvent::QuasimodeEnd => self.message = None,
                _ => {}
            }
        }
        match event {
            HookEvent::QuasimodeStart => {
                println!("Starting quasimode.");
                self.input.clear();
                self.quasimode = Some(QuasimodeRenderer::new(&mut self.d3d_device)?);
                redraw_quasimode = true;
            }
            HookEvent::QuasimodeEnd => {
                println!("Ending quasimode.");
                self.quasimode = None;
                if let Some(menu) = self.menu.take() {
                    let mut sugg = menu.into_selected_entry();
                    sugg.value.execute(self)?;
                } else if self.input.len() > 0 {
                    println!("Unknown command '{}'.", self.input);
                    self.show_message(format!(
                        "Alas, I am unfamiliar with the \u{201C}{}\u{201D} command.",
                        self.input
                    ))?;
                }
            }
            HookEvent::Keypress(vk_code) => {
                let input_changed = if vk_code == VK_BACK {
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

                if input_changed {
                    let suggs = self.commands.autocomplete(&self.input, MAX_SUGGESTIONS);
                    self.menu = if let Ok(menu) = Menu::try_from(suggs) {
                        Some(menu)
                    } else {
                        None
                    };
                    redraw_quasimode = true;
                } else {
                    match vk_code {
                        VK_UP | VK_DOWN => {
                            if let Some(menu) = &mut self.menu {
                                redraw_quasimode = true;
                                if vk_code == VK_UP {
                                    menu.select_prev();
                                } else {
                                    menu.select_next();
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        };
        if redraw_quasimode {
            if let Some(ref mut quasimode) = self.quasimode {
                let help_text: String = if let Some(menu) = &self.menu {
                    let cmd_name = menu.selected_entry().value.name();
                    format!("Run the command \u{201C}{}\u{201D}.", cmd_name)
                } else if self.input.len() > 0 {
                    String::from(NOCMD_HELP)
                } else {
                    String::from(EMPTY_INPUT_HELP)
                };

                quasimode.draw(
                    &self.input,
                    &self.menu,
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
