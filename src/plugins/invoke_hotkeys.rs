use std::convert::TryFrom;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::command::{Command, SimpleCommand};
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

    pub fn reload(&mut self, ui: &mut UserInterface, hotkeys_path: PathBuf) -> Result<(), Error> {
        println!("Parsing hotkeys from \"{}\".", hotkeys_path.display());
        let mut warnings: Vec<String> = Vec::new();
        let mut commands: Vec<Box<dyn Command>> = Vec::new();
        let mut line_no = 0;
        for line in std::fs::read_to_string(hotkeys_path)?.lines() {
            line_no += 1;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut parts = line.rsplitn(2, ':');
            let hotkey_str = parts.next().unwrap().trim();
            match HotkeyCombination::try_from(hotkey_str) {
                Ok(hotkey) => {
                    let Some(command_name) = parts.next() else {
                        warnings.push(format!("No colon found on line {}: {}", line_no, line));
                        continue;
                    };
                    let command_name = format!("{} ({})", command_name.trim(), hotkey_str);
                    let command =
                        SimpleCommand::new(command_name.clone(), move |_ui| hotkey.press());
                    commands.push(command.into_box());
                }
                Err(e) => {
                    warnings.push(format!("Error parsing hotkey on line {}: {}", line_no, e));
                    continue;
                }
            }
        }

        if warnings.len() > 0 {
            let message = format!(
                "Problems occurred while parsing hotkeys file:\n{}",
                warnings.join("\n")
            );
            eprintln!("{}", message);
            ui.show_message(message)?;
        }

        self.unload(ui)?;
        for command in commands {
            let command_name = command.name();
            if ui.has_command(&command_name) {
                println!("Command \"{}\" already exists, skipping.", command_name);
            } else {
                self.commands_loaded.push(command_name);
                ui.add_command(command);
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
