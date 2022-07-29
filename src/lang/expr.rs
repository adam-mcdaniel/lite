use super::*;
use crate::{Buffer, Change, Direction, Editor};
use std::{collections::BTreeMap, fmt};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Float(u64);

impl From<f64> for Float {
    fn from(n: f64) -> Self {
        Self(n.to_bits())
    }
}

impl From<Float> for f64 {
    fn from(n: Float) -> Self {
        f64::from_bits(n.0)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Expr {
    Quote(Box<Self>),
    Symbol(String),
    Int(isize),
    Float(Float),
    Bool(bool),
    String(String),
    Group(Box<Self>),
    List(Vec<Self>),
    Dict(BTreeMap<Self, Self>),
    Builtin(Builtin),
    None,
    To(Box<Self>, Box<Self>),
    Get(Box<Self>, Box<Self>),

    Neg(Box<Self>),
    Add(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),
    Mul(Box<Self>, Box<Self>),
    Div(Box<Self>, Box<Self>),
    Rem(Box<Self>, Box<Self>),
    Pow(Box<Self>, Box<Self>),

    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    Not(Box<Self>),

    Do(Vec<Self>),

    Macro(Vec<Self>, Box<Self>),
    Proc(Vec<Self>, Box<Self>),
    Fn(Vec<Self>, Box<Self>, Env),
    Apply(Box<Self>, Vec<Self>),
    Let(Box<Self>, Box<Self>, Box<Self>),
    Assign(Box<Self>, Box<Self>),

    If(Box<Self>, Box<Self>, Box<Self>),
    Try(Box<Self>, Box<Self>), Raise(Box<Self>),
    For(Box<Self>, Box<Self>, Box<Self>),
    While(Box<Self>, Box<Self>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::String(s) => write!(f, "{}", s),
            other => write!(f, "{:?}", other),
        }
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Quote(e) => write!(f, "'{e:?}"),
            Self::Symbol(s) => write!(f, "{s}"),
            Self::Int(n) => write!(f, "{n}"),
            Self::Float(n) => write!(f, "{}", f64::from(*n)),
            Self::Bool(b) => write!(f, "{}", if *b {"True"} else {"False"}),
            Self::String(s) => {
                let debug = format!("{s:?}");
                write!(f, "\"{}\"", &debug[1..debug.len() - 1])
            }
            Self::None => write!(f, "None"),
            Self::List(items) => write!(f, "{items:?}"),
            Self::Dict(dict) => write!(f, "{dict:?}"),
            Self::Group(e) => write!(f, "({e:?})"),
            Self::Builtin(builtin) => write!(f, "{builtin:?}"),
            Self::To(start, end) => write!(f, "{start:?} to {end:?}"),
            Self::Get(val, idx) => write!(f, "{val:?}@{idx:?}"),

            Self::Neg(e) => write!(f, "-{e:?}"),
            Self::Add(a, b) => write!(f, "{a:?} + {b:?}"),
            Self::Sub(a, b) => write!(f, "{a:?} - {b:?}"),
            Self::Mul(a, b) => write!(f, "{a:?} * {b:?}"),
            Self::Div(a, b) => write!(f, "{a:?} / {b:?}"),
            Self::Rem(a, b) => write!(f, "{a:?} % {b:?}"),
            Self::Pow(a, b) => write!(f, "{a:?} ^ {b:?}"),

            Self::And(a, b) => write!(f, "{a:?} & {b:?}"),
            Self::Or(a, b) => write!(f, "{a:?} | {b:?}"),
            Self::Not(e) => write!(f, "!{e:?}"),

            Self::Do(exprs) => {
                write!(f, "{{ ")?;
                for expr in exprs {
                    write!(f, "{expr:?}; ")?
                }
                write!(f, "}}")
            }
            Self::Macro(args, ret) => {
                write!(f, "macro(")?;
                for (i, arg) in args.iter().enumerate() {
                    write!(f, "{arg:?}{}", if i == args.len() - 1 { "" } else { ", " })?
                }
                write!(f, ") -> {ret:?}")?;
                Ok(())
            }
            Self::Proc(args, ret) => {
                write!(f, "proc(")?;
                for (i, arg) in args.iter().enumerate() {
                    write!(f, "{arg:?}{}", if i == args.len() - 1 { "" } else { ", " })?
                }
                write!(f, ") -> {ret:?}")?;
                Ok(())
            }
            Self::Fn(args, ret, _) => {
                write!(f, "fn(")?;
                for (i, arg) in args.iter().enumerate() {
                    write!(f, "{arg:?}{}", if i == args.len() - 1 { "" } else { ", " })?
                }
                write!(f, ") -> {ret:?}")?;
                Ok(())
            }
            Self::Apply(g, args) => {
                write!(f, "{g:?}")?;
                for arg in args {
                    write!(f, " {arg:?}")?
                }
                Ok(())
            }

            Self::Let(var, val, ret) => write!(f, "let {var:?} = {val:?} in {ret:?}"),
            Self::Assign(var, val) => write!(f, "{var:?} = {val:?}"),
            Self::For(var, items, body) => write!(f, "for {var:?} in {items:?} do {body:?}"),
            Self::While(cond, body) => write!(f, "while {cond:?} {body:?}"),
            Self::Try(var, val) => write!(f, "try {var:?} catch {val:?}"),
            Self::Raise(e) => write!(f, "raise {e:?}"),
            Self::If(c, t, e) => write!(f, "if {c:?} then {t:?} else {e:?}"),
        }
    }
}

