use crate::error::Error;
use crate::ui::{UserInterface, UserInterfacePlugin};

use super::insert_commands::insert_commands;

pub struct DefaultCommandsPlugin;

impl DefaultCommandsPlugin {
    pub fn new() -> Box<dyn UserInterfacePlugin> {
        Box::new(DefaultCommandsPlugin)
    }
}

impl UserInterfacePlugin for DefaultCommandsPlugin {
    fn init(&mut self, ui: &mut UserInterface) -> Result<(), Error> {
        ui.add_simple_command("help", |ui| {
            ui.show_message("Sorry, still need to implement help!")
        });

        ui.add_simple_command("quit", |ui| ui.quit());

        insert_commands(ui);
        Ok(())
    }

    fn on_quasimode_start(&mut self, _ui: &mut UserInterface) -> Result<(), Error> {
        println!("TODO: REFRESH DEFAULT COMMANDS");
        Ok(())
    }
}
