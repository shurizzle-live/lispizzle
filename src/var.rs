use std::{cell::RefCell, fmt, hash::Hash, rc::Rc};

use crate::Value;

#[derive(Clone)]
pub struct Var(Rc<RefCell<Value>>);

impl Var {
    #[inline]
    pub fn new(value: Value) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    #[inline]
    pub fn set(&self, mut value: Value) -> Value {
        std::mem::swap(&mut *RefCell::borrow_mut(&*self.0), &mut value);
        value
    }

    #[inline]
    pub fn get(&self) -> Value {
        RefCell::borrow(&*self.0).clone()
    }
}

impl Default for Var {
    #[inline]
    fn default() -> Self {
        Self::new(Value::Unspecified)
    }
}

impl PartialEq for Var {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Var {}

impl Hash for Var {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0.as_ptr() as usize).hash(state);
    }
}

impl fmt::Debug for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#<var {:x}>", self.0.as_ptr() as usize)
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#<var {:x}>", self.0.as_ptr() as usize)
    }
}
