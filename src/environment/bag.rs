use std::{
    borrow::Borrow,
    collections::{hash_map::RandomState, HashMap},
    mem,
};

use crate::{Symbol, Var};

pub enum BagRepr<S = RandomState> {
    Empty,
    Single(Symbol, Var),
    Map(HashMap<Symbol, Var, S>),
}

impl BagRepr {
    pub fn insert(self, key: Symbol, var: Var) -> (Self, Option<Var>) {
        match self {
            Self::Empty => (Self::Single(key, var), None),
            Self::Single(key1, var1) => {
                if key == key1 {
                    (Self::Single(key, var), Some(var1))
                } else {
                    #[allow(clippy::mutable_key_type)]
                    let mut map = HashMap::new();
                    map.insert(key1, var1);
                    map.insert(key, var);
                    (Self::Map(map), None)
                }
            }
            Self::Map(mut map) => {
                let old = map.insert(key, var);
                (Self::Map(map), old)
            }
        }
    }

    pub fn get<B: Borrow<Symbol>>(&self, key: B) -> Option<Var> {
        match self {
            Self::Empty => None,
            Self::Single(ref key1, ref var) => {
                if key1.eq(key.borrow()) {
                    Some(var.clone())
                } else {
                    None
                }
            }
            Self::Map(ref map) => map.get(key.borrow()).cloned(),
        }
    }
}

pub struct Bag<S = RandomState>(BagRepr<S>);

impl Bag {
    #[inline]
    pub fn new() -> Self {
        Self(BagRepr::Empty)
    }

    pub fn insert(&mut self, key: Symbol, var: Var) -> Option<Var> {
        let mut bag = BagRepr::Empty;
        mem::swap(&mut self.0, &mut bag);
        let old;
        (bag, old) = bag.insert(key, var);
        mem::swap(&mut self.0, &mut bag);

        old
    }

    #[inline]
    pub fn get<B: Borrow<Symbol>>(&self, key: B) -> Option<Var> {
        self.0.get(key)
    }
}

impl Default for Bag {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
