use std::convert::TryFrom;

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

impl TryFrom<&str> for HotkeyCombination {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut result: Vec<VirtualKey> = Vec::new();
        let keys = s.split('+');
        for key in keys {
            let vkey = VirtualKey::try_from(key)?;
            result.push(vkey);
        }
        Ok(HotkeyCombination { keys: result })
    }
}

impl UserInterfacePlugin for InvokeHotkeysPlugin {
    fn init(&mut self, ui: &mut UserInterface) -> Result<(), Error> {
        let _ = get_enso_home_dir()?;

        ui.add_simple_command("copy to clipboard", |_ui| {
            let hotkey = HotkeyCombination::try_from("ctrl+c")?;
            hotkey.press()?;
            Ok(())
        });

        Ok(())
    }
}
