#[derive(Debug)]
pub enum Event {
    Keypress(i32),
    QuasimodeStart,
    QuasimodeEnd,
}
