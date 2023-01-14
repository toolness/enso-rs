use crate::{command::SimpleCommand, insert_commands, ui::UserInterface};

pub fn install_default_commands(ui: &mut UserInterface) {
    ui.add_command(
        SimpleCommand::new("help", |ui| {
            ui.show_message("Sorry, still need to implement help!")
        })
        .into_box(),
    );

    ui.add_command(SimpleCommand::new("quit", |ui| ui.quit()).into_box());

    insert_commands::insert_commands(ui);
}
