use crate::{command::SimpleCommand, ui::UserInterface};

use super::insert_commands::insert_commands;

pub fn install_default_commands(ui: &mut UserInterface) {
    ui.add_command(
        SimpleCommand::new("help", |ui| {
            ui.show_message("Sorry, still need to implement help!")
        })
        .into_box(),
    );

    ui.add_command(SimpleCommand::new("quit", |ui| ui.quit()).into_box());

    insert_commands(ui);
}
