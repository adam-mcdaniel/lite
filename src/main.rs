use blackjack::{parse, eval, err, Direction, undo, redo, move_cursor, goto_cursor, insert, delete, get_selection_start, get_selection_end, get_selection_len, get_undo_stack_len, select, unselect, get_selected, get_selected_lines, Builtin, Input, Frontend, Editor, Buffer, Change, Expr};

use crossterm::{
    cursor::position,
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    style::{Print, SetBackgroundColor, Color, ResetColor},
    cursor::MoveTo,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};

use std::io::stdout;

struct Terminal {
    screen_start_row: usize, // the starting row to be displayed
    screen_cols: usize,      // the number of columns to be displayed
    screen_rows: usize,      // the number of rows to be displayed
}

impl Default for Terminal {
    fn default() -> Self {
        enable_raw_mode();
        Self {
            screen_start_row: 0,
            screen_cols: 80,
            screen_rows: 24,
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        disable_raw_mode();
    }
}

impl Frontend for Terminal {
    fn render(&mut self, editor: &Editor) -> Result<(), String> {
        if let Some(buf) = editor.cur_buf() {
            let (cur_row, cur_col) = buf.cur_pos();
            while cur_row >= self.screen_rows + self.screen_start_row {
                self.screen_start_row += 1
            }

            while cur_row < self.screen_start_row {
                self.screen_start_row -= 1
            }
            execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).unwrap();

            let width = (self.screen_start_row + self.screen_rows).to_string().len();

            if let Some((start_select, end_select)) = buf.selection_range() {
                let (start_row, start_col) = start_select;
                let (end_row, end_col) = end_select;
                for (i, line) in buf.get_lines(self.screen_start_row, self.screen_start_row + self.screen_rows).iter().enumerate() {
                    let max_len = std::cmp::min(line.len(), self.screen_cols - width - 1);
                    execute!(stdout(), MoveTo(0, i as u16), Print(format!("{:<width$?} {}", self.screen_start_row + i + 1, &line[..max_len]))).unwrap();

                    let row = i + self.screen_start_row;
                    if start_row <= row && row <= end_row {
                        let mut line = line.clone();
                        let mut start = width + 1;
                        if end_row == row {
                            line = line[..end_col].to_string();
                        }
                        if start_row == row {
                            start += start_col;
                            line = line[start_col..].to_string();
                        }
                        let max_len = std::cmp::min(line.len(), self.screen_cols - width - 1);
                        execute!(stdout(), SetBackgroundColor(Color::White), MoveTo(start as u16, i as u16), Print(if line.is_empty() { " " } else { &line[..max_len] }), ResetColor).unwrap();
                    }
                }
            } else {
                for (i, line) in buf.get_lines(self.screen_start_row, self.screen_start_row + self.screen_rows).iter().enumerate() {
                    let max_len = std::cmp::min(line.len(), self.screen_cols - width - 1);
                    execute!(stdout(), MoveTo(0, i as u16), Print(format!("{:<width$?} {}", self.screen_start_row + i + 1, &line[..max_len]))).unwrap();
                }
            }

            execute!(stdout(), MoveTo((cur_col + width + 1) as u16, (cur_row - self.screen_start_row) as u16)).unwrap();
        }

        Ok(())
    }
    fn set_status(&mut self, status: &str) -> Result<(), String> {
        Ok(())
    }
    fn wait_for_input(&mut self, editor: &Editor) -> Result<Input, String> {
        loop {
            // Wait up to 1s for another event
            if poll(std::time::Duration::from_millis(1_000)).is_ok() {
                // It's guaranteed that read() won't block if `poll` returns `Ok(true)`
                if let Ok(event) = read() {
                    match event {
                        Event::Key(key_event) => {
                            let mut result = match key_event.code {
                                KeyCode::Backspace => Input::Backspace,
                                KeyCode::Delete => Input::Delete,
                                KeyCode::Left => Input::Left,
                                KeyCode::Right => Input::Right,
                                KeyCode::Up => Input::Up,
                                KeyCode::Down => Input::Down,
                                KeyCode::Enter => Input::Enter,
                                KeyCode::Tab => Input::Tab,
        
                                KeyCode::Char(ch) => Input::Char(ch),
                                _ => continue,
                            };
                            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                                result = Input::Shift(Box::new(result));
                            }
                            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                                result = Input::Control(Box::new(result));
                            }
                            if key_event.modifiers.contains(KeyModifiers::ALT) {
                                result = Input::Alt(Box::new(result));
                            }
                            return Ok(result)
                        }
                        Event::Mouse(_) => {}
                        Event::Resize(cols, rows) => {
                            self.screen_cols = cols as usize;
                            self.screen_rows = rows as usize;
                            self.render(editor)?;
                        }
                    }
                }
            }
        }
    }

    fn prompt(&mut self, text: &str) -> Result<String, String> {
        Ok(String::new())
    }
    fn ask(&mut self, prompt: &str, yes: &str, no: &str) -> Result<bool, String> {
        Ok(false)
    }
    fn choose(&mut self, prompt: &str, options: Vec<String>) -> Result<String, String> {
        Ok(options.into_iter().next().unwrap())
    }
    fn get_num(&mut self, prompt: &str) -> Result<isize, String> {
        Ok(0)
    }
}

