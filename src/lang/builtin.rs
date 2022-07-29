use super::{Env, Expr};
use crate::Editor;
use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
};

#[derive(Clone)]
pub struct Builtin {
    pub name: String,
    pub help: String,
    pub help_long: String,
    code: fn(Vec<Expr>, &mut Editor, &mut Env) -> Result<Expr, Expr>,
}

impl Builtin {
    pub fn new(
        name: impl ToString,
        help: impl ToString,
        help_long: impl ToString,
        code: fn(Vec<Expr>, &mut Editor, &mut Env) -> Result<Expr, Expr>,
    ) -> Self {
        Self {
            name: name.to_string(),
            help: help.to_string(),
            help_long: help_long.to_string(),
            code,
        }
    }

    pub fn call(&self, args: Vec<Expr>, buf: &mut Editor, env: &mut Env) -> Result<Expr, Expr> {
        (self.code)(args, buf, env)
    }
}

impl fmt::Debug for Builtin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Eq for Builtin {}

impl PartialEq for Builtin {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Ord for Builtin {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Builtin {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Hash for Builtin {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}
