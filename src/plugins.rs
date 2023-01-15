use crate::ui::UserInterfacePlugin;

mod builtin;
mod insert_unicode_characters;
mod invoke_hotkeys;

pub fn get_all_plugins() -> Vec<Box<dyn UserInterfacePlugin>> {
    vec![
        Box::new(builtin::BuiltinPlugin::default()),
        Box::new(insert_unicode_characters::InsertUnicodeCharactersPlugin::default()),
        Box::new(invoke_hotkeys::InvokeHotkeysPlugin::default()),
    ]
}
