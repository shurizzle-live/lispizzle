use std::{fmt, hash::Hash};

use ecow::EcoString;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Symbol {
    Name(EcoString),
    Gensym(usize),
}

impl From<EcoString> for Symbol {
    #[inline]
    fn from(value: EcoString) -> Self {
        Self::Name(value)
    }
}

impl From<usize> for Symbol {
    #[inline]
    fn from(value: usize) -> Self {
        Self::Gensym(value)
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Name(name) => fmt::Display::fmt(name, f),
            Self::Gensym(n) => write!(f, "gensym({})", n),
        }
    }
}

impl fmt::Display for Symbol {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
