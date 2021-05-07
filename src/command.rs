use super::error::Error;

pub trait CommandUI {
    fn quit(&mut self);
}

pub trait Command {
    fn name(&self) -> String;
    fn help(&self) -> String;
    fn execute(&mut self, ui: &mut dyn CommandUI) -> Result<(), Error>;
}

pub struct CommandRegistry {
    commands: Vec<Box<dyn Command>>,
}

impl CommandRegistry {
    pub fn new() -> CommandRegistry {
        CommandRegistry {
            commands: Vec::new(),
        }
    }

    pub fn register(&mut self, command: Box<dyn Command>) {
        self.commands.push(command);
    }
}
