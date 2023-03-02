use std::borrow::Borrow;

use crate::{BackTrace, Str, StrCache, TraceFrame};

#[derive(Clone)]
pub struct Context {
    cache: StrCache,
    trace: BackTrace,
}

impl Context {
    #[inline]
    pub fn new() -> Self {
        Self::with_cache(StrCache::new())
    }

    #[inline]
    pub fn with_cache(cache: StrCache) -> Self {
        Self {
            cache,
            trace: BackTrace::new(),
        }
    }

    #[inline]
    pub fn trace(&self) -> BackTrace {
        self.trace.clone()
    }

    pub fn with_frame(&self, frame: TraceFrame) -> Self {
        Self {
            cache: self.cache.clone(),
            trace: self.trace.with_frame(frame),
        }
    }

    #[inline]
    pub fn make_string<T: Borrow<str> + Into<Str>>(&mut self, s: T) -> Str {
        self.cache.get(s)
    }
}

impl Default for Context {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
