use blackjack::{
    delete,  eval, get_selected, get_selected_lines, get_selection_end, get_selection_len,
    get_selection_start, get_undo_stack_len, goto_cursor, insert, move_cursor, redo, select,
    undo, unselect, Buffer, Builtin, Direction, Editor, Expr, Frontend, Input, Terminal
};

fn main() -> Result<(), Expr> {
    // let mut editor = Editor::new();
    // Check if there's a file to open
    let args: Vec<String> = std::env::args().collect();
    let mut editor = if args.len() > 1 {
        Editor::from_file_name(args[1].clone())
    } else {
        Editor::new()
    };

    
    editor.env.scope.insert(
        Expr::Symbol(String::from("insert")),
        Expr::Builtin(Builtin::new(
            "insert",
            "insert some text",
            "insert some text into the buffer",
            insert,
        )),
    );
    editor.env.scope.insert(
        Expr::Symbol(String::from("delete")),
        Expr::Builtin(Builtin::new(
            "delete",
            "delete some text",
            "delete some text form the buffer",
            delete,
        )),
    );
    editor.env.scope.insert(
        Expr::Symbol(String::from("move")),
        Expr::Builtin(Builtin::new(
            "move",
            "move the cursor",
            "move the cursor in the current buffer",
            move_cursor,
        )),
    );
    editor.env.scope.insert(
        Expr::Symbol(String::from("goto")),
        Expr::Builtin(Builtin::new(
            "goto",
            "move the cursor",
            "move the cursor in the current buffer",
            goto_cursor,
        )),
    );
    editor.env.scope.insert(
        Expr::Symbol(String::from("get-select-start")),
        Expr::Builtin(Builtin::new(
            "select-start",
            "move the cursor",
            "move the cursor in the current buffer",
            get_selection_start,
        )),
    );
    editor.env.scope.insert(
        Expr::Symbol(String::from("get-select-end")),
        Expr::Builtin(Builtin::new(
            "select-end",
            "move the cursor",
            "move the cursor in the current buffer",
            get_selection_end,
        )),
    );
    editor.env.scope.insert(
        Expr::Symbol(String::from("get-select")),
        Expr::Builtin(Builtin::new(
            "get-select",
            "move the cursor",
            "move the cursor in the current buffer",
            get_selected,
        )),
    );
    editor.env.scope.insert(
        Expr::Symbol(String::from("get-select-lines")),
        Expr::Builtin(Builtin::new(
            "get-select-lines",
            "move the cursor",
            "move the cursor in the current buffer",
            get_selected_lines,
        )),
    );
    editor.env.scope.insert(
        Expr::Symbol(String::from("get-select-len")),
        Expr::Builtin(Builtin::new(
            "get-select-len",
            "move the cursor",
            "move the cursor in the current buffer",
            get_selection_len,
        )),
    );
    editor.env.scope.insert(
        Expr::Symbol(String::from("get-undo-stack-len")),
        Expr::Builtin(Builtin::new(
            "get-undo-stack-len",
            "move the cursor",
            "move the cursor in the current buffer",
            get_undo_stack_len,
        )),
    );
    editor.env.scope.insert(
        Expr::Symbol(String::from("select")),
        Expr::Builtin(Builtin::new(
            "select",
            "move the cursor",
            "move the cursor in the current buffer",
            select,
        )),
    );
    editor.env.scope.insert(
        Expr::Symbol(String::from("unselect")),
        Expr::Builtin(Builtin::new(
            "unselect",
            "move the cursor",
            "move the cursor in the current buffer",
            unselect,
        )),
    );
    editor.env.scope.insert(
        Expr::Symbol(String::from("undo")),
        Expr::Builtin(Builtin::new(
            "undo",
            "move the cursor",
            "move the cursor in the current buffer",
            undo,
        )),
    );
    editor.env.scope.insert(
        Expr::Symbol(String::from("redo")),
        Expr::Builtin(Builtin::new(
            "redo",
            "move the cursor",
            "move the cursor in the current buffer",
            redo,
        )),
    );
    editor.env.scope.insert(
        Expr::Symbol(String::from("add")),
        Expr::Builtin(Builtin::new(
            "add",
            "adds two things",
            "adds two things together",
            |args, editor, env| {
                eval(
                    Expr::Add(Box::new(args[0].clone()), Box::new(args[1].clone())),
                    editor,
                    env,
                )
            },
        )),
    );
    editor.env.scope.insert(
        Expr::Symbol(String::from("sub")),
        Expr::Builtin(Builtin::new(
            "sub",
            "subtracts two things",
            "subtracts two things together",
            |args, editor, env| {
                eval(
                    Expr::Sub(Box::new(args[0].clone()), Box::new(args[1].clone())),
                    editor,
                    env,
                )
            },
        )),
    );
    editor.env.scope.insert(
        Expr::Symbol(String::from("mul")),
        Expr::Builtin(Builtin::new(
            "mul",
            "multiplies two things",
            "multiplies two things together",
            |args, editor, env| {
                eval(
                    Expr::Mul(Box::new(args[0].clone()), Box::new(args[1].clone())),
                    editor,
                    env,
                )
            },
        )),
    );
    editor.env.scope.insert(
        Expr::Symbol(String::from("div")),
        Expr::Builtin(Builtin::new(
            "div",
            "divides two things",
            "divides two things together",
            |args, editor, env| {
                eval(
                    Expr::Div(Box::new(args[0].clone()), Box::new(args[1].clone())),
                    editor,
                    env,
                )
            },
        )),
    );
    editor.env.scope.insert(
        Expr::Symbol(String::from("rem")),
        Expr::Builtin(Builtin::new(
            "rem",
            "remainders two things",
            "remainders two things together",
            |args, editor, env| {
                eval(
                    Expr::Rem(Box::new(args[0].clone()), Box::new(args[1].clone())),
                    editor,
                    env,
                )
            },
        )),
    );
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
    frontend.set_status(&format!("Editing in buffer #{}: {}", editor.cur_buf_id(), editor.cur_buf().unwrap().get_file_name().unwrap_or("unnamed"))).unwrap();

    loop {
        frontend.render(&editor, false).unwrap();

        let input = frontend.wait_for_input(&editor);
        // println!("{:?}", input);
        // std::thread::sleep(std::time::Duration::from_millis(1000));
        if Ok(Input::Alt(Box::new(Input::Char('q')))) == input {
            break;
        } else {
            match input {
                Ok(Input::Shift(shift)) => {
                        match *shift {
                        Input::Char(ch) => {
                            selected = false;
                            editor.unselect();
                            editor.insert(ch.to_uppercase())
                        }
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
                        Input::Home => {
                            if !selected {
                                editor.select();
                                selected = true;
                            }
                            while editor.cur_buf().unwrap().cur_pos().1 > 0 {
                                editor.move_cur(Direction::Left);
                            }
                        }
                        Input::End => {
                            if !selected {
                                editor.select();
                                selected = true;
                            }
                            while editor.cur_buf().unwrap().cur_pos().1 < editor.cur_buf().unwrap().cur_line().len() {
                                editor.move_cur(Direction::Right);
                            }
                        }
                        Input::PageDown => {
                            if !selected {
                                editor.select();
                                selected = true;
                            }
                            for _ in 0..frontend.height() {
                                editor.move_cur(Direction::Down);
                            }
                        }
                        Input::PageUp => {
                            if !selected {
                                editor.select();
                                selected = true;
                            }
                            for _ in 0..frontend.height() {
                                editor.move_cur(Direction::Up);
                            }
                        }
                        // other => panic!("{:?}", other),
                        _ => {}
                    }
                    frontend.render(&editor, true).expect("Failed to render");
                },

                Ok(Input::PageUp) => {
                    for _ in 0..frontend.height() {
                        editor.move_cur(Direction::Up);
                    }
                }
                Ok(Input::PageDown) => {
                    for _ in 0..frontend.height() {
                        editor.move_cur(Direction::Down);
                    }
                }
                Ok(Input::Home) => {
                    while editor.cur_buf().unwrap().cur_pos().1 > 0 {
                        editor.move_cur(Direction::Left);
                    }
                }
                Ok(Input::End) => {
                    while editor.cur_buf().unwrap().cur_pos().1 < editor.cur_buf().unwrap().cur_line().len() {
                        editor.move_cur(Direction::Right);
                    }
                }

                Ok(Input::Char(ch)) => {
                    selected = false;
                    editor.unselect();
                    editor.insert(ch);
                }

                Ok(Input::Alt(alt)) => {
                    match *alt {
                        Input::Char('1') => editor.set_buf(1),
                        Input::Char('2') => editor.set_buf(2),
                        Input::Char('3') => editor.set_buf(3),
                        Input::Char('4') => editor.set_buf(4),
                        Input::Char('5') => editor.set_buf(5),
                        Input::Char('6') => editor.set_buf(6),
                        Input::Char('7') => editor.set_buf(7),
                        Input::Char('8') => editor.set_buf(8),
                        Input::Char('9') => editor.set_buf(9),
                        Input::Char('0') => editor.set_buf(0),
                        Input::Char('!') => {
                            // Get a shell command
                            if let Ok(cmd) = frontend.prompt("Enter shell command: ") {
                                let words = cmd.split_whitespace().collect::<Vec<&str>>();
                                // Join together the stdout and stderr
                                if let Ok(output) = std::process::Command::new(words[0])
                                    .args(&words[1..])
                                    .output() {

                                    let output = format!(
                                        "STDOUT:\n{}\nSTDERR:\n{}",
                                        String::from_utf8_lossy(&output.stdout),
                                        String::from_utf8_lossy(&output.stderr)
                                    );
                                    let buf = Buffer::from_text(&output);
                                    editor.add_buf(buf);
                                    editor.set_buf(editor.max_buf_id());
                                } else {
                                    frontend.set_status("Failed to run command").unwrap();
                                }
                            }
                        },
                        Input::Char('\'') => {
                            editor.next_buf();
                        },
                        Input::Char('"') => {
                            editor.prev_buf();
                        },
                        _ => {}
                    }
                    frontend.set_status(&format!("Editing in buffer #{}: {}", editor.cur_buf_id(), editor.cur_buf().unwrap().get_file_name().unwrap_or("unnamed"))).unwrap();
                }

                Ok(Input::Control(ctrl)) => {
                    match *ctrl {
                        Input::Char('s') => {
                            let buf = editor.cur_buf_mut().unwrap();
                            match buf.get_file_name().map(String::from) {
                                Some(file_name) => {
                                    buf.save(&file_name).unwrap();
                                },
                                None => {
                                    if let Ok(file_name) = frontend.prompt("Enter file name: ") {
                                        buf.save(&file_name).unwrap();
                                    }
                                }
                            }
                        },
                        Input::Char('q') => {
                            if !editor.cur_buf().unwrap().is_edited() {
                                editor.quit_buf(false);
                                continue;
                            }

                            let should_save = frontend.ask("Do you want to save the buffer?", "y", "n").unwrap();
                            if should_save && editor.cur_buf().unwrap().get_file_name().is_none() {
                                if let Ok(filename) = frontend.prompt("Enter file name: ") {
                                    editor.cur_buf_mut().unwrap().set_file_name(filename);
                                    editor.quit_buf(should_save);
                                }
                            } else {
                                editor.quit_buf(should_save);
                            }
                            frontend.set_status(&format!("Editing in buffer #{}: {}", editor.cur_buf_id(), editor.cur_buf().unwrap().get_file_name().unwrap_or("unnamed"))).unwrap();
                        },
                        Input::Char('o') => {
                            if let Ok(file) = frontend.prompt("Enter file name: ") {
                                editor.add_buf(Buffer::from_file_name(file));
                                editor.set_buf(editor.max_buf_id());
                                frontend.set_status(&format!("Editing in buffer #{}: {}", editor.cur_buf_id(), editor.cur_buf().unwrap().get_file_name().unwrap_or("unnamed"))).unwrap();
                            }
                        },

                        Input::Char('a') => {
                            editor.goto_cur((0, 0));
                            editor.select();
                            let lines = editor.cur_buf().unwrap().content();
                            let row = lines.len() - 1;
                            let col = lines[row].len();
                            editor.goto_cur((row, col));
                        },
                        Input::Char('z') => {
                            editor.undo();
                            frontend.render(&editor, true).expect("Failed to render");
                        },
                        Input::Char('y') => {
                            editor.redo();
                            frontend.render(&editor, true).expect("Failed to render");
                        },
                        Input::Char('c') => {
                            if let Some(selected) = editor.get_selected() {
                                copied = selected.clone();
                            }
                        }
                        Input::Char('d') => {
                            if let Some(selected_text) = editor.get_selected() {
                                let size = selected_text.len();
                                editor.goto_cur(editor.selection_end().unwrap());
                                editor.delete(size);
                                editor.unselect();
                                selected = false;
                                frontend.render(&editor, true).expect("Failed to render");
                            } else {
                                editor.delete(1);
                            }
                        }
                        Input::Char('x') => {
                            if let Some(selected_text) = editor.get_selected() {
                                copied = selected_text.clone();
                                let size = selected_text.len();
                                editor.goto_cur(editor.selection_end().unwrap());
                                editor.delete(size);
                                editor.unselect();
                                selected = false;
                                frontend.render(&editor, true).expect("Failed to render");
                            }
                        }
                        Input::Char('v') if !copied.is_empty() => {
                            editor.insert(&copied);
                            selected = false;
                            editor.unselect();
                        },
                        _ => {}
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
                    // If we are at the beginning of the document, do nothing.
                    if editor.cur_buf().unwrap().cur_pos() == (0, 0) {
                        continue;
                    }

                    if let Some(end) = editor.selection_end() {
                        let size = editor.get_selected().unwrap().len();
                        editor.goto_cur(end);
                        editor.delete(size);
                    } else {
                        editor.delete(1);
                    }
                    selected = false;
                    editor.unselect();
                }

                Ok(Input::Delete) => {
                    if let Some(end) = editor.selection_end() {
                        let size = editor.get_selected().unwrap().len();
                        editor.goto_cur(end);
                        editor.delete(size);
                    } else {
                        editor.move_cur(Direction::Right);
                        editor.delete(1);
                    }
                    selected = false;
                    editor.unselect();
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
    }

    frontend.exit();

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