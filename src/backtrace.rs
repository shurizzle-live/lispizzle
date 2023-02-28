use std::fmt;

use im_rc::Vector;

use crate::Str;

#[derive(Debug, Clone)]
enum TraceFrameRepr {
    Main,
    Unnamed(usize),
    Named(usize, Str),
}

#[derive(Debug, Clone)]
pub struct TraceFrame(TraceFrameRepr);

pub type BackTrace = Vector<TraceFrame>;

impl TraceFrame {
    pub const fn main() -> Self {
        Self(TraceFrameRepr::Main)
    }

    #[inline]
    pub const fn unnamed(address: usize) -> Self {
        Self(TraceFrameRepr::Unnamed(address))
    }

    #[inline]
    pub const fn named(address: usize, name: Str) -> Self {
        Self(TraceFrameRepr::Named(address, name))
    }
}

impl PartialEq for TraceFrameRepr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Main, Self::Main) => true,
            (Self::Unnamed(l0), Self::Unnamed(r0))
            | (Self::Unnamed(l0), Self::Named(r0, _))
            | (Self::Named(l0, _), Self::Unnamed(r0))
            | (Self::Named(l0, _), Self::Named(r0, _)) => l0 == r0,
            _ => false,
        }
    }
}

impl fmt::Display for TraceFrameRepr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Main => write!(f, "<main>"),
            Self::Unnamed(a) => write!(f, "{:x}", a),
            Self::Named(_, s) => fmt::Display::fmt(s, f),
        }
    }
}

impl fmt::Display for TraceFrame {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
