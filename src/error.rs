use std::fmt;

use im_rc::Vector;

use crate::{BackTrace, Str, Value};

#[derive(Debug, Clone)]
pub struct Error {
    name: Str,
    args: Option<Vector<Value>>,
    trace: BackTrace,
}

impl Error {
    #[inline]
    pub fn new(name: Str, args: Option<Vector<Value>>, trace: BackTrace) -> Self {
        Self { name, args, trace }
    }

    #[inline]
    pub fn name(&self) -> Str {
        self.name.clone()
    }

    #[inline]
    pub fn args(&self) -> Option<Vector<Value>> {
        self.args.clone()
    }

    #[inline]
    pub fn backtrace(&self) -> BackTrace {
        self.trace.clone()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}
