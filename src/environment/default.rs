use std::ops::{AddAssign, Neg, SubAssign};

use im_rc::Vector;

use crate::{Environment, Error, Str, Symbol, Value};

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

        fn unshift(v: &mut Vector<Value>) -> Value {
            unsafe { v.pop_front().unwrap_unchecked() }
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
                1 => match unshift(&mut values) {
                    Value::Integer(i) => Ok(i.neg().into()),
                    _ => Err(env.error("wrong-type-arg", None)),
                },
                _ => {
                    let mut acc = match unshift(&mut values) {
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
                let l = unshift(&mut values);
                match (&l, unshift(&mut values)) {
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
            |env, mut values| unshift(&mut values).eval(env),
        );

        define_fn(
            &me,
            "procedure-documentation",
            Parameters::Exact(1),
            Some("Return the documentation string associated with `proc'."),
            |env, mut values| {
                if let Value::Proc(p) = unshift(&mut values) {
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
                if let Value::Proc(p) = unshift(&mut values) {
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
                let f = unshift(&mut values);
                if let Value::List(args) = unshift(&mut values) {
                    f.apply(env, args)
                } else {
                    Err(env.error("wrong-type-arg", None))
                }
            },
        );

        define_fn(
            &me,
            "eq?",
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
            Option::<&str>::None,
            |_env, mut values| {
                if let Some(first) = values.pop_front() {
                    while let Some(other) = values.pop_front() {
                        if first.ne(&other) {
                            return Ok(false.into());
                        }
                    }
                    Ok(true.into())
                } else {
                    Ok(true.into())
                }
            },
        );

        define_fn(
            &me,
            "=",
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
            Option::<&str>::None,
            |env, mut values| {
                if let Some(first) = values.pop_front() {
                    while let Some(other) = values.pop_front() {
                        if core::mem::discriminant(&first) != core::mem::discriminant(&other) {
                            return Err(env.error("wrong-type-arg", None));
                        }

                        if first.ne(&other) {
                            return Ok(false.into());
                        }
                    }
                    Ok(true.into())
                } else {
                    Ok(true.into())
                }
            },
        );

        me
    }
}
