use super::error::Error;
use super::ui::UserInterface;
use dyn_clone::DynClone;

pub trait Command: DynClone {
    fn name(&self) -> String;
    fn execute(&mut self, ui: &mut UserInterface) -> Result<(), Error>;
}

dyn_clone::clone_trait_object!(Command);

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

    pub fn into_box(self) -> Box<Self> {
        Box::new(self)
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

#[test]
fn test_simple_command_works() {
    let cmd = SimpleCommand::new("hi", |ui| {
        ui.show_message("HALLO")?;
        Ok(())
    })
    .into_box();
    let _cmd2 = cmd.clone();
}
