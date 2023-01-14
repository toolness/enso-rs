#[derive(Debug)]
pub enum HookEvent {
    Keypress(i32),
    QuasimodeStart,
    QuasimodeEnd,
}
