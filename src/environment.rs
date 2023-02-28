use std::{
    borrow::Borrow,
    cell::{Ref, RefCell},
    fmt,
    hash::Hash,
    ops::{AddAssign, Neg, SubAssign},
    rc::Rc,
};

use im_rc::{HashMap, Vector};

use crate::{BackTrace, Error, Str, Symbol, TraceFrame, Value, Var};

struct EnvironmentRepr {
    parent: Option<Rc<RefCell<EnvironmentRepr>>>,
    gensym: usize,
    storage: HashMap<Symbol, Var>,
    trace: Option<TraceFrame>,
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
            storage: HashMap::new(),
            trace: Some(TraceFrame::main()),
        })))
    }

    fn _child(&self, trace: Option<TraceFrame>) -> Self {
        Self(Rc::new(RefCell::new(EnvironmentRepr {
            parent: Some(Rc::clone(&self.0)),
            gensym: 0,
            storage: HashMap::new(),
            trace,
        })))
    }

    #[inline]
    pub fn child(&self) -> Self {
        self._child(None)
    }

    #[inline]
    pub fn with_trace(&self, trace: TraceFrame) -> Self {
        self._child(Some(trace))
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
        use crate::{Lambda, Parameters};
        use rug::Integer;
        use std::num::NonZeroUsize;

        let me = Self::new();

        fn define<F, S1, S2>(
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
            let mut lambda = Lambda::from_native(ps, doc.map(|s| s.into()), f);
            let name: Str = name.into();
            lambda.set_name(name.clone());
            env.define(Symbol::Name(name), lambda.into());
        }

        define(
            &me,
            "+", 
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
            Some("Return the sum of all parameter values. Return 0 if called without any parameters."),
            |env, values| match values.len() {
                0 => Ok(Integer::from(0).into()),
                1 => Ok(values[0].clone()),
                _ => {
                    let mut acc = match &values[0] {
                        Value::Integer(i) => i.clone(),
                        _ => return Err(env.error("wrong-type-arg", None))
                    };

                    for v in values.iter().skip(1) {
                        match v {
                            Value::Integer(i) => acc.add_assign(i.clone()),
                        _ => return Err(env.error("wrong-type-arg", None))
                        }
                    }

                    Ok(acc.into())
                }
            });

        define(
            &me,
            "-",
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
            Some("If called with one argument Z1, -Z1 returned. Otherwise the sum of all but the first \
                argument are subtracted from the first argument."),
            |env, values| match values.len() {
                0 => unreachable!(),
                1 => match &values[0] {
                    Value::Integer(i) => Ok(i.clone().neg().into()),
                    _ => Err(env.error("wrong-type-arg", None)),
                },
                _ => {
                    let mut acc = match &values[0] {
                        Value::Integer(i) => i.clone(),
                        _ => return Err(env.error("wrong-type-arg", None)),
                    };

                    for v in values.iter().skip(1) {
                        match v {
                            Value::Integer(i) => acc.sub_assign(i.clone()),
                            _ => return Err(env.error("wrong-type-arg", None)),
                        }
                    }

                    Ok(acc.into())
                }
            },
        );

        define(
            &me,
            "print",
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
            Some("Print arguments"),
            |_env, values| {
                for (i, v) in values.iter().enumerate() {
                    if i == 0 {
                        print!("{v}");
                    } else {
                        print!(" {v}");
                    }
                }
                Ok(Value::Nil)
            },
        );

        define(
            &me,
            "println",
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
            Some("Print arguments followed by a newline"),
            |_env, values| {
                for (i, v) in values.iter().enumerate() {
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

        define(
            &me,
            "list",
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
            Some("Create a list."),
            |_env, values| Ok(values.into()),
        );

        define(
            &me,
            "string->symbol",
            Parameters::Exact(1),
            Some("Return the symbol whose name is STRING."),
            |env, values| match values[0] {
                Value::String(ref s) => Ok(Value::Symbol(Symbol::Name(s.clone()))),
                _ => Err(env.error("wrong-type-arg", None)),
            },
        );

        define(
            &me,
            "current-environment",
            Parameters::Exact(0),
            Some("Return the current environment."),
            |env, _| Ok(Value::Environment(env)),
        );

        define(
            &me,
            "eval",
            Parameters::Exact(2),
            Some("Evaluate expression in the given environment."),
            |env, values| match (&values[0], &values[1]) {
                (Value::List(_), Value::Environment(env)) => values[0].eval(env.clone()),
                _ => Err(env.error("wrong-type-arg", None)),
            },
        );

        define(
            &me,
            "primitive-eval",
            Parameters::Exact(1),
            Some("Evaluate expression in the current environment."),
            |env, values| values[0].eval(env),
        );

        define(
            &me,
            "procedure-documentation",
            Parameters::Exact(1),
            Some("Return the documentation string associated with `proc'."),
            |env, values| {
                if let Value::Lambda(ref p) = values[0] {
                    Ok(p.doc().into())
                } else {
                    Err(env.error("wrong-type-arg", None))
                }
            },
        );

        define(
            &me,
            "procedure-name",
            Parameters::Exact(1),
            Some("Return the name of the procedure."),
            |env, values| {
                if let Value::Lambda(ref p) = values[0] {
                    Ok(p.name().into())
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
