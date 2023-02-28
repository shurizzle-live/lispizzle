use std::{
    borrow::Borrow,
    collections::{hash_map::RandomState, HashMap},
    fmt, mem,
};

use crate::{Symbol, Value, Var};

#[derive(Clone)]
enum BagRepr<S = RandomState> {
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

impl fmt::Debug for BagRepr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut x = f.debug_map();

        match self {
            Self::Empty => &mut x,
            Self::Single(ref k, ref v) => x.entry(k, v),
            Self::Map(ref map) => x.entries(map.iter()),
        }
        .finish()
    }
}

#[derive(Clone)]
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

impl fmt::Debug for Bag {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl<I> FromIterator<I> for Bag
where
    I: Into<Symbol>,
{
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        let mut bag = Bag::new();
        for name in iter {
            bag.insert(name.into(), Var::new(Value::Unspecified));
        }
        bag
    }
}
