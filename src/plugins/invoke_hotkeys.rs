use crate::error::Error;
use crate::system::{get_enso_home_dir, press_key, press_modifier_key, KeyDirection, ModifierKey};
use crate::ui::{UserInterface, UserInterfacePlugin};

#[derive(Default)]
pub struct InvokeHotkeysPlugin;

impl UserInterfacePlugin for InvokeHotkeysPlugin {
    fn init(&mut self, ui: &mut UserInterface) -> Result<(), Error> {
        let _ = get_enso_home_dir()?;

        ui.add_simple_command("copy to clipboard", |_ui| {
            press_modifier_key(ModifierKey::Control, KeyDirection::Down)?;
            press_key('c', KeyDirection::Down)?;
            press_key('c', KeyDirection::Up)?;
            press_modifier_key(ModifierKey::Control, KeyDirection::Up)?;

            Ok(())
        });

        Ok(())
    }
}
