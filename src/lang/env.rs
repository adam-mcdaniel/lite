use super::Expr;
use std::collections::BTreeMap;
// use crate::Buffer;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Env {
    pub scope: BTreeMap<Expr, Expr>,
}
