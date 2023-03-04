use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use crate::{BackTrace, Str, StrCache, Symbol, TraceFrame};

pub struct Context {
    cache: StrCache,
    trace: BackTrace,
    gensym: Rc<RefCell<usize>>,
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
            gensym: Rc::new(RefCell::new(0)),
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
            gensym: Rc::clone(&self.gensym),
        }
    }

    #[inline]
    pub fn make_string<'a, 'b, T: Borrow<str> + Into<Str> + 'a>(&'b mut self, s: T) -> Str {
        self.cache.get(s)
    }

    pub fn make_sym(&self) -> Symbol {
        let mut gensym = RefCell::borrow_mut(&*self.gensym);
        let res = *gensym;
        *gensym += 1;
        Symbol::Gensym(res)
    }
}

impl Default for Context {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Context {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
            trace: self.trace.clone(),
            gensym: Rc::clone(&self.gensym),
        }
    }
}