fn main() -> Result<(), Expr> {
    let mut editor = Editor::new();
    editor.env.scope.insert(Expr::Symbol(String::from("insert")), Expr::Builtin(Builtin::new("insert", "insert some text", "insert some text into the buffer", insert)));
    editor.env.scope.insert(Expr::Symbol(String::from("delete")), Expr::Builtin(Builtin::new("delete", "delete some text", "delete some text form the buffer", delete)));
    editor.env.scope.insert(Expr::Symbol(String::from("move")), Expr::Builtin(Builtin::new("move", "move the cursor", "move the cursor in the current buffer", move_cursor)));
    editor.env.scope.insert(Expr::Symbol(String::from("goto")), Expr::Builtin(Builtin::new("goto", "move the cursor", "move the cursor in the current buffer", goto_cursor)));
    editor.env.scope.insert(Expr::Symbol(String::from("get-select-start")), Expr::Builtin(Builtin::new("select-start", "move the cursor", "move the cursor in the current buffer", get_selection_start)));
    editor.env.scope.insert(Expr::Symbol(String::from("get-select-end")), Expr::Builtin(Builtin::new("select-end", "move the cursor", "move the cursor in the current buffer", get_selection_end)));
    editor.env.scope.insert(Expr::Symbol(String::from("get-select")), Expr::Builtin(Builtin::new("get-select", "move the cursor", "move the cursor in the current buffer", get_selected)));
    editor.env.scope.insert(Expr::Symbol(String::from("get-select-lines")), Expr::Builtin(Builtin::new("get-select-lines", "move the cursor", "move the cursor in the current buffer", get_selected_lines)));
    editor.env.scope.insert(Expr::Symbol(String::from("get-select-len")), Expr::Builtin(Builtin::new("get-select-len", "move the cursor", "move the cursor in the current buffer", get_selection_len)));
    editor.env.scope.insert(Expr::Symbol(String::from("get-undo-stack-len")), Expr::Builtin(Builtin::new("get-undo-stack-len", "move the cursor", "move the cursor in the current buffer", get_undo_stack_len)));
    editor.env.scope.insert(Expr::Symbol(String::from("select")), Expr::Builtin(Builtin::new("select", "move the cursor", "move the cursor in the current buffer", select)));
    editor.env.scope.insert(Expr::Symbol(String::from("unselect")), Expr::Builtin(Builtin::new("unselect", "move the cursor", "move the cursor in the current buffer", unselect)));
    editor.env.scope.insert(Expr::Symbol(String::from("undo")), Expr::Builtin(Builtin::new("undo", "move the cursor", "move the cursor in the current buffer", undo)));
    editor.env.scope.insert(Expr::Symbol(String::from("redo")), Expr::Builtin(Builtin::new("redo", "move the cursor", "move the cursor in the current buffer", redo)));
    editor.env.scope.insert(Expr::Symbol(String::from("add")), Expr::Builtin(Builtin::new("add", "adds two things", "adds two things together", |args, editor, env| {
        eval(Expr::Add(Box::new(args[0].clone()), Box::new(args[1].clone())), editor, env)
    })));
    editor.env.scope.insert(Expr::Symbol(String::from("sub")), Expr::Builtin(Builtin::new("sub", "subtracts two things", "subtracts two things together", |args, editor, env| {
        eval(Expr::Sub(Box::new(args[0].clone()), Box::new(args[1].clone())), editor, env)
    })));
    editor.env.scope.insert(Expr::Symbol(String::from("mul")), Expr::Builtin(Builtin::new("mul", "multiplies two things", "multiplies two things together", |args, editor, env| {
        eval(Expr::Mul(Box::new(args[0].clone()), Box::new(args[1].clone())), editor, env)
    })));
    editor.env.scope.insert(Expr::Symbol(String::from("div")), Expr::Builtin(Builtin::new("div", "divides two things", "divides two things together", |args, editor, env| {
        eval(Expr::Div(Box::new(args[0].clone()), Box::new(args[1].clone())), editor, env)
    })));
    editor.env.scope.insert(Expr::Symbol(String::from("rem")), Expr::Builtin(Builtin::new("rem", "remainders two things", "remainders two things together", |args, editor, env| {
        eval(Expr::Rem(Box::new(args[0].clone()), Box::new(args[1].clone())), editor, env)
    })));
    // println!("{:?}", editor.eval(parse(r#"{
    //     insert "a";
    //     insert "b";
    //     insert "c";
    //     insert "d";
    //     insert "e";
    //     move "left";
    //     move "left";
    //     delete 3;
    //     let n = get-undo-stack-len ();
    //     insert "xyz";

    //     undo (sub (get-undo-stack-len ()) n)
    // }"#)?));

    let mut frontend = Terminal::default();
    let mut selected = false;
    let mut copied = String::new();
    loop {
        frontend.render(&editor);

        let input = frontend.wait_for_input(&editor);
        if Ok(Input::Control(Box::new(Input::Char('q')))) == input {
            break;
        } else {
            match input {
                Ok(Input::Shift(shift)) => {
                    match *shift {
                        Input::Char(ch) => {
                            selected = false;
                            editor.unselect();
                            editor.insert(ch.to_uppercase())
                        },
                        Input::Left | Input::Right | Input::Up | Input::Down => {
                            if !selected {
                                editor.select();
                                selected = true;
                            }
                            match *shift {
                                Input::Left => editor.move_cur(Direction::Left),
                                Input::Right => editor.move_cur(Direction::Right),
                                Input::Up => editor.move_cur(Direction::Up),
                                Input::Down => editor.move_cur(Direction::Down),
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Input::Char(ch)) => {
                    selected = false;
                    editor.unselect();
                    editor.insert(ch);
                }

                Ok(Input::Control(ctrl)) => {
                    if *ctrl == Input::Char('z') {
                        editor.undo();
                    } else if *ctrl == Input::Shift(Box::new(Input::Char('z'))) {
                        editor.redo();
                    } else if *ctrl == Input::Char('y') {
                        editor.redo();
                    } else if *ctrl == Input::Char('c') {
                        if let Some(selected) = editor.get_selected() {
                            copied = selected.clone();
                        }
                    } else if *ctrl == Input::Char('v') {
                        editor.insert(&copied);
                    }
                }

                Ok(Input::Enter) => {
                    selected = false;
                    editor.unselect();
                    editor.insert('\n');
                }

                Ok(Input::Tab) => {
                    selected = false;
                    editor.unselect();
                    editor.insert("    ");
                }

                Ok(Input::Backspace) => {
                    selected = false;
                    editor.unselect();
                    editor.delete(1);
                }

                Ok(Input::Left) => {
                    selected = false;
                    editor.unselect();
                    editor.move_cur(Direction::Left);
                }

                Ok(Input::Right) => {
                    selected = false;
                    editor.unselect();
                    editor.move_cur(Direction::Right);
                }

                Ok(Input::Up) => {
                    selected = false;
                    editor.unselect();
                    editor.move_cur(Direction::Up);
                }
                
                Ok(Input::Down) => {
                    selected = false;
                    editor.unselect();
                    editor.move_cur(Direction::Down);
                }

                _ => {}
            }
        }
        // editor.eval()
    }
    println!("{:?}", editor);
    
    Ok(())

    // let mut buf = Buffer::default();

    // Change::Insert("Hello world!\n".to_string()).apply(&mut buf);
    // Change::move_cur(Direction::Left, &buf).apply(&mut buf);
    // Change::Insert(" testing".to_string()).apply(&mut buf);
    // Change::move_cur(Direction::Down, &buf).apply(&mut buf);

    // Change::Undo.apply(&mut buf);
    // Change::Undo.apply(&mut buf);
    // Change::Undo.apply(&mut buf);
    // Change::Undo.apply(&mut buf);
    // // Change::Redo.apply(&mut buf);
    // // Change::Undo.apply(&mut buf);
    // // Change::Redo.apply(&mut buf);
    // // Change::Redo.apply(&mut buf);
    // // Change::Redo.apply(&mut buf);

    // println!("{:#?}", buf);
    // println!("selected: {:?}", buf.selected());

    // println!("{:?}", f64::from(Float::from(1232.1221394873219847334213)) + f64::from(Float::from(1234.231234321)));
    // println!("{:?}", 1232.1221394873219847334213 + 1234.231234321)
}