pub fn get_nth_arg(args: &Vec<Expr>, n: usize) -> Result<Expr, Expr> {
    if let Some(e) = args.iter().nth(n) {
        Ok(e.clone())
    } else {
        return err("TooFewArgs", Expr::List(args.clone()));
    }
}

pub fn select(args: Vec<Expr>, editor: &mut Editor, env: &mut Env) -> Result<Expr, Expr> {
    // for arg in args {
    //     match eval(arg, editor, env)? {
    //         Expr::String(s) => editor.insert(s),
    //         other => return err("TypeMismatch", other),
    //     }
    // }
    editor.select();
    Ok(Expr::None)
}
pub fn unselect(args: Vec<Expr>, editor: &mut Editor, env: &mut Env) -> Result<Expr, Expr> {
    // for arg in args {
    //     match eval(arg, editor, env)? {
    //         Expr::String(s) => editor.insert(s),
    //         other => return err("TypeMismatch", other),
    //     }
    // }
    editor.unselect();
    Ok(Expr::None)
}

pub fn insert(args: Vec<Expr>, editor: &mut Editor, env: &mut Env) -> Result<Expr, Expr> {
    for arg in args {
        match eval(arg, editor, env)? {
            Expr::String(s) => editor.insert(s),
            other => return err("TypeMismatch", other),
        }
    }
    Ok(Expr::None)
}

pub fn delete(args: Vec<Expr>, editor: &mut Editor, env: &mut Env) -> Result<Expr, Expr> {
    if args.len() > 1 {
        return err("TooManyArgs", Expr::List(args));
    }

    let e = get_nth_arg(&args, 0)?;
    match eval(e, editor, env)? {
        Expr::Int(count) if count > 0 => editor.delete(count as usize),
        Expr::Int(_) => {}
        other => return err("TypeMismatch", other),
    }

    Ok(Expr::None)
}

pub fn get_undo_stack_len(args: Vec<Expr>, editor: &mut Editor, env: &mut Env) -> Result<Expr, Expr> {
    Ok(if let Some(buf) = editor.cur_buf() {
        Expr::Int(buf.undo_stack.len() as isize)
    } else {
        Expr::None
    })
}

pub fn undo(args: Vec<Expr>, editor: &mut Editor, env: &mut Env) -> Result<Expr, Expr> {
    if args.len() > 1 {
        return err("TooManyArgs", Expr::List(args));
    }

    let e = get_nth_arg(&args, 0)?;
    match eval(e, editor, env)? {
        Expr::Int(count) if count > 0 => {
            for _ in 0..count as usize {
                editor.undo()
            }
        },
        Expr::Int(_) => {}
        other => return err("TypeMismatch", other),
    }

    Ok(Expr::None)
}

