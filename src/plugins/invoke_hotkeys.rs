use crate::error::Error;
use crate::system::{get_enso_home_dir, press_key, GraphicKey, KeyDirection, VirtualKey};
use crate::ui::{UserInterface, UserInterfacePlugin};

#[derive(Default)]
pub struct InvokeHotkeysPlugin;

impl UserInterfacePlugin for InvokeHotkeysPlugin {
    fn init(&mut self, ui: &mut UserInterface) -> Result<(), Error> {
        let _ = get_enso_home_dir()?;

        ui.add_simple_command("copy to clipboard", |_ui| {
            let ctrl_key = VirtualKey::Control;
            let c_key = VirtualKey::Graphic(GraphicKey::new('c').unwrap());
            press_key(ctrl_key, KeyDirection::Down)?;
            press_key(c_key, KeyDirection::Down)?;
            press_key(c_key, KeyDirection::Up)?;
            press_key(ctrl_key, KeyDirection::Up)?;

            Ok(())
        });

        Ok(())
    }
}
