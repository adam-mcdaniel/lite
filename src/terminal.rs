use crate::*;

use crossterm::{
    cursor::MoveTo,
    event::{poll, read, Event, KeyCode, KeyModifiers},
    execute,
    style::{Color, Attribute, Print, ResetColor, SetBackgroundColor, SetForegroundColor, SetAttribute},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
};

use std::io::stdout;

pub struct Terminal {
    screen_start_row: usize, // the starting row to be displayed
    screen_cols: usize,      // the number of columns to be displayed
    screen_rows: usize,      // the number of rows to be displayed
    status: String,
    last_get_lines: Option<(usize, usize)>,
    last_buffer_id: usize,
}

impl Terminal {
    const STATUS_COLOR: Color = Color::DarkMagenta;

    const KEYWORDS: &'static [&'static str] = &[
        "use",
        "struct",
        "enum",
        "union",
        "class",
        "const",
        "static",
        "interface",
        "trait",
        "impl",
        "fn",
        "let",
        "if",
        "else",
        "while",
        "for",
        "in",
        "return",
        "break",
        "continue",
        "true",
        "false",
        "null",
        "NULL",
        "nil",
        "NIL",
        "and",
        "or",
        "not",
        "xor",
        "mod",
        "div",
        "as",
        "is",
        "new",
        "delete",
        "sizeof",
        "typeof",
    ];

    const BUILTINS: &'static [&'static str] = &[
        "printf",
        "scanf",
        "malloc",
        "free",
        "calloc",
        "realloc",
        "exit",
        "abort",
        "memcpy",
        "memset",
        "memmove",
        "memcmp",
        "strcmp",
        "strcpy",
        "strcat",
        "strlen",
        "strchr",
        "strstr",
        "strtok",
    ];

    const TYPES: &'static [&'static str] = &[
        "int",
        "float",
        "double",
        "char",
        "bool",
        "string",
        "void",
    ];

    const OPERATORS: &'static [&'static str] = &[
        "#include",
        "+",
        "-",
        "*",
        "/",
        "%",
        "(",
        ")",
        "{",
        "}",
        "()",
        "++",
        "--",
        "+=",
        "-=",
        "*=",
        "/=",
        "%=",
        "<<",
        ">>",
        "<",
        ">",
        "<=",
        ">=",
        "==",
        "!=",
        ",",
        "&",
    ];

    fn print_line_with_highlighting(&mut self, row: u16, line: &str, start: usize, _end: usize) {
        // Split the input into characters
        let words: Vec<_> = line.split_inclusive(char::is_whitespace).collect();

        execute!(stdout(), MoveTo(start as u16, row)).unwrap();
        for word in words {
            let trimmed = word.trim().to_lowercase();
            let mut color = Color::White;
            if Self::KEYWORDS.contains(&trimmed.as_str()) {
                // execute!(stdout(), SetForegroundColor(Color::Magenta)).unwrap();
                color = Color::Magenta;
            } else if Self::TYPES.contains(&trimmed.as_str()) {
                // execute!(stdout(), SetForegroundColor(Color::Blue)).unwrap();
                color = Color::Blue;
            } else if Self::OPERATORS.contains(&trimmed.as_str()) {
                // execute!(stdout(), SetForegroundColor(Color::Yellow)).unwrap();
                color = Color::Yellow;
            } else if Self::BUILTINS.contains(&trimmed.as_str()) {
                // execute!(stdout(), SetForegroundColor(Color::Green)).unwrap();
                color = Color::Green;
            } else {
                // execute!(stdout(), ResetColor).unwrap();
            }

            if !word.chars().all(char::is_alphanumeric) {
                for ch in word.chars() {
                    if Self::OPERATORS.contains(&ch.to_string().as_str()) {
                        execute!(stdout(), SetForegroundColor(Color::Yellow), Print(ch)).unwrap();
                    } else {
                        execute!(stdout(), SetForegroundColor(color), Print(ch)).unwrap();
                    }
                }
                execute!(stdout(), ResetColor).unwrap();
            } else {
                execute!(stdout(), SetForegroundColor(color), Print(word), ResetColor).unwrap();
            }

        }
    }
}

impl Default for Terminal {
    fn default() -> Self {
        enable_raw_mode().expect("Failed to enable raw mode");
        Self {
            screen_start_row: 0,
            screen_cols: 80,
            screen_rows: 23,
            status: String::new(),
            last_get_lines: None,
            last_buffer_id: 0,
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        disable_raw_mode().expect("Failed to disable raw mode");
    }
}

impl Frontend for Terminal {
    fn height(&self) -> usize {
        self.screen_rows
    }

    fn width(&self) -> usize {
        self.screen_cols
    }

    fn exit(&mut self) {
        execute!(stdout(), MoveTo(0, 0), Clear(ClearType::All)).unwrap();
    }

