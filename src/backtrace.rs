use std::fmt;

use im_rc::Vector;

use crate::Str;

#[derive(Debug, Clone)]
pub enum TraceName {
    Main,
    Unknown,
    Known(Str),
}

#[derive(Debug, Clone)]
pub struct Trace {
    name: TraceName,
}

pub type BackTrace = Vector<Trace>;

impl Trace {
    pub fn main() -> Self {
        Self {
            name: TraceName::Main,
        }
    }
}

impl fmt::Display for TraceName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Main => write!(f, "<main>"),
            Self::Unknown => write!(f, "?"),
            Self::Known(s) => fmt::Display::fmt(s, f),
        }
    }
}

impl fmt::Display for Trace {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.name, f)
    }
}
