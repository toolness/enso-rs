use super::cldr_annotations::CLDR_ANNOTATIONS;
use super::command::SimpleCommand;
use super::ui::UserInterface;

pub fn insert_commands(ui: &mut UserInterface) {
    for (ch, name) in &CLDR_ANNOTATIONS {
        let name = format!("insert {}", name);
        let cmd = SimpleCommand::new(name, move |ui| ui.type_char(ch));
        ui.add_command(cmd.into_box());
    }
}