    fn render(&mut self, editor: &Editor, flush: bool) -> Result<(), String> {
        (self.screen_cols, self.screen_rows) = match size() {
            Ok((cols, rows)) => (cols as usize, rows as usize - 1),
            Err(_) => (80, 23),
        };

        if flush {
            execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).unwrap();
        }
        
        if let Some(buf) = editor.cur_buf() {
            let (cur_row, cur_col) = buf.cur_pos();
            while cur_row >= self.screen_rows + self.screen_start_row {
                self.screen_start_row += 1
            }

            while cur_row < self.screen_start_row {
                self.screen_start_row -= 1
            }

            let width = (self.screen_start_row + self.screen_rows).to_string().len();

            if let Some((start_select, end_select)) = buf.selection_range() {
                let (start_row, start_col) = start_select;
                let (end_row, end_col) = end_select;

                for (i, line) in buf
                    .get_lines(
                        self.screen_start_row,
                        self.screen_start_row + self.screen_rows,
                    )
                    .iter()
                    .enumerate()
                {
                    let max_len = std::cmp::min(line.len(), self.screen_cols - width - 1);
                    // execute!(
                    //     stdout(),
                    //     MoveTo(0, i as u16),
                    //     Print(format!(
                    //         "{:<width$?} {}",
                    //         self.screen_start_row + i + 1,
                    //         &line[..max_len]
                    //     ))
                    // )
                    // .unwrap();
                    self.print_line_with_highlighting(i as u16, &
                        &format!("{:<width$?} {}", self.screen_start_row + i + 1, &line[..max_len]), 0, max_len + width + 1
                    );


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
                        execute!(
                            stdout(),
                            SetBackgroundColor(Color::White),
                            SetForegroundColor(Color::Black),
                            MoveTo(start as u16, i as u16),
                            Print(if line.is_empty() {
                                " "
                            } else {
                                &line[..max_len]
                            }),
                            ResetColor
                        )
                        .unwrap();
                    }
                }
            } else if !flush && self.last_buffer_id != editor.cur_buf_id() || self.last_get_lines != Some((self.screen_start_row, self.screen_start_row + self.screen_rows - 1)) {
                execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).unwrap();
                self.last_get_lines = Some((self.screen_start_row, self.screen_start_row + self.screen_rows));
                for (i, line) in buf
                    .get_lines(
                        self.screen_start_row,
                        self.screen_start_row + self.screen_rows,
                    )
                    .iter()
                    .enumerate()
                {
                    let max_len = std::cmp::min(line.len(), self.screen_cols - width - 1);
                    // execute!(
                    //     stdout(),
                    //     MoveTo(0, i as u16),
                    //     Print(format!(
                    //         "{:<width$?} {}",
                    //         self.screen_start_row + i + 1,
                    //         &line[..max_len]
                    //     ))
                    // )
                    // .unwrap();
                    self.print_line_with_highlighting(i as u16, &
                        &format!("{:<width$?} {}", self.screen_start_row + i + 1, &line[..max_len]), 0, max_len + width + 1
                    );

                }
            }
            
            // Draw over the status line
            execute!(stdout(), SetAttribute(Attribute::Italic), MoveTo(0, self.screen_rows as u16), SetBackgroundColor(Self::STATUS_COLOR), Clear(ClearType::CurrentLine))
                .unwrap();
            execute!(stdout(), MoveTo(0, self.screen_rows as u16), Print(&self.status), ResetColor).unwrap();

