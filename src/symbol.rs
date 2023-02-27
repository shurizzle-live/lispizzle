use std::{fmt, hash::Hash};

use crate::Str;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Symbol {
    Name(Str),
    Gensym(usize, usize),
}

impl From<Str> for Symbol {
    #[inline]
    fn from(value: Str) -> Self {
        Self::Name(value)
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Name(name) => fmt::Display::fmt(name, f),
            Self::Gensym(env, n) => write!(f, "gensym({},{})", env, n),
        }
    }
}

impl fmt::Display for Symbol {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
