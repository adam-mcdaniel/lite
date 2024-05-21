use super::{eval, Buffer, Change, Direction, Env, Expr};
use std::{cmp::min, fmt};

pub struct Editor {
    buffers: Vec<Buffer>,
    current_buffer_index: usize,
    pub env: Env,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffers: vec![Buffer::default()],
            current_buffer_index: 0,
            env: Env::default(),
        }
    }

    pub fn cur_buf_id(&self) -> usize {
        self.current_buffer_index
    }

    pub fn from_file_name(file: String) -> Self {
        let mut editor = Self::new();
        editor.buffers[0] = Buffer::from_file_name(file);
        editor
    }

    pub fn new_buf(&mut self) {
        self.buffers.push(Buffer::default());
    }

    pub fn add_buf(&mut self, buf: Buffer) {
        self.buffers.push(buf);
    }

    pub fn quit_buf(&mut self, save: bool) {
        if save {
            let buf = self.cur_buf_mut().unwrap();
            if let Some(file_name) = buf.get_file_name().map(String::from) {
                buf.save(&file_name).unwrap();
            }
        }
        self.buffers.remove(self.cur_buf_id());
        if self.buffers.is_empty() {
            self.new_buf();
        }
        self.set_buf(min(self.buffers.len() - 1, self.cur_buf_id() - 1));
    }

    pub fn max_buf_id(&self) -> usize {
        self.buffers.len() - 1
    }

    pub fn set_buf(&mut self, id: usize) {
        if id < self.buffers.len() {
            self.current_buffer_index = id;
        }
    }

    fn apply(&mut self, change: Change) {
        if let Some(buf) = self.cur_buf_mut() {
            change.apply(buf)
        }
    }

    fn clear_redo_stack(&mut self) {
        if let Some(buf) = self.cur_buf_mut() {
            buf.redo_stack.clear();
        }
    }

    pub fn get_selected(&self) -> Option<String> {
        self.cur_buf().and_then(|buf| buf.selected())
    }

    pub fn selection_range(&self) -> Option<((usize, usize), (usize, usize))> {
        self.cur_buf().and_then(|buf| buf.selection_range())
    }

    pub fn selection_start(&self) -> Option<(usize, usize)> {
        self.cur_buf().and_then(|buf| buf.selection_start())
    }

    pub fn selection_end(&self) -> Option<(usize, usize)> {
        self.cur_buf().and_then(|buf| buf.selection_end())
    }

    pub fn get_selected_lines(&self) -> Option<&[String]> {
        self.cur_buf().and_then(|buf| buf.selected_lines())
    }

    pub fn select(&mut self) {
        if let Some(buf) = self.cur_buf() {
            if buf.selection_start().is_none() {
                self.apply(Change::Select)
            }
        }
    }

    pub fn unselect(&mut self) {
        if let Some(buf) = self.cur_buf() {
            if buf.selection_start().is_some() {
                self.apply(Change::Unselect)
            }
        }
    }

    pub fn is_selected(&self) -> bool {
        self.cur_buf().map(|buf| buf.selection_start().is_some()).unwrap_or(false)
    }

    pub fn insert(&mut self, text: impl ToString) {
        self.apply(Change::Insert(text.to_string()));
        self.clear_redo_stack();
    }

    pub fn delete(&mut self, count: usize) {
        self.apply(Change::delete(count));
        self.clear_redo_stack();
    }

    pub fn move_cur(&mut self, dir: Direction) {
        if let Some(buf) = self.cur_buf() {
            let change = Change::move_cur(dir, buf, 1);
            self.apply(change)
        }
    }

    pub fn move_cur_by(&mut self, dir: Direction, count: usize) {
        if let Some(buf) = self.cur_buf() {
            let change = Change::move_cur(dir, buf, count);
            self.apply(change)
        }
    }

    pub fn goto_cur(&mut self, pos: (usize, usize)) {
        if let Some(buf) = self.cur_buf() {
            let change = Change::goto_cur(pos, buf);
            self.apply(change)
        }
    }

    pub fn undo(&mut self) {
        self.apply(Change::Undo)
    }

    pub fn redo(&mut self) {
        self.apply(Change::Redo)
    }

    pub fn cur_buf(&self) -> Option<&Buffer> {
        if self.buffers.is_empty() {
            None
        } else {
            Some(&self.buffers[min(self.buffers.len() - 1, self.current_buffer_index)])
        }
    }

    pub fn cur_buf_mut(&mut self) -> Option<&mut Buffer> {
        if self.buffers.is_empty() {
            None
        } else {
            self.current_buffer_index = min(self.buffers.len() - 1, self.current_buffer_index);
            Some(&mut self.buffers[self.current_buffer_index])
        }
    }

    pub fn eval(&mut self, expr: Expr) -> Result<Expr, Expr> {
        let mut env = self.env.clone();
        let result = eval(expr, self, &mut env);
        self.env = env;
        result
    }

    pub fn next_buf(&mut self) {
        if self.buffers.is_empty() {
            return;
        }
        self.set_buf((self.current_buffer_index + 1) % self.buffers.len());
    }

    pub fn prev_buf(&mut self) {
        if self.buffers.is_empty() {
            return;
        }
        if self.current_buffer_index > 0 {
            self.current_buffer_index -= 1;
        } else {
            self.current_buffer_index = self.buffers.len() - 1;
        }
    }
}

impl fmt::Debug for Editor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.buffers.is_empty() {
            write!(
                f,
                "{:?} {:?}",
                self.buffers, self.buffers[self.current_buffer_index]
            )
        } else {
            Ok(())
        }
    }
}
