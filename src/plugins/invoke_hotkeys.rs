use std::convert::TryFrom;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::command::SimpleCommand;
use crate::error::Error;
use crate::system::{get_enso_home_dir, press_key, KeyDirection, VirtualKey};
use crate::ui::{UserInterface, UserInterfacePlugin};
use crate::windows_util::{get_foreground_executable_path, get_foreground_window_name};

#[derive(Debug, Clone)]
struct HotkeyCombination {
    pub keys: Vec<VirtualKey>,
}

impl HotkeyCombination {
    pub fn press(&self) -> Result<(), Error> {
        for key in self.keys.iter() {
            press_key(*key, KeyDirection::Down)?;
        }
        for key in self.keys.iter().rev() {
            press_key(*key, KeyDirection::Up)?;
        }
        Ok(())
    }
}

impl TryFrom<&str> for HotkeyCombination {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut result: Vec<VirtualKey> = Vec::new();
        let keys = s.split('+');
        for key in keys {
            let vkey = VirtualKey::try_from(key.trim())?;
            result.push(vkey);
        }
        Ok(HotkeyCombination { keys: result })
    }
}

#[derive(Default)]
pub struct InvokeHotkeysPlugin {
    commands_loaded: Vec<String>,
    last_update: Option<SystemTime>,
}

impl InvokeHotkeysPlugin {
    fn unload(&mut self, ui: &mut UserInterface) -> Result<(), Error> {
        if self.commands_loaded.len() > 0 {
            println!("Unloading {} hotkey commands.", self.commands_loaded.len());
            for command in self.commands_loaded.iter() {
                ui.remove_command(command);
            }
            self.commands_loaded.clear();
        }
        Ok(())
    }

    fn parse_hotkeys(&self, hotkeys_path: PathBuf) -> Result<HotkeyParseResult, Error> {
        let mut line_no = 0;
        let mut result = HotkeyParseResult {
            warnings: vec![],
            sections: vec![],
        };
        let mut current_section = HotkeySection {
            name: "Global".to_string(),
            exe_filter: None,
            commands: vec![],
        };
        for line in std::fs::read_to_string(hotkeys_path)?.lines() {
            line_no += 1;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with("@") {
                let mut parts = line.splitn(2, ' ');
                let directive = parts.next().unwrap().trim();
                match directive {
                    "@exefilter" => {
                        if current_section.exe_filter.is_some() {
                            result.warnings.push(format!(
                                "Duplicate @exefilter directive on line {}",
                                line_no
                            ));
                            continue;
                        }
                        let Some(exefilter) = parts.next() else {
                            result.warnings.push(format!(
                                "No value found for @exefilter on line {}",
                                line_no
                            ));
                            continue;
                        };
                        current_section.exe_filter = Some(exefilter.trim().to_string());
                    }
                    "@app" => {
                        let Some(app) = parts.next() else {
                            result.warnings.push(format!(
                                "No value found for @app on line {}",
                                line_no
                            ));
                            continue;
                        };
                        result.sections.push(current_section);
                        current_section = HotkeySection {
                            name: app.trim().to_string(),
                            exe_filter: None,
                            commands: vec![],
                        };
                    }
                    _ => {
                        result.warnings.push(format!(
                            "Unrecognized directive \"{}\" on line {}",
                            directive, line_no
                        ));
                        continue;
                    }
                }
            } else {
                let mut parts = line.rsplitn(2, ':');
                let hotkey_str = parts.next().unwrap().trim();
                let Some(command_name) = parts.next() else {
                    result.warnings.push(format!("No colon found on line {}: {}", line_no, line));
                    continue;
                };
                match HotkeyCombination::try_from(hotkey_str) {
                    Ok(hotkey) => {
                        let command = HotkeyCommand {
                            name: format!("{} ({})", command_name.trim(), hotkey_str),
                            hotkey,
                        };
                        current_section.commands.push(command);
                    }
                    Err(e) => {
                        result
                            .warnings
                            .push(format!("Error parsing hotkey on line {}: {}", line_no, e));
                        continue;
                    }
                }
            }
        }
        result.sections.push(current_section);
        Ok(result)
    }

    pub fn reload(&mut self, ui: &mut UserInterface, hotkeys_path: PathBuf) -> Result<(), Error> {
        println!("Parsing hotkeys from \"{}\".", hotkeys_path.display());

        let result = self.parse_hotkeys(hotkeys_path)?;

        if result.warnings.len() > 0 {
            let message = format!(
                "Problems occurred while parsing hotkeys file:\n{}",
                result.warnings.join("\n")
            );
            eprintln!("{}", message);
            ui.show_message(message)?;
        }

        self.unload(ui)?;
        for section in result.sections {
            let _ = section.name; // TODO: We should use this in the help text.
            for command in section.commands {
                let hotkey = command.hotkey;
                if ui.has_command(&command.name) {
                    println!("Command \"{}\" already exists, skipping.", command.name);
                } else {
                    self.commands_loaded.push(command.name.clone());
                    let simple_command =
                        SimpleCommand::new(command.name, move |_ui| hotkey.press());
                    ui.add_command(simple_command.into_box());
                }
            }
        }
        println!("Loaded {} hotkey commands.", self.commands_loaded.len());
        Ok(())
    }

    pub fn maybe_reload(&mut self, ui: &mut UserInterface) -> Result<(), Error> {
        let mut hotkeys_path = get_enso_home_dir()?;
        hotkeys_path.push("hotkeys.txt");
        if hotkeys_path.exists() {
            let metadata = std::fs::metadata(&hotkeys_path)?;
            let new_last_update = metadata.modified()?;
            if let Some(last_update) = self.last_update {
                if new_last_update > last_update {
                    // The hotkeys file has changed, so reload it.
                    self.last_update = Some(new_last_update);
                    self.reload(ui, hotkeys_path)
                } else {
                    // The hotkeys file hasn't changed, so do nothing.
                    Ok(())
                }
            } else {
                // We haven't loaded hotkeys before, so load them now.
                self.last_update = Some(new_last_update);
                self.reload(ui, hotkeys_path)
            }
        } else if self.last_update.is_some() {
            // We've loaded hotkeys before, but now the hotkeys.txt file is gone, so unload them.
            self.last_update = None;
            self.unload(ui)
        } else {
            // No hotkeys file, and we haven't loaded any hotkeys yet, so do nothing.
            Ok(())
        }
    }
}

impl UserInterfacePlugin for InvokeHotkeysPlugin {
    fn init(&mut self, ui: &mut UserInterface) -> Result<(), Error> {
        self.maybe_reload(ui)?;
        ui.add_simple_command("show foreground window info", |ui| {
            let window_name = get_foreground_window_name().unwrap_or(String::from("ERR"));
            let executable_path = get_foreground_executable_path().unwrap_or(String::from("ERR"));
            ui.show_message(format!(
                "Window name: {}\nExecutable path: {}",
                window_name, executable_path
            ))?;
            Ok(())
        });
        Ok(())
    }

    fn on_quasimode_start(&mut self, ui: &mut UserInterface) -> Result<(), Error> {
        self.maybe_reload(ui)
    }
}

struct HotkeySection {
    name: String,
    exe_filter: Option<String>,
    commands: Vec<HotkeyCommand>,
}

struct HotkeyCommand {
    name: String,
    hotkey: HotkeyCombination,
}

struct HotkeyParseResult {
    warnings: Vec<String>,
    sections: Vec<HotkeySection>,
}
