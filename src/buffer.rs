use super::{Change, Direction};
use std::cmp::{max, min};

#[derive(Clone, Debug)]
pub struct Buffer {
    lines: Vec<String>,
    // screen_min_row: usize, // the starting row to be displayed
    // screen_rows: usize,    // the number of rows to be displayed
    pub cursor_col: usize,
    pub cursor_row: usize,

    select_row_col: Option<(usize, usize)>,

    pub undo_stack: Vec<Change>,
    pub redo_stack: Vec<Change>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            // screen_min_row: 0,
            // screen_rows: 24,
            cursor_col: 0,
            cursor_row: 0,
            select_row_col: None,
            undo_stack: vec![],
            redo_stack: vec![],
        }
    }
}

impl Buffer {
    // fn set_screen_rows(&mut self, rows: usize) {
    //     self.screen_rows = rows;
    // }

    // fn fix_screen(&mut self) {
    //     if self.cursor_row > self.screen_min_row + self.screen_rows {
    //         self.screen_min_row += self.cursor_row - (self.screen_min_row + self.screen_rows)
    //     } else if self.cursor_row < self.screen_min_row {
    //         self.screen_min_row -= self.screen_min_row - self.cursor_row;
    //     }
    // }

    pub fn get_lines(&self, min_row: usize, max_row: usize) -> &[String] {
        &self.lines[min_row..min(max_row, self.lines.len())]
    }

    pub fn cur_pos(&self) -> (usize, usize) {
        (self.cursor_row, self.cursor_col)
    }

    pub fn fix_cursor(&mut self) {
        self.cursor_row = min(self.lines.len() - 1, self.cursor_row);
        self.cursor_col = min(self.cur_line().len(), self.cursor_col);
    }

    pub fn select(&mut self) {
        self.select_row_col = Some((self.cursor_row, self.cursor_col))
    }

    pub fn unselect(&mut self) {
        self.select_row_col = None
    }

    pub fn cur_line(&self) -> &String {
        &self.lines[self.cursor_row]
    }
    pub fn cur_line_after(&self) -> &str {
        &self.lines[self.cursor_row][self.cursor_col..]
    }
    pub fn cur_line_before(&self) -> &str {
        &self.lines[self.cursor_row][self.cursor_col..]
    }

    pub fn selection_range(&self) -> Option<((usize, usize), (usize, usize))> {
        if let Some((selected_row, selected_col)) = self.select_row_col {
            if selected_row < self.cursor_row {
                Some((
                    (selected_row, selected_col),
                    (self.cursor_row, self.cursor_col),
                ))
            } else if selected_row == self.cursor_row && selected_col < self.cursor_col {
                Some((
                    (self.cursor_row, selected_col),
                    (selected_row, self.cursor_col),
                ))
            } else {
                Some((
                    (self.cursor_row, self.cursor_col),
                    (selected_row, selected_col),
                ))
            }
        } else {
            None
        }
    }

    pub fn selection_start(&self) -> Option<(usize, usize)> {
        self.selection_range().and_then(|(start, _)| Some(start))
    }

    pub fn selection_end(&self) -> Option<(usize, usize)> {
        self.selection_range().and_then(|(_, end)| Some(end))
    }

    pub fn selected_lines(&self) -> Option<&[String]> {
        if let Some((selected_row, _)) = self.select_row_col {
            if selected_row < self.cursor_row {
                Some(&self.lines[selected_row..=self.cursor_row])
            } else {
                Some(&self.lines[self.cursor_row..=selected_row])
            }
        } else {
            None
        }
    }
    pub fn selected_lines_mut(&mut self) -> Option<&mut [String]> {
        if let Some((selected_row, _)) = self.select_row_col {
            if selected_row < self.cursor_row {
                Some(&mut self.lines[selected_row..=self.cursor_row])
            } else {
                Some(&mut self.lines[self.cursor_row..=selected_row])
            }
        } else {
            None
        }
    }

    pub fn selected(&self) -> Option<String> {
        let lines = self.selected_lines()?;
        eprintln!("{:?}", lines);
        let last_line_len = lines.last()?.len();
        let mut lines: String = lines.join("\n");
        if let Some((selected_row, selected_col)) = self.select_row_col {
            if selected_row < self.cursor_row {
                lines = lines[selected_col..].to_owned();
                lines.truncate(lines.len() - (last_line_len - self.cursor_col));
                Some(lines)
            } else if selected_row > self.cursor_row {
                lines = lines[self.cursor_col..].to_owned();
                lines.truncate(lines.len() - (last_line_len - selected_col));
                Some(lines)
            } else if selected_col < self.cursor_col {
                lines = lines[selected_col..].to_owned();
                lines.truncate(lines.len() - (last_line_len - self.cursor_col));
                Some(lines)
            } else if selected_col > self.cursor_col {
                lines = lines[self.cursor_col..].to_owned();
                lines.truncate(lines.len() - (last_line_len - selected_col));
                Some(lines)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn cur_line_mut(&mut self) -> &mut String {
        &mut self.lines[self.cursor_row]
    }

    pub fn insert(&mut self, ch: char) {
        if ch == '\n' {
            let col = self.cursor_col;
            let line = self.cur_line_mut();
            let last_half = line.split_off(col);
            self.lines.insert(self.cursor_row + 1, last_half);
        } else {
            let col = self.cursor_col;
            self.cur_line_mut().insert(col, ch);
        }
        self.move_cur(Direction::Right);
    }

    pub fn delete(&mut self) -> Option<char> {
        let col = self.cursor_col;
        let lines_len = self.lines.len();
        let line = self.cur_line_mut();

        if lines_len == 1 && line.len() == 0 {
            None
        } else if col < line.len() {
            // If the cursor is before the end of the string,
            // remove the char after the cursor.
            Some(line.remove(col))
        } else {
            // Otherwise, join it with the next line
            drop(line);
            if self.cursor_row + 1 < self.lines.len() {
                let next = self.lines.remove(self.cursor_row + 1);
                self.lines[self.cursor_row] += &next;
                Some('\n')
            } else {
                None
            }
        }
    }

    pub fn move_cur(&mut self, dir: Direction) {
        // self.fix_screen()
        match dir {
            Direction::Up => {
                if self.cursor_row > 0 {
                    self.cursor_row -= 1;
                }
                self.cursor_col = min(self.cur_line().len(), self.cursor_col)
            }
            Direction::Down => {
                self.cursor_row = min(self.lines.len() - 1, self.cursor_row + 1);
                self.cursor_col = min(self.cur_line().len(), self.cursor_col)
            }
            Direction::Left => {
                if self.cursor_col == 0 {
                    let old_row = self.cursor_row;
                    self.move_cur(Direction::Up);
                    if self.cursor_row < old_row {
                        self.cursor_col = self.cur_line().len()
                    }
                } else {
                    self.cursor_col -= 1
                }
            }
            Direction::Right => {
                if self.cursor_col == self.cur_line().len() {
                    let old_row = self.cursor_row;
                    self.move_cur(Direction::Down);
                    if self.cursor_row > old_row {
                        self.cursor_col = 0
                    }
                } else {
                    self.cursor_col += 1
                }
            }
            Direction::Nowhere => {}
        }
        // self.fix_screen()
    }
}
