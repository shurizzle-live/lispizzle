use std::{
    borrow::Borrow,
    cell::{Ref, RefCell},
    fmt,
    hash::Hash,
    rc::Rc,
};

use im_rc::HashMap;

use crate::{Symbol, Value, Var};

struct EnvironmentRepr {
    parent: Option<Rc<RefCell<EnvironmentRepr>>>,
    gensym: usize,
    storage: HashMap<Symbol, Var>,
}

impl EnvironmentRepr {
    #[inline]
    fn parent(&self) -> Option<Ref<EnvironmentRepr>> {
        self.parent.as_ref().map(|r| RefCell::borrow(r))
    }

    pub fn get<B: Borrow<Symbol>>(&self, key: B) -> Option<Var> {
        let key = key.borrow();

        match key {
            Symbol::Gensym(env, _) if *env == (self as *const _ as usize) => {
                self.storage.get(key).cloned()
            }
            Symbol::Name(_) => self
                .storage
                .get(key)
                .cloned()
                .or_else(|| self.parent().and_then(|p| p.get(key))),
            _ => self.parent().and_then(|p| p.get(key)),
        }
    }

    pub fn set<B: Borrow<Symbol>>(&self, key: B, value: Value) -> Result<(), Value> {
        if let Some(var) = self.get(key) {
            var.set(value);
            Ok(())
        } else {
            Err(value)
        }
    }

    pub fn define<I: Into<Symbol>>(&mut self, key: I, value: Value) {
        let key = key.into();

        if matches!(&key, Symbol::Gensym(env, _) if *env == (self as *const _ as usize))
            || matches!(&key, Symbol::Name(_))
        {
            if let Some(var) = self.storage.get(&key).cloned() {
                var.set(value);
            } else {
                self.storage.insert(key, Var::new(value));
            }
        }
    }

    pub fn generate(&mut self) -> Symbol {
        let sym = Symbol::Gensym(self as *mut _ as usize, self.gensym);
        self.gensym += 1;
        sym
    }
}

#[derive(Clone)]
pub struct Environment(Rc<RefCell<EnvironmentRepr>>);

impl Environment {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(EnvironmentRepr {
            parent: None,
            gensym: 0,
            storage: HashMap::new(),
        })))
    }

    pub fn child(&self) -> Self {
        Self(Rc::new(RefCell::new(EnvironmentRepr {
            parent: Some(Rc::clone(&self.0)),
            gensym: 0,
            storage: HashMap::new(),
        })))
    }

    #[inline]
    pub fn get<B: Borrow<Symbol>>(&self, key: B) -> Option<Var> {
        RefCell::borrow(&*self.0).get(key)
    }

    #[inline]
    pub fn set<B: Borrow<Symbol>>(&self, key: B, value: Value) -> Result<(), Value> {
        RefCell::borrow(&*self.0).set(key, value)
    }

    #[inline]
    pub fn define<I: Into<Symbol>>(&self, key: I, value: Value) {
        RefCell::borrow_mut(&*self.0).define(key, value)
    }

    #[inline]
    pub fn generate(&self) -> Symbol {
        RefCell::borrow_mut(&*self.0).generate()
    }
}

impl Default for Environment {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Environment {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Environment {}

impl Hash for Environment {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0.as_ptr() as usize).hash(state);
    }
}

impl fmt::Debug for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#<environment {:x}>", self.0.as_ptr() as usize)
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#<environment {:x}>", self.0.as_ptr() as usize)
    }
}
