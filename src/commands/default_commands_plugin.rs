use crate::command::SimpleCommand;
use crate::error::Error;
use crate::ui::{UserInterface, UserInterfacePlugin};

use super::insert_commands::insert_commands;

pub struct DefaultCommandsPlugin {
    counter: usize,
}

impl DefaultCommandsPlugin {
    pub fn new() -> Box<dyn UserInterfacePlugin> {
        Box::new(DefaultCommandsPlugin { counter: 0 })
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

    fn on_quasimode_start(&mut self, ui: &mut UserInterface) -> Result<(), Error> {
        let prev_cmd_name = format!("boop {}", self.counter);
        ui.remove_command(&prev_cmd_name);
        self.counter += 1;
        let new_cmd_name = format!("boop {}", self.counter);
        let counter = self.counter;
        ui.add_command(
            SimpleCommand::new(new_cmd_name, move |ui| {
                ui.show_message(format!("Boop {}!", counter))
            })
            .into_box(),
        );
        Ok(())
    }
}
