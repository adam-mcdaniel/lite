mod buffer;
pub use buffer::*;
mod change;
pub use change::*;
mod editor;
pub use editor::*;
mod frontend;
pub use frontend::*;
mod lang;
pub use lang::*;

extern crate pest;
#[macro_use]
extern crate pest_derive;


#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Nowhere,
}
