use crate::error::Error;
use crate::ui::UserInterface;

use super::insert_commands::insert_commands;

pub fn install_default_commands(ui: &mut UserInterface) {
    ui.add_simple_command("help", |ui| {
        ui.show_message("Sorry, still need to implement help!")
    });

    ui.add_simple_command("quit", |ui| ui.quit());

    insert_commands(ui);
}

pub fn refresh_default_commands(_ui: &mut UserInterface) -> Result<(), Error> {
    println!("TODO: REFRESH DEFAULT COMMANDS");
    Ok(())
}
