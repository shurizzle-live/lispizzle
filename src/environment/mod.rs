mod bag;

use std::{
    borrow::Borrow,
    cell::{Ref, RefCell, RefMut},
    fmt,
    hash::Hash,
    ops::{AddAssign, Neg, SubAssign},
    rc::Rc,
};

use im_rc::Vector;

use crate::{BackTrace, Error, Str, Symbol, TraceFrame, Value, Var};

use bag::Bag;

struct EnvironmentRepr {
    parent: Option<Rc<RefCell<EnvironmentRepr>>>,
    gensym: usize,
    bag: Bag,
    trace: Option<TraceFrame>,
}

impl EnvironmentRepr {
    #[inline]
    fn parent(&self) -> Option<Ref<EnvironmentRepr>> {
        self.parent.as_ref().map(|r| RefCell::borrow(r))
    }

    #[inline]
    fn parent_mut(&self) -> Option<RefMut<EnvironmentRepr>> {
        self.parent.as_ref().map(|r| RefCell::borrow_mut(r))
    }

    pub fn get<B: Borrow<Symbol>>(&self, key: B) -> Option<Var> {
        let key = key.borrow();

        match key {
            Symbol::Gensym(env, _) if *env == (self as *const _ as usize) => self.bag.get(key),
            Symbol::Name(_) => self
                .bag
                .get(key)
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
        if let Some(mut p) = self.parent_mut() {
            return p.define(key, value);
        }

        let key = key.into();

        if matches!(&key, Symbol::Gensym(env, _) if *env == (self as *const _ as usize))
            || matches!(&key, Symbol::Name(_))
        {
            if let Some(var) = self.bag.get(&key) {
                var.set(value);
            } else {
                self.bag.insert(key, Var::new(value));
            }
        }
    }

    pub fn generate(&mut self) -> Symbol {
        let sym = Symbol::Gensym(self as *mut _ as usize, self.gensym);
        self.gensym += 1;
        sym
    }

    #[inline]
    pub fn trace(&self) -> Option<TraceFrame> {
        self.trace.clone()
    }

    fn write_trace(&self, bt: &mut BackTrace) {
        if let Some(trace) = self.trace() {
            bt.push_back(trace);
        }
        if let Some(parent) = self.parent() {
            parent.write_trace(bt);
        }
    }

    #[inline]
    pub fn backtrace(&self) -> BackTrace {
        let mut backtrace = BackTrace::new();
        self.write_trace(&mut backtrace);
        backtrace
    }
}

#[derive(Clone)]
pub struct Environment(Rc<RefCell<EnvironmentRepr>>);

impl Environment {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(EnvironmentRepr {
            parent: None,
            gensym: 0,
            bag: Bag::new(),
            trace: Some(TraceFrame::main()),
        })))
    }

    fn _child<S, I>(&self, names: I, trace: Option<TraceFrame>) -> Self
    where
        S: Into<Symbol>,
        I: IntoIterator<Item = S>,
    {
        Self(Rc::new(RefCell::new(EnvironmentRepr {
            parent: Some(Rc::clone(&self.0)),
            gensym: 0,
            bag: names.into_iter().collect(),
            trace,
        })))
    }

    #[inline]
    pub fn child<S, I>(&self, names: I) -> Self
    where
        S: Into<Symbol>,
        I: IntoIterator<Item = S>,
    {
        self._child(names, None)
    }

    #[inline]
    pub fn with_trace<S, I>(&self, names: I, trace: TraceFrame) -> Self
    where
        S: Into<Symbol>,
        I: IntoIterator<Item = S>,
    {
        self._child(names, Some(trace))
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

    #[inline]
    pub fn trace(&self) -> Option<TraceFrame> {
        RefCell::borrow(&*self.0).trace()
    }

    #[inline]
    pub fn backtrace(&self) -> BackTrace {
        RefCell::borrow(&*self.0).backtrace()
    }

    #[inline]
    pub fn backtrace_from(&self, bt: &mut BackTrace) {
        RefCell::borrow(&*self.0).write_trace(bt)
    }

    #[inline]
    pub fn error<S: Into<Str>>(&self, name: S, args: Option<Vector<Value>>) -> Error {
        Error::new(name.into(), args, self.backtrace())
    }
}

