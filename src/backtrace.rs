use std::fmt;

use im_rc::{vector, vector::ConsumingIter, Vector};

use crate::{Error, Str, Symbol, Value};

#[derive(Clone)]
enum TraceFrameRepr {
    Main,
    Unnamed(usize),
    Named(usize, Symbol),
}

#[derive(Clone)]
pub struct TraceFrame(TraceFrameRepr);

#[derive(Clone)]
pub struct BackTrace(Vector<TraceFrame>);

impl TraceFrame {
    pub const fn main() -> Self {
        Self(TraceFrameRepr::Main)
    }

    #[inline]
    pub const fn unnamed(address: usize) -> Self {
        Self(TraceFrameRepr::Unnamed(address))
    }

    #[inline]
    pub const fn named(address: usize, name: Symbol) -> Self {
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

impl Eq for TraceFrameRepr {}

impl PartialEq for TraceFrame {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
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

impl fmt::Debug for TraceFrameRepr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Main => write!(f, "main"),
            Self::Unnamed(_) => write!(f, "?"),
            Self::Named(_, name) => fmt::Debug::fmt(name, f),
        }
    }
}

impl fmt::Debug for TraceFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#<frame {}>", self.0)
    }
}

impl BackTrace {
    pub fn new() -> Self {
        Self(vector![TraceFrame::main()])
    }

    #[inline]
    pub fn current(&self) -> TraceFrame {
        unsafe { self.0.last().cloned().unwrap_unchecked() }
    }

    pub fn parent(&self) -> Option<Self> {
        if self.0.len() == 1 {
            None
        } else {
            let mut v = self.0.clone();
            v.remove(v.len() - 1);
            Some(Self(v))
        }
    }

    #[inline]
    pub fn error<S: Into<Str>>(self, name: S, args: Option<Vector<Value>>) -> Error {
        Error::new(name.into(), args, self)
    }

    #[inline]
    pub fn with_frame(&self, frame: TraceFrame) -> Self {
        let mut v = self.0.clone();
        v.push_back(frame);
        Self(v)
    }

    pub fn get(&self, i: usize) -> Option<TraceFrame> {
        self.0
            .len()
            .checked_sub(1)
            .and_then(|last| last.checked_sub(i))
            .and_then(|i| self.0.get(i).cloned())
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Default for BackTrace {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for BackTrace {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#<backtrace>")
    }
}

impl PartialEq for BackTrace {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for BackTrace {}

impl IntoIterator for BackTrace {
    type Item = TraceFrame;

    type IntoIter = std::iter::Rev<ConsumingIter<TraceFrame>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().rev()
    }
}
