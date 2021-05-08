use super::error::Error;
use super::ui::UserInterface;
use dyn_clone::DynClone;

pub trait Command: DynClone {
    fn name(&self) -> String;
    fn execute(&mut self, ui: &mut UserInterface) -> Result<(), Error>;
}

#[derive(Clone)]
pub struct SimpleCommand<F: FnMut(&mut UserInterface) -> Result<(), Error> + Clone> {
    name_: String,
    execute_: F,
}

impl<F: FnMut(&mut UserInterface) -> Result<(), Error> + Clone> SimpleCommand<F> {
    pub fn new<T: Into<String>>(name: T, execute: F) -> Self {
        SimpleCommand {
            name_: name.into(),
            execute_: execute,
        }
    }
}

impl<F: FnMut(&mut UserInterface) -> Result<(), Error> + Clone> Command for SimpleCommand<F> {
    fn name(&self) -> String {
        self.name_.clone()
    }

    fn execute(&mut self, ui: &mut UserInterface) -> Result<(), Error> {
        (self.execute_)(ui)
    }
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

fn tada_command() -> Box<dyn Command> {
    Box::new(SimpleCommand::new("tada", |ui| {
        ui.type_char("🎉")?;
        Ok(())
    }))
}

#[test]
fn test_simple_command_works() {
    let cmd = SimpleCommand::new("hi", |ui| {
        ui.show_message("HALLO")?;
        Ok(())
    });
    let _cmd2 = cmd.clone();
}