            execute!(
                stdout(),
                MoveTo(
                    (cur_col + width + 1) as u16,
                    (cur_row - self.screen_start_row) as u16
                )
            )
            .unwrap();
        }
        Ok(())
    }
    fn set_status(&mut self, status: &str) -> Result<(), String> {
        self.status = status.to_string();
        Ok(())
    }
    fn wait_for_input(&mut self, editor: &Editor) -> Result<Input, String> {
        loop {
            self.render(editor, false)?;
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
                                KeyCode::Esc => Input::Esc,
                                KeyCode::Home => Input::Home,
                                KeyCode::End => Input::End,
                                KeyCode::PageUp => Input::PageUp,
                                KeyCode::PageDown => Input::PageDown,
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
                            return Ok(result);
                        }
                        Event::Mouse(_) => {}
                        Event::Resize(cols, rows) => {
                            self.screen_cols = cols as usize;
                            self.screen_rows = rows as usize;
                            self.render(editor, true)?;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn prompt(&mut self, text: &str) -> Result<String, String> {
        // Go to the status line and ask the question
        execute!(stdout(), MoveTo(0, self.screen_rows as u16), SetBackgroundColor(Self::STATUS_COLOR), Clear(ClearType::CurrentLine))
            .unwrap();
        execute!(stdout(), MoveTo(0, self.screen_rows as u16), Print(text), ResetColor).unwrap();
        let mut input = String::new();
        loop {
            if poll(std::time::Duration::from_millis(1_000)).is_ok() {
                if let Ok(event) = read() {
                    match event {
                        Event::Key(key_event) => {
                            match key_event.code {
                                KeyCode::Char(ch) => {
                                    input.push(ch);
                                    // Draw the input
                                    execute!(
                                        stdout(),
                                        MoveTo(0, self.screen_rows as u16),
                                        SetBackgroundColor(Self::STATUS_COLOR), 
                                        Clear(ClearType::CurrentLine),
                                        Print(text),
                                        Print(&input),
                                        ResetColor
                                    ).unwrap();
                                }
                                KeyCode::Enter => {
                                    return Ok(input);
                                }
                                KeyCode::Esc => {
                                    return Err("User cancelled".to_string());
                                }
                                KeyCode::Backspace => {
                                    input.pop();
                                    execute!(
                                        stdout(),
                                        MoveTo(0, self.screen_rows as u16),
                                        SetBackgroundColor(Self::STATUS_COLOR), 
                                        Clear(ClearType::CurrentLine),
                                        Print(text),
                                        Print(&input),
                                        ResetColor
                                    ).unwrap();
                                }
                                _ => continue,
                            }
                        }
                        _ => continue,
                    }
                }
            }
        }
        // Ok(String::new())
    }
    fn ask(&mut self, prompt: &str, yes: &str, no: &str) -> Result<bool, String> {
        // Ok(false)
        // Go to the status line and ask the question
        execute!(stdout(), MoveTo(0, self.screen_rows as u16), SetBackgroundColor(Self::STATUS_COLOR), Clear(ClearType::CurrentLine))
            .unwrap();
        execute!(stdout(), MoveTo(0, self.screen_rows as u16), Print(prompt), Print(format!(" {} / {}", yes, no)), ResetColor).unwrap();
        loop {
            if poll(std::time::Duration::from_millis(1_000)).is_ok() {
                if let Ok(event) = read() {
                    match event {
                        Event::Key(key_event) => {
                            match key_event.code {
                                KeyCode::Char('y') => return Ok(true),
                                KeyCode::Char('n') => return Ok(false),
                                _ => continue,
                            }
                        }
                        _ => continue,
                    }
                }
            }
        }
    }
    fn choose(&mut self, prompt: &str, options: Vec<String>) -> Result<String, String> {
        // Ok(options.into_iter().next().unwrap())
        // Go to the status line and ask the question
        execute!(stdout(), MoveTo(0, self.screen_rows as u16), SetBackgroundColor(Self::STATUS_COLOR), Clear(ClearType::CurrentLine))
            .unwrap();
        execute!(stdout(), MoveTo(0, self.screen_rows as u16 - 1 - options.len() as u16), Print(prompt), ResetColor).unwrap();
        for (i, option) in options.iter().enumerate() {
            execute!(
                stdout(),
                MoveTo(0, self.screen_rows as u16 - options.len() as u16 + i as u16 - 1),
                Print(format!("{}: {}", i + 1, option))
            )
            .unwrap();
        }
        loop {
            if poll(std::time::Duration::from_millis(1_000)).is_ok() {
                if let Ok(event) = read() {
                    match event {
                        Event::Key(key_event) => {
                            if let KeyCode::Char(ch) = key_event.code {
                                if let Some(index) = ch.to_digit(10) {
                                    if index as usize <= options.len() {
                                        return Ok(options[index as usize - 1].clone());
                                    }
                                }
                            }
                        }
                        _ => continue,
                    }
                }
            }
        }
    }
    fn get_num(&mut self, prompt: &str) -> Result<isize, String> {
        // Ok(0)
        // Go to the status line and ask the question
        execute!(stdout(), MoveTo(0, self.screen_rows as u16), SetBackgroundColor(Self::STATUS_COLOR), Clear(ClearType::CurrentLine))
            .unwrap();
        execute!(stdout(), MoveTo(0, self.screen_rows as u16), Print(prompt), ResetColor).unwrap();
        let mut num = String::new();
        loop {
            if poll(std::time::Duration::from_millis(1_000)).is_ok() {
                if let Ok(event) = read() {
                    match event {
                        Event::Key(key_event) => {
                            match key_event.code {
                                KeyCode::Char(ch) => {
                                    if let Some(_digit) = ch.to_digit(10) {
                                        num.push(ch);
                                    }
                                }
                                KeyCode::Enter => {
                                    if let Ok(num) = num.parse() {
                                        return Ok(num);
                                    }
                                }
                                KeyCode::Esc => {
                                    return Err("User cancelled".to_string());
                                }
                                KeyCode::Backspace => {
                                    num.pop();
                                }
                                _ => continue,
                            }
                        }
                        _ => continue,
                    }
                }
            }
        }
    }
}