pub fn redo(args: Vec<Expr>, editor: &mut Editor, env: &mut Env) -> Result<Expr, Expr> {
    if args.len() > 1 {
        return err("TooManyArgs", Expr::List(args));
    }

    let e = get_nth_arg(&args, 0)?;
    match eval(e, editor, env)? {
        Expr::Int(count) if count > 0 => {
            for _ in 0..count as usize {
                editor.redo()
            }
        },
        Expr::Int(_) => {}
        other => return err("TypeMismatch", other),
    }

    Ok(Expr::None)
}

pub fn move_cursor(args: Vec<Expr>, editor: &mut Editor, env: &mut Env) -> Result<Expr, Expr> {
    for arg in args {
        match eval(arg, editor, env)? {
            Expr::String(dir) | Expr::Symbol(dir) => editor.move_cur(match dir.as_str() {
                "left" => Direction::Left,
                "right" => Direction::Right,
                "up" => Direction::Up,
                "down" => Direction::Down,
                "nowhere" => Direction::Nowhere,
                _ => return err("InvalidArg", Expr::String(dir)),
            }),
            other => return err("TypeMismatch", other),
        }
    }
    Ok(Expr::None)
}

pub fn goto_cursor(args: Vec<Expr>, editor: &mut Editor, env: &mut Env) -> Result<Expr, Expr> {
    if args.len() > 2 {
        return err("TooManyArgs", Expr::List(args));
    }

    let row = get_nth_arg(&args, 0)?;
    let col = get_nth_arg(&args, 1)?;
    match (eval(row, editor, env)?, eval(col, editor, env)?) {
        (Expr::Int(row), Expr::Int(col)) => editor.goto_cur((row as usize, col as usize)),
        (a, b) => return err("TypeMismatch", Expr::List(vec![a, b])),
    }

    Ok(Expr::None)
}

pub fn get_selection_start(
    args: Vec<Expr>,
    editor: &mut Editor,
    env: &mut Env,
) -> Result<Expr, Expr> {
    if args.len() > 0 {
        return err("TooManyArgs", Expr::List(args));
    }

    Ok(
        if let Some((row, col)) = editor.cur_buf().and_then(Buffer::selection_start) {
            Expr::List(vec![Expr::Int(row as isize), Expr::Int(col as isize)])
        } else {
            Expr::None
        },
    )
}

pub fn get_selection_end(
    args: Vec<Expr>,
    editor: &mut Editor,
    env: &mut Env,
) -> Result<Expr, Expr> {
    if args.len() > 0 {
        return err("TooManyArgs", Expr::List(args));
    }

    Ok(
        if let Some((row, col)) = editor.cur_buf().and_then(Buffer::selection_end) {
            Expr::List(vec![Expr::Int(row as isize), Expr::Int(col as isize)])
        } else {
            Expr::None
        },
    )
}

pub fn get_selection_len(
    args: Vec<Expr>,
    editor: &mut Editor,
    env: &mut Env,
) -> Result<Expr, Expr> {
    if args.len() > 0 {
        return err("TooManyArgs", Expr::List(args));
    }

    Ok(if let Some(selected) = editor.get_selected() {
        Expr::Int(selected.len() as isize)
    } else {
        Expr::None
    })
}

pub fn get_selected(args: Vec<Expr>, editor: &mut Editor, env: &mut Env) -> Result<Expr, Expr> {
    if args.len() > 0 {
        return err("TooManyArgs", Expr::List(vec![]));
    }

    Ok(if let Some(selected) = editor.get_selected() {
        Expr::String(selected)
    } else {
        Expr::None
    })
}

pub fn get_selected_lines(
    args: Vec<Expr>,
    editor: &mut Editor,
    env: &mut Env,
) -> Result<Expr, Expr> {
    if args.len() > 0 {
        return err("TooManyArgs", Expr::List(vec![]));
    }

    Ok(if let Some(selected) = editor.get_selected_lines() {
        Expr::List(selected.to_owned().into_iter().map(Expr::String).collect())
    } else {
        Expr::None
    })
}

