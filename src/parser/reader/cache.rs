use std::{borrow::Borrow, cell::RefCell, fmt, rc::Rc};

use crate::Str;

struct Repr(Vec<Str>);

impl Repr {
    #[inline]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn get<T: Borrow<str> + Into<Str>>(&mut self, s: T) -> Str {
        match self.0.binary_search_by(|o| o.as_str().cmp(s.borrow())) {
            Ok(i) => unsafe { self.0.get_unchecked(i).clone() },
            Err(i) => {
                let s = s.into();
                self.0.insert(i, s.clone());
                s
            }
        }
    }
}

pub struct StrCache(Rc<RefCell<Repr>>);

impl StrCache {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(Repr::new())))
    }

    #[inline]
    pub fn get<T: Borrow<str> + Into<Str>>(&mut self, s: T) -> Str {
        RefCell::borrow_mut(&*self.0).get(s)
    }
}

impl Clone for StrCache {
    #[inline]
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl fmt::Debug for StrCache {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&RefCell::borrow(&*self.0).0, f)
    }
}
