use crate::error::Error;
use crate::system::{get_enso_home_dir, open_in_explorer};
use crate::ui::{UserInterface, UserInterfacePlugin};

#[derive(Default)]
pub struct BuiltinPlugin;

impl UserInterfacePlugin for BuiltinPlugin {
    fn init(&mut self, ui: &mut UserInterface) -> Result<(), Error> {
        ui.add_simple_command("help", |ui| {
            ui.show_message("Sorry, still need to implement help!")
        });

        ui.add_simple_command("quit", |ui| ui.quit());

        ui.add_simple_command("open enso directory", |_ui| {
            open_in_explorer(&get_enso_home_dir()?)
        });

        Ok(())
    }
}
