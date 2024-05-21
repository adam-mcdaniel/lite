use super::{Buffer, Direction};

#[derive(Clone, Debug)]
pub enum Change {
    Insert(String),
    Delete(String),
    Move((usize, usize), Direction, usize),
    Goto((usize, usize), (usize, usize)),
    Select,
    Unselect,
    Undo,
    Redo,
}

impl Change {
    pub fn move_cur(dir: Direction, buf: &Buffer, count: usize) -> Self {
        Self::Move((buf.cursor_row, buf.cursor_col), dir, count)
    }

    pub fn modifies_content(&self) -> bool {
        match self {
            Self::Insert(_) | Self::Delete(_) => true,
            _ => false,
        }
    }

    pub fn goto_cur(pos: (usize, usize), buf: &Buffer) -> Self {
        Self::Goto((buf.cursor_row, buf.cursor_col), pos)
    }

    pub fn delete(count: usize) -> Self {
        Self::Delete(" ".repeat(count))
    }

    pub fn apply(&self, buf: &mut Buffer) {
        match self {
            Self::Insert(text) => {
                for ch in text.chars() {
                    buf.insert(ch)
                }
                buf.undo_stack.push(self.clone());
            }

            Self::Delete(text) => {
                let mut deleted = String::new();
                for _ in text.chars() {
                    buf.move_cur(Direction::Left);
                    if let Some(ch) = buf.delete() {
                        deleted.push(ch);
                    }
                }
                buf.undo_stack
                    .push(Self::Delete(deleted.chars().rev().collect()));
            }

            Self::Move(_, dir, count) => {
                let old_pos = buf.cur_pos();
                for _ in 0..*count {
                    buf.move_cur(*dir);
                }
                if buf.cur_pos() == old_pos {
                    buf.undo_stack.push(Self::Move(old_pos, Direction::Nowhere, *count));
                } else {
                    buf.undo_stack.push(self.clone());
                }
            }
            Self::Goto((old_row, old_col), (new_row, new_col)) => {
                buf.cursor_row = *new_row;
                buf.cursor_col = *new_col;
                buf.fix_cursor();
                buf.undo_stack.push(Self::Goto(
                    (*old_row, *old_col),
                    (buf.cursor_row, buf.cursor_col),
                ));
            }

            Self::Select => {
                buf.select();
                buf.undo_stack.push(self.clone());
            }

            Self::Unselect => {
                buf.unselect();
                buf.undo_stack.push(self.clone());
            }

            Self::Undo => {
                if let Some(change) = buf.undo_stack.pop() {
                    change.undo(buf);
                    buf.redo_stack.push(change);
                }
            }
            Self::Redo => {
                if let Some(change) = buf.redo_stack.pop() {
                    change.apply(buf);
                }
            }
        }
    }

    fn undo(&self, buf: &mut Buffer) {
        match self {
            Self::Insert(text) => {
                for _ in text.chars() {
                    buf.move_cur(Direction::Left);
                    buf.delete();
                }
            }

            Self::Delete(text) => {
                for ch in text.chars() {
                    buf.insert(ch)
                }
            }

            Self::Move((old_row, old_col), _, _) => {
                buf.cursor_row = *old_row;
                buf.cursor_col = *old_col;
            }
            Self::Goto((old_row, old_col), _) => {
                buf.cursor_row = *old_row;
                buf.cursor_col = *old_col;
            }

            Self::Select => buf.unselect(),
            Self::Unselect => buf.select(),

            Self::Undo => Self::Redo.apply(buf),
            Self::Redo => Self::Undo.apply(buf),
        }
    }
}
