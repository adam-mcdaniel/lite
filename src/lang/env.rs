use super::Expr;
use std::collections::BTreeMap;
// use crate::Buffer;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Env {
    pub scope: BTreeMap<Expr, Expr>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            scope: BTreeMap::new(),
        }
    }

    pub fn get(&self, key: &Expr) -> Option<&Expr> {
        self.scope.get(key)
    }

    pub fn set(&mut self, key: Expr, value: Expr) {
        self.scope.insert(key, value);
    }

    pub fn remove(&mut self, key: &Expr) {
        self.scope.remove(key);
    }

    pub fn alias(&mut self, key: Expr, value: Expr) {
        self.scope.insert(key, value);
    }
}