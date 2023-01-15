use crate::{
    command::SimpleCommand,
    error::Error,
    ui::{UserInterface, UserInterfacePlugin},
};

use super::cldr_annotations::CLDR_ANNOTATIONS;

#[derive(Default)]
pub struct InsertUnicodeCharactersPlugin;

impl UserInterfacePlugin for InsertUnicodeCharactersPlugin {
    fn init(&mut self, ui: &mut UserInterface) -> Result<(), Error> {
        for (ch, name) in &CLDR_ANNOTATIONS {
            let name = format!("insert {}", name);
            let cmd = SimpleCommand::new(name, move |ui| ui.type_char(ch));
            ui.add_command(cmd.into_box());
        }
        Ok(())
    }
}
