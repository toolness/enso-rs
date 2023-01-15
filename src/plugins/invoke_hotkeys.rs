use crate::error::Error;
use crate::system::{get_enso_home_dir, press_key, GraphicKey, KeyDirection, VirtualKey};
use crate::ui::{UserInterface, UserInterfacePlugin};

#[derive(Default)]
pub struct InvokeHotkeysPlugin;

struct HotkeyCombination {
    pub keys: Vec<VirtualKey>,
}

impl HotkeyCombination {
    pub fn press(&self) -> Result<(), Error> {
        for key in self.keys.iter() {
            press_key(*key, KeyDirection::Down)?;
        }
        for key in self.keys.iter().rev() {
            press_key(*key, KeyDirection::Up)?;
        }
        Ok(())
    }
}

impl UserInterfacePlugin for InvokeHotkeysPlugin {
    fn init(&mut self, ui: &mut UserInterface) -> Result<(), Error> {
        let _ = get_enso_home_dir()?;

        ui.add_simple_command("copy to clipboard", |_ui| {
            let hotkey = HotkeyCombination {
                keys: vec![
                    VirtualKey::Control,
                    VirtualKey::Graphic(GraphicKey::new('c').unwrap()),
                ],
            };
            hotkey.press()?;
            Ok(())
        });

        Ok(())
    }
}
