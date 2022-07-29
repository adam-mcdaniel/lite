use std::collections::BTreeMap;

mod expr;
pub use expr::*;
mod env;
pub use env::*;
mod builtin;
pub use builtin::*;
mod parse;
pub use parse::*;

pub fn symbol(s: impl ToString) -> Expr {
    Expr::Symbol(s.to_string())
}
pub fn string(s: impl ToString) -> Expr {
    Expr::String(s.to_string())
}

pub fn dict(d: &[(Expr, Expr)]) -> Expr {
    let mut result = BTreeMap::new();
    for (k, v) in d {
        result.insert(k.clone(), v.clone());
    }
    Expr::Dict(result)
}

pub fn err(kind: impl ToString, expr: Expr) -> Result<Expr, Expr> {
    Err(dict(&[
        (symbol("kind"), string(kind)),
        (symbol("expr"), expr),
    ]))
}