impl Default for Environment {
    fn default() -> Self {
        use crate::{Parameters, Proc};
        use rug::Integer;
        use std::num::NonZeroUsize;

        let me = Self::new();

        fn define<F, S1, S2>(
            env: &Environment,
            name: S1,
            ps: Parameters<usize, NonZeroUsize>,
            doc: Option<S2>,
            r#macro: bool,
            f: F,
        ) where
            F: (Fn(Environment, Vector<Value>) -> Result<Value, Error>) + 'static,
            S1: Into<Str>,
            S2: Into<Str>,
        {
            let mut lambda = Proc::from_native(ps, doc.map(|s| s.into()), r#macro, f);
            let name: Str = name.into();
            lambda.set_name(name.clone());
            env.define(Symbol::Name(name), lambda.into());
        }

        fn define_fn<F, S1, S2>(
            env: &Environment,
            name: S1,
            ps: Parameters<usize, NonZeroUsize>,
            doc: Option<S2>,
            f: F,
        ) where
            F: (Fn(Environment, Vector<Value>) -> Result<Value, Error>) + 'static,
            S1: Into<Str>,
            S2: Into<Str>,
        {
            define(env, name, ps, doc, false, f)
        }

        #[allow(dead_code)]
        fn define_macro<F, S1, S2>(
            env: &Environment,
            name: S1,
            ps: Parameters<usize, NonZeroUsize>,
            doc: Option<S2>,
            f: F,
        ) where
            F: (Fn(Environment, Vector<Value>) -> Result<Value, Error>) + 'static,
            S1: Into<Str>,
            S2: Into<Str>,
        {
            define(env, name, ps, doc, true, f)
        }

        define_fn(
            &me,
            "+", 
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
            Some("Return the sum of all parameter values. Return 0 if called without any parameters."),
            |env, mut values| match values.len() {
                0 => Ok(Integer::from(0).into()),
                1 => Ok(unsafe { values.into_iter().next().unwrap_unchecked() }),
                _ => {
                    let mut acc = match unsafe { values.pop_front().unwrap_unchecked() } {
                        Value::Integer(i) => i,
                        _ => return Err(env.error("wrong-type-arg", None))
                    };

                    for v in values.into_iter() {
                        match v {
                            Value::Integer(i) => acc.add_assign(i),
                        _ => return Err(env.error("wrong-type-arg", None))
                        }
                    }

                    Ok(acc.into())
                }
            });

        define_fn(
            &me,
            "-",
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
            Some("If called with one argument Z1, -Z1 returned. Otherwise the sum of all but the first \
                argument are subtracted from the first argument."),
            |env, mut values| match values.len() {
                0 => unreachable!(),
                1 => match unsafe { values.pop_front().unwrap_unchecked() } {
                    Value::Integer(i) => Ok(i.neg().into()),
                    _ => Err(env.error("wrong-type-arg", None)),
                },
                _ => {
                    let mut acc = match unsafe { values.pop_front().unwrap_unchecked() } {
                        Value::Integer(i) => i,
                        _ => return Err(env.error("wrong-type-arg", None)),
                    };

                    for v in values.into_iter() {
                        match v {
                            Value::Integer(i) => acc.sub_assign(i),
                            _ => return Err(env.error("wrong-type-arg", None)),
                        }
                    }

                    Ok(acc.into())
                }
            },
        );

        define_fn(
            &me,
            "print",
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
            Some("Print arguments"),
            |_env, values| {
                for (i, v) in values.into_iter().enumerate() {
                    if i == 0 {
                        print!("{v}");
                    } else {
                        print!(" {v}");
                    }
                }
                Ok(Value::Nil)
            },
        );

        define_fn(
            &me,
            "println",
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
            Some("Print arguments followed by a newline"),
            |_env, values| {
                for (i, v) in values.into_iter().enumerate() {
                    if i == 0 {
                        print!("{v}");
                    } else {
                        print!(" {v}");
                    }
                }
                println!();
                Ok(Value::Nil)
            },
        );

        define_fn(
            &me,
            "list",
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
            Some("Create a list."),
            |_env, values| Ok(values.into()),
        );

        define_fn(
            &me,
            "string->symbol",
            Parameters::Exact(1),
            Some("Return the symbol whose name is STRING."),
            |env, mut values| match unsafe { values.pop_front().unwrap_unchecked() } {
                Value::String(s) => Ok(Value::Symbol(Symbol::Name(s))),
                _ => Err(env.error("wrong-type-arg", None)),
            },
        );

        define_fn(
            &me,
            "current-environment",
            Parameters::Exact(0),
            Some("Return the current environment."),
            |env, _| Ok(Value::Environment(env)),
        );

        define_fn(
            &me,
            "eval",
            Parameters::Exact(2),
            Some("Evaluate expression in the given environment."),
            |env, mut values| {
                let l = unsafe { values.pop_front().unwrap_unchecked() };
                match (&l, unsafe { values.pop_front().unwrap_unchecked() }) {
                    (Value::List(_), Value::Environment(env)) => l.eval(env),
                    _ => Err(env.error("wrong-type-arg", None)),
                }
            },
        );

        define_fn(
            &me,
            "primitive-eval",
            Parameters::Exact(1),
            Some("Evaluate expression in the current environment."),
            |env, mut values| unsafe { values.pop_front().unwrap_unchecked() }.eval(env),
        );

        define_fn(
            &me,
            "procedure-documentation",
            Parameters::Exact(1),
            Some("Return the documentation string associated with `proc'."),
            |env, mut values| {
                if let Value::Proc(p) = unsafe { values.pop_front().unwrap_unchecked() } {
                    Ok(p.doc().map(Value::from).unwrap_or(Value::Boolean(false)))
                } else {
                    Err(env.error("wrong-type-arg", None))
                }
            },
        );

        define_fn(
            &me,
            "procedure-name",
            Parameters::Exact(1),
            Some("Return the name of the procedure."),
            |env, mut values| {
                if let Value::Proc(p) = unsafe { values.pop_front().unwrap_unchecked() } {
                    Ok(p.name().into())
                } else {
                    Err(env.error("wrong-type-arg", None))
                }
            },
        );

        define_fn(
            &me,
            "apply",
            Parameters::Exact(2),
            Option::<&str>::None,
            |env, mut values| {
                let f = unsafe { values.pop_front().unwrap_unchecked() };
                if let Value::List(args) = unsafe { values.pop_front().unwrap_unchecked() } {
                    f.apply(env, args)
                } else {
                    Err(env.error("wrong-type-arg", None))
                }
            },
        );

        me
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

    use crate::{Environment, Symbol, Value};

    #[test]
    fn plus() {
        let env = Environment::default();

        assert_eq!(
            env.get(Symbol::Name("+".into()))
                .unwrap()
                .get()
                .apply(env.clone(), vector![])
                .unwrap(),
            Integer::from(0).into()
        );

        assert_eq!(
            env.get(Symbol::Name("+".into()))
                .unwrap()
                .get()
                .apply(env.clone(), vector![Integer::from(69).into()])
                .unwrap(),
            Integer::from(69).into()
        );

        assert_eq!(
            env.get(Symbol::Name("+".into()))
                .unwrap()
                .get()
                .apply(env.clone(), vector![Value::String("ciao".into())])
                .unwrap(),
            Value::String("ciao".into())
        );

        assert_eq!(
            env.get(Symbol::Name("+".into()))
                .unwrap()
                .get()
                .apply(
                    env.clone(),
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
                env,
                vector![Integer::from(69).into(), Value::String("ciao".into())]
            )
            .is_err());
    }

    #[test]
    fn list() {
        let env = Environment::default();

        let l = vector![1.into(), 2.into(), 3.into()];
        assert_eq!(
            env.get(Symbol::Name("list".into()))
                .unwrap()
                .get()
                .apply(env, l.clone())
                .unwrap(),
            l.into()
        );
    }
}