pub fn eval(mut expr: Expr, editor: &mut Editor, env: &mut Env) -> Result<Expr, Expr> {
    loop {
        if let Some(e) = env.scope.get(&expr) {
            return Ok(e.clone());
        }
        return Ok(match expr {
            Expr::Quote(e) => *e,
            Expr::Symbol(_) => {
                if let Some(e) = env.scope.get(&expr) {
                    e.clone()
                } else {
                    return err("SymbolNotDefined", expr);
                }
            }
            Expr::Int(_)
            | Expr::Float(_)
            | Expr::Bool(_)
            | Expr::String(_)
            | Expr::Builtin(_)
            | Expr::None => expr,

            Expr::Group(e) => {
                expr = *e;
                continue;
            }
            Expr::List(items) => Expr::List(
                items
                    .into_iter()
                    .map(|e| eval(e, editor, env))
                    .collect::<Result<Vec<Expr>, Expr>>()?,
            ),
            Expr::Dict(items) => Expr::Dict(
                items
                    .into_iter()
                    .map(|(k, v)| Ok((k, eval(v, editor, env)?)))
                    .collect::<Result<BTreeMap<Expr, Expr>, Expr>>()?,
            ),

            Expr::To(start, end) => match (eval(*start, editor, env)?, eval(*end, editor, env)?) {
                (Expr::Int(start), Expr::Int(end)) => Expr::List(
                    (start..end)
                        .into_iter()
                        .map(|n| Expr::Int(n as isize))
                        .collect::<Vec<Expr>>(),
                ),

                (start, end) => return err("InvalidTo", Expr::To(Box::new(start), Box::new(end))),
            },
            Expr::Get(val, idx) => match (eval(*val, editor, env)?, *idx) {
                (Expr::Dict(items), key) => {
                    if let Some(item) = items.get(&key) {
                        item.clone()
                    } else if let Some(item) = items.get(&eval(key, editor, env)?) {
                        item.clone()
                    } else {
                        Expr::None
                    }
                }
                (Expr::List(items), key) => {
                    if let Expr::Int(n) = eval(key.clone(), editor, env)? {
                        if let Some(item) = items.into_iter().nth(n as usize) {
                            item
                        } else {
                            Expr::None
                        }
                    } else {
                        return err(
                            "InvalidGet",
                            Expr::Get(Box::new(Expr::List(items)), Box::new(key)),
                        );
                    }
                }
                (Expr::String(text), key) => {
                    if let Expr::Int(n) = eval(key.clone(), editor, env)? {
                        if let Some(ch) = text.chars().nth(n as usize) {
                            string(ch)
                        } else {
                            string("")
                        }
                    } else {
                        return err(
                            "InvalidGet",
                            Expr::Get(Box::new(Expr::String(text)), Box::new(key)),
                        );
                    }
                }
                (val, idx) => return err("InvalidGet", Expr::Get(Box::new(val), Box::new(idx))),
            },

            Expr::Neg(val) => match eval(*val, editor, env)? {
                Expr::Int(n) => Expr::Int(-n),
                Expr::Float(n) => Expr::Float((-f64::from(n)).into()),
                Expr::List(mut items) => {
                    items.reverse();
                    Expr::List(items)
                }
                Expr::String(text) => Expr::String(text.chars().rev().collect()),
                x => return err("InvalidNeg", Expr::Neg(Box::new(x))),
            },
            Expr::Add(a, b) => match (eval(*a, editor, env)?, eval(*b, editor, env)?) {
                (Expr::Int(m), Expr::Int(n)) => Expr::Int(m + n),
                (Expr::Float(m), Expr::Int(n)) => Expr::Float((f64::from(m) + n as f64).into()),
                (Expr::Int(m), Expr::Float(n)) => Expr::Float((m as f64 + f64::from(n)).into()),
                (Expr::Float(m), Expr::Float(n)) => {
                    Expr::Float((f64::from(m) + f64::from(n)).into())
                }

                (Expr::String(mut text1), Expr::String(text2)) => {
                    text1 += &text2;
                    Expr::String(text1)
                }

                (Expr::String(mut text1), Expr::Int(n)) => {
                    text1 += &n.to_string();
                    Expr::String(text1)
                }

                (Expr::String(mut text1), Expr::Float(n)) => {
                    text1 += &f64::from(n).to_string();
                    Expr::String(text1)
                }

                (Expr::List(mut items1), Expr::List(items2)) => {
                    items1.extend(items2.into_iter());
                    Expr::List(items1)
                }

                (Expr::Dict(mut items1), Expr::Dict(items2)) => {
                    items1.extend(items2.into_iter());
                    Expr::Dict(items1)
                }

                (a, b) => return err("InvalidAdd", Expr::Add(Box::new(a), Box::new(b))),
            },
            Expr::Sub(a, b) => match (eval(*a, editor, env)?, eval(*b, editor, env)?) {
                (Expr::Int(m), Expr::Int(n)) => Expr::Int(m - n),
                (Expr::Float(m), Expr::Int(n)) => Expr::Float((f64::from(m) - n as f64).into()),
                (Expr::Int(m), Expr::Float(n)) => Expr::Float((m as f64 - f64::from(n)).into()),
                (Expr::Float(m), Expr::Float(n)) => {
                    Expr::Float((f64::from(m) - f64::from(n)).into())
                }

                (a, b) => return err("InvalidSub", Expr::Sub(Box::new(a), Box::new(b))),
            },
            Expr::Mul(a, b) => match (eval(*a, editor, env)?, eval(*b, editor, env)?) {
                (Expr::Int(m), Expr::Int(n)) => Expr::Int(m * n),
                (Expr::Float(m), Expr::Int(n)) => Expr::Float((f64::from(m) * n as f64).into()),
                (Expr::Int(m), Expr::Float(n)) => Expr::Float((m as f64 * f64::from(n)).into()),
                (Expr::Float(m), Expr::Float(n)) => {
                    Expr::Float((f64::from(m) * f64::from(n)).into())
                }

                (Expr::String(text1), Expr::Int(n)) => Expr::String(text1.repeat(n as usize)),

                (Expr::List(items1), Expr::Int(n)) => {
                    let mut result = vec![];
                    for _ in 0..n {
                        result.extend(items1.clone().into_iter())
                    }
                    Expr::List(result)
                }

                (a, b) => return err("InvalidMul", Expr::Mul(Box::new(a), Box::new(b))),
            },
            Expr::Div(a, b) => match (eval(*a, editor, env)?, eval(*b, editor, env)?) {
                (Expr::Int(m), Expr::Int(n)) => Expr::Int(m / n),
                (Expr::Float(m), Expr::Int(n)) => Expr::Float((f64::from(m) / n as f64).into()),
                (Expr::Int(m), Expr::Float(n)) => Expr::Float((m as f64 / f64::from(n)).into()),
                (Expr::Float(m), Expr::Float(n)) => {
                    Expr::Float((f64::from(m) / f64::from(n)).into())
                }

                (a, b) => return err("InvalidDiv", Expr::Div(Box::new(a), Box::new(b))),
            },
            Expr::Rem(a, b) => match (eval(*a, editor, env)?, eval(*b, editor, env)?) {
                (Expr::Int(m), Expr::Int(n)) => Expr::Int(m % n),
                (Expr::Float(m), Expr::Int(n)) => Expr::Float((f64::from(m) % n as f64).into()),
                (Expr::Int(m), Expr::Float(n)) => Expr::Float((m as f64 % f64::from(n)).into()),
                (Expr::Float(m), Expr::Float(n)) => {
                    Expr::Float((f64::from(m) % f64::from(n)).into())
                }

                (a, b) => return err("InvalidRem", Expr::Rem(Box::new(a), Box::new(b))),
            },
            Expr::Pow(base, power) => match (eval(*base, editor, env)?, eval(*power, editor, env)?)
            {
                (Expr::Int(m), Expr::Int(n)) => Expr::Int(m.pow(n as u32)),
                (Expr::Float(m), Expr::Int(n)) => Expr::Float((f64::from(m).powf(n as f64)).into()),
                (Expr::Int(m), Expr::Float(n)) => {
                    Expr::Float(((m as f64).powf(f64::from(n))).into())
                }
                (Expr::Float(m), Expr::Float(n)) => {
                    Expr::Float((f64::from(m).powf(f64::from(n))).into())
                }

                (a, b) => return err("InvalidPow", Expr::Pow(Box::new(a), Box::new(b))),
            },

            Expr::And(a, b) => match (eval(*a, editor, env)?, eval(*b, editor, env)?) {
                (Expr::Bool(a), Expr::Bool(b)) => Expr::Bool(a && b),
                (a, b) => return err("InvalidAnd", Expr::And(Box::new(a), Box::new(b))),
            },
            Expr::Or(a, b) => match (eval(*a, editor, env)?, eval(*b, editor, env)?) {
                (Expr::Bool(a), Expr::Bool(b)) => Expr::Bool(a || b),
                (a, b) => return err("InvalidOr", Expr::Or(Box::new(a), Box::new(b))),
            },
            Expr::Not(val) => match eval(*val, editor, env)? {
                Expr::Bool(b) => Expr::Bool(!b),
                val => return err("InvalidNot", Expr::Not(Box::new(val))),
            },

            Expr::Assign(var, val) => {
                let val = eval(*val, editor, env)?;
                env.scope.insert(*var, val);
                Expr::None
            }
            Expr::Let(var, val, ret) => {
                let mut new_env = env.clone();
                new_env.scope.insert(*var, eval(*val, editor, env)?);
                eval(*ret, editor, &mut new_env)?
            }
            Expr::If(c, t, e) => {
                expr = match eval(*c, editor, env)? {
                    Expr::Bool(true) => *t,
                    Expr::Bool(false) => *e,
                    cond => return err("InvalidCond", cond),
                };
                continue;
            }
            Expr::Raise(e) => return Err(eval(*e, editor, env)?),
            Expr::Try(t, c) => match eval(*t, editor, env) {
                Ok(result) => result,
                Err(e) => {
                    expr = Expr::Apply(c, vec![e]);
                    continue;
                }
            },
            Expr::While(cond, body) => {
                loop {
                    match eval(*cond.clone(), editor, env)? {
                        Expr::Bool(true) => {
                            eval(*body.clone(), editor, env)?;
                        }
                        Expr::Bool(false) => break,
                        cond => return err("InvalidCond", cond),
                    }
                }
                Expr::None
            }
            Expr::For(var, vals, body) => match eval(*vals, editor, env)? {
                Expr::Dict(dict) => {
                    for (key, val) in dict.into_iter() {
                        env.scope.insert(*var.clone(), Expr::List(vec![key, val]));
                        eval(*body.clone(), editor, env)?;
                    }
                    Expr::None
                }
                Expr::List(list) => {
                    for val in list {
                        env.scope.insert(*var.clone(), val);
                        eval(*body.clone(), editor, env)?;
                    }
                    Expr::None
                }
                Expr::String(s) => {
                    for val in s.chars() {
                        env.scope.insert(*var.clone(), string(val));
                        eval(*body.clone(), editor, env)?;
                    }
                    Expr::None
                }
                iter => return err("InvalidIter", iter),
            },

            Expr::Do(exprs) => {
                let exprs_len = exprs.len();
                for (i, expr) in exprs.into_iter().enumerate() {
                    let expr = eval(expr, editor, env)?;
                    if i == exprs_len - 1 {
                        return Ok(expr);
                    }
                }
                Expr::None
            }

            Expr::Fn(args, ret, mut captured) => {
                captured.scope.extend(env.scope.clone().into_iter());
                Expr::Fn(args, ret, captured)
            }
            Expr::Macro(args, ret) => Expr::Macro(args, ret),
            Expr::Proc(args, ret) => Expr::Proc(args, ret),
            Expr::Apply(f, args) => match eval(*f, editor, env)? {
                Expr::Fn(params, ret, mut captured) => {
                    for (param, arg) in params.into_iter().zip(args.into_iter()) {
                        captured.scope.insert(param, eval(arg, editor, env)?);
                    }
                    eval(*ret, editor, &mut captured)?
                }
                Expr::Proc(params, ret) => {
                    let mut new_env = Env::default();
                    for (param, arg) in params.into_iter().zip(args.into_iter()) {
                        new_env.scope.insert(param, eval(arg, editor, env)?);
                    }
                    eval(*ret, editor, &mut new_env)?
                }
                Expr::Macro(params, ret) => {
                    for (param, arg) in params.into_iter().zip(args.into_iter()) {
                        let val = eval(arg, editor, env)?;
                        env.scope.insert(param, val);
                    }
                    eval(*ret, editor, env)?
                }
                Expr::Builtin(builtin) => builtin.call(args, editor, env)?,
                f => return err("InvalidFn", f),
            },
        });
    }
    Ok(Expr::None)
}
