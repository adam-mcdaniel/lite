use super::Editor;

pub trait Frontend {
    fn render(&mut self, editor: &Editor, flush: bool) -> Result<(), String>;
    fn wait_for_input(&mut self, editor: &Editor) -> Result<Input, String>;
    fn set_status(&mut self, status: &str) -> Result<(), String>;

    fn prompt(&mut self, text: &str) -> Result<String, String>;
    fn ask(&mut self, prompt: &str, yes: &str, no: &str) -> Result<bool, String>;
    fn choose(&mut self, prompt: &str, options: Vec<String>) -> Result<String, String>;
    fn get_num(&mut self, prompt: &str) -> Result<isize, String>;

    fn exit(&mut self);

    fn height(&self) -> usize;
    fn width(&self) -> usize;
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Input {
    Esc,
    Char(char),
    Home, End,
    Left,
    Right,
    Up,
    Down,
    Backspace,
    Delete,
    Enter,
    Tab,
    PageUp,
    PageDown,
    Control(Box<Self>),
    Shift(Box<Self>),
    Alt(Box<Self>),
}
