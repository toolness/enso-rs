use crate::error::Error;
use crate::ui::{UserInterface, UserInterfacePlugin};

#[derive(Default)]
pub struct BuiltinPlugin;

impl UserInterfacePlugin for BuiltinPlugin {
    fn init(&mut self, ui: &mut UserInterface) -> Result<(), Error> {
        ui.add_simple_command("help", |ui| {
            ui.show_message("Sorry, still need to implement help!")
        });

        ui.add_simple_command("quit", |ui| ui.quit());

        Ok(())
    }
}
