use std::{fmt, rc::Rc};

use im_rc::Vector;

use crate::{Error, Str, Symbol, Value};

#[derive(Debug, Clone)]
enum TraceFrameRepr {
    Main,
    Unnamed(usize),
    Named(usize, Symbol),
}

#[derive(Debug, Clone)]
pub struct TraceFrame(TraceFrameRepr);

struct BackTraceRepr {
    parent: Option<Rc<BackTraceRepr>>,
    frame: TraceFrame,
}

pub struct BTrace(Rc<BackTraceRepr>);

impl BTrace {
    #[inline]
    pub fn new() -> Self {
        Self(Rc::new(BackTraceRepr {
            parent: None,
            frame: TraceFrame::main(),
        }))
    }

    #[inline]
    pub fn current(&self) -> TraceFrame {
        self.0.frame.clone()
    }

    pub fn parent(&self) -> Option<Self> {
        self.0.parent.as_ref().map(Rc::clone).map(Self)
    }

    #[inline]
    pub fn error<S: Into<Str>>(self, name: S, args: Option<Vector<Value>>) -> Error {
        Error::new(name.into(), args, self)
    }

    #[inline]
    pub fn with_frame(&self, frame: TraceFrame) -> Self {
        Self(Rc::new(BackTraceRepr {
            parent: Some(Rc::clone(&self.0)),
            frame,
        }))
    }
}

pub struct IntoIter {
    inner: Option<BTrace>,
}

impl Iterator for IntoIter {
    type Item = TraceFrame;

    fn next(&mut self) -> Option<Self::Item> {
        let trace = self.inner.take()?;

        let res = trace.current();
        self.inner = trace.parent();
        Some(res)
    }
}

impl IntoIterator for BTrace {
    type Item = TraceFrame;

    type IntoIter = IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter { inner: Some(self) }
    }
}

impl Default for BTrace {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for BTrace {
    #[inline]
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl fmt::Debug for BTrace {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#<backtrace>")
    }
}

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
