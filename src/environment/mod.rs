mod bag;
mod default;
mod proc;

use std::{
    borrow::Borrow,
    cell::{Ref, RefCell, RefMut},
    fmt,
    hash::Hash,
    mem,
    rc::Rc,
};

use crate::{Symbol, Value, Var};

pub use bag::Bag;

struct Repr {
    parent: Option<Rc<RefCell<Repr>>>,
    bag: Bag,
}

impl Repr {
    #[inline]
    fn parent(&self) -> Option<Ref<Repr>> {
        self.parent.as_ref().map(|r| RefCell::borrow(r))
    }

    #[inline]
    #[allow(dead_code)]
    fn parent_mut(&self) -> Option<RefMut<Repr>> {
        self.parent.as_ref().map(|r| RefCell::borrow_mut(r))
    }

    pub fn get<B: Borrow<Symbol>>(&self, key: B) -> Option<Var> {
        let key = key.borrow();
        self.bag
            .get(key)
            .or_else(|| self.parent().and_then(|p| p.get(key)))
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

        if let Some(var) = self.bag.get(&key) {
            var.set(value);
        } else {
            self.bag.insert(key, Var::new(value));
        }
    }

    #[inline]
    pub fn is_toplevel(&self) -> bool {
        self.parent.is_none()
    }

    #[must_use]
    #[inline]
    pub unsafe fn take_bag(&mut self) -> Bag {
        let mut bag = Bag::new();
        mem::swap(&mut bag, &mut self.bag);
        bag
    }

    #[must_use]
    #[inline]
    pub unsafe fn set_bag(&mut self, mut other: Bag) -> Bag {
        mem::swap(&mut other, &mut self.bag);
        other
    }
}

#[inline(always)]
unsafe fn parent_ptr(current: *const Rc<RefCell<Repr>>) -> Option<*const Rc<RefCell<Repr>>> {
    RefCell::borrow(&**current)
        .parent
        .as_ref()
        .map(|x| x as *const _)
}

unsafe fn toplevel(mut current: *const Rc<RefCell<Repr>>) -> Rc<RefCell<Repr>> {
    while let Some(parent) = parent_ptr(current) {
        current = parent;
    }
    Rc::clone(&*current)
}

#[derive(Clone)]
pub struct Environment(Rc<RefCell<Repr>>);

impl Environment {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(Repr {
            parent: None,
            bag: Bag::new(),
        })))
    }

    pub fn child<S, I>(&self, names: I) -> Self
    where
        S: Into<Symbol>,
        I: IntoIterator<Item = S>,
    {
        Self(Rc::new(RefCell::new(Repr {
            parent: Some(Rc::clone(&self.0)),
            bag: names.into_iter().collect(),
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
    pub fn toplevel(&self) -> Self {
        Self(unsafe { toplevel(&self.0) })
    }

    #[inline]
    pub fn is_toplevel(&self) -> bool {
        RefCell::borrow(&*self.0).is_toplevel()
    }

    /// # Safety
    /// It's unsafe to change the current scope but we need it for letrec
    #[inline]
    pub unsafe fn take_bag(&self) -> Bag {
        RefCell::borrow_mut(&*self.0).take_bag()
    }

    /// # Safety
    /// It's unsafe to change the current scope but we need it for letrec
    #[inline]
    pub unsafe fn set_bag(&self, other: Bag) -> Bag {
        RefCell::borrow_mut(&*self.0).set_bag(other)
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

#[cfg(test)]
mod tests {
    use im_rc::vector;
    use rug::Integer;

    use crate::{Context, Environment, Symbol, Value};

    #[test]
    fn plus() {
        let ctx = Context::new();
        let env = Environment::default();

        assert_eq!(
            env.get(Symbol::Name("+".into()))
                .unwrap()
                .get()
                .apply(ctx.clone(), vector![])
                .unwrap(),
            Integer::from(0).into()
        );

        assert_eq!(
            env.get(Symbol::Name("+".into()))
                .unwrap()
                .get()
                .apply(ctx.clone(), vector![Integer::from(69).into()])
                .unwrap(),
            Integer::from(69).into()
        );

        assert_eq!(
            env.get(Symbol::Name("+".into()))
                .unwrap()
                .get()
                .apply(ctx.clone(), vector![Value::String("ciao".into())])
                .unwrap(),
            Value::String("ciao".into())
        );

        assert_eq!(
            env.get(Symbol::Name("+".into()))
                .unwrap()
                .get()
                .apply(
                    ctx.clone(),
                    vector![Integer::from(34).into(), Integer::from(35).into()]
                )
                .unwrap(),
            Integer::from(69).into()
        );

        assert!(env
            .get(Symbol::Name("+".into()))
            .unwrap()
            .get()
            .apply(
                ctx,
                vector![Integer::from(69).into(), Value::String("ciao".into())]
            )
            .is_err());
    }

    #[test]
    fn list() {
        let ctx = Context::new();
        let env = Environment::default();

        let l = vector![1.into(), 2.into(), 3.into()];
        assert_eq!(
            env.get(Symbol::Name("list".into()))
                .unwrap()
                .get()
                .apply(ctx, l.clone())
                .unwrap(),
            l.into()
        );
    }

    #[test]
    fn toplevel() {
        let env = Environment::new();
        let child = env.child::<Symbol, _>([]).child::<Symbol, _>([]);
        assert_eq!(env, child.toplevel());
    }
}
