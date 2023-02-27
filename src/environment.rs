use std::{
    borrow::Borrow,
    cell::{Ref, RefCell},
    fmt,
    hash::Hash,
    ops::{AddAssign, Neg, SubAssign},
    rc::Rc,
};

use ecow::EcoString;
use im_rc::{HashMap, Vector};

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
    fn default() -> Self {
        use crate::{Lambda, Parameters};
        use rug::Integer;
        use std::num::NonZeroUsize;

        let me = Self::new();

        fn define<F: (Fn(Environment, Vector<Value>) -> Value) + 'static>(
            env: &Environment,
            name: &str,
            ps: Parameters<usize, NonZeroUsize>,
            doc: Option<&str>,
            f: F,
        ) {
            let mut lambda = Lambda::from_native(ps, doc.map(|s| s.into()), f);
            let name: EcoString = name.into();
            lambda.set_name(name.clone());
            env.define(Symbol::Name(name), lambda.into());
        }

        define(
            &me,
            "+", 
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
            Some("Return the sum of all parameter values. Return 0 if called without any parameters."),
            |_env, values| match values.len() {
                0 => Integer::from(0).into(),
                1 => values[0].clone(),
                _ => {
                    let mut acc = match &values[0] {
                        Value::Integer(i) => i.clone(),
                        _ => return Value::Nil,
                    };

                    for v in values.iter().skip(1) {
                        match v {
                            Value::Integer(i) => acc.add_assign(i.clone()),
                            _ => return Value::Nil,
                        }
                    }

                    acc.into()
                }
            });

        define(
            &me,
            "-",
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
            Some("If called with one argument Z1, -Z1 returned. Otherwise the sum of all but the first \
                argument are subtracted from the first argument."),
            |_env, values| match values.len() {
                0 => Value::Nil,
                1 => match &values[0] {
                    Value::Integer(i) => i.clone().neg().into(),
                    _ => Value::Nil,
                },
                _ => {
                    let mut acc = match &values[0] {
                        Value::Integer(i) => i.clone(),
                        _ => return Value::Nil,
                    };

                    for v in values.iter().skip(1) {
                        match v {
                            Value::Integer(i) => acc.sub_assign(i.clone()),
                            _ => return Value::Nil,
                        }
                    }

                    acc.into()
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
                Value::Nil
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
                Value::Nil
            },
        );

        define(
            &me,
            "list",
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
            Some("Create a list."),
            |_env, values| values.into(),
        );

        define(
            &me,
            "string->symbol",
            Parameters::Exact(1),
            Some("Return the symbol whose name is STRING."),
            |_env, values| match values[0] {
                Value::String(ref s) => Value::Symbol(Symbol::Name(s.clone())),
                _ => Value::Nil,
            },
        );

        define(
            &me,
            "current-environment",
            Parameters::Exact(0),
            Some("Return the current environment."),
            |env, _| Value::Environment(env),
        );

        define(
            &me,
            "eval",
            Parameters::Exact(2),
            Some("Evaluate expression in the given environment."),
            |_env, values| match (&values[0], &values[1]) {
                (Value::List(_), Value::Environment(env)) => values[0].eval(env.clone()),
                _ => Value::Nil,
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
            |_env, values| {
                if let Value::Lambda(ref p) = values[0] {
                    p.doc().into()
                } else {
                    Value::Nil
                }
            },
        );

        define(
            &me,
            "procedure-name",
            Parameters::Exact(1),
            Some("Return the name of the procedure."),
            |_env, values| {
                if let Value::Lambda(ref p) = values[0] {
                    p.name().into()
                } else {
                    Value::Nil
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
                .apply(env.clone(), vector![]),
            Integer::from(0).into()
        );

        assert_eq!(
            env.get(Symbol::Name("+".into()))
                .unwrap()
                .get()
                .apply(env.clone(), vector![Integer::from(69).into()]),
            Integer::from(69).into()
        );

        assert_eq!(
            env.get(Symbol::Name("+".into()))
                .unwrap()
                .get()
                .apply(env.clone(), vector![Value::String("ciao".into())]),
            Value::String("ciao".into())
        );

        assert_eq!(
            env.get(Symbol::Name("+".into())).unwrap().get().apply(
                env.clone(),
                vector![Integer::from(34).into(), Integer::from(35).into()]
            ),
            Integer::from(69).into()
        );

        assert_eq!(
            env.get(Symbol::Name("+".into())).unwrap().get().apply(
                env,
                vector![Integer::from(69).into(), Value::String("ciao".into())]
            ),
            Value::Nil
        );
    }

    #[test]
    fn list() {
        let env = Environment::default();

        let l = vector![1.into(), 2.into(), 3.into()];
        assert_eq!(
            env.get(Symbol::Name("list".into()))
                .unwrap()
                .get()
                .apply(env, l.clone()),
            l.into()
        );
    }
}
