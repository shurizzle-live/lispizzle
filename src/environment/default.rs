use std::ops::{AddAssign, Neg, SubAssign};

use im_rc::{vector, Vector};

use crate::{BackTrace, Environment, Error, Str, Symbol, Value};

impl Default for Environment {
    fn default() -> Self {
        use crate::{Parameters, Proc};
        use rug::Integer;
        use std::num::NonZeroUsize;

        let me = Self::new();

        fn define_fn<F, S1, S2>(
            env: &Environment,
            name: S1,
            ps: Parameters<usize, NonZeroUsize>,
            doc: Option<S2>,
            f: F,
        ) where
            F: (Fn(BackTrace, Vector<Value>) -> Result<Value, Error>) + 'static,
            S1: Into<Str>,
            S2: Into<Str>,
        {
            let mut lambda = Proc::from_native(ps, doc.map(|s| s.into()), f);
            let name: Str = name.into();
            lambda.set_name(name.clone());
            env.define(Symbol::Name(name), Value::Fn(lambda));
        }

        #[allow(dead_code)]
        fn define_macro<F, S1, S2>(
            env: &Environment,
            name: S1,
            ps: Parameters<usize, NonZeroUsize>,
            doc: Option<S2>,
            f: F,
        ) where
            F: (Fn(BackTrace, Vector<Value>) -> Result<Value, Error>) + 'static,
            S1: Into<Str>,
            S2: Into<Str>,
        {
            let mut lambda = Proc::from_native(ps, doc.map(|s| s.into()), f);
            let name: Str = name.into();
            lambda.set_name(name.clone());
            env.define(Symbol::Name(name), Value::Macro(lambda));
        }

        fn unshift(v: &mut Vector<Value>) -> Value {
            unsafe { v.pop_front().unwrap_unchecked() }
        }

        define_fn(
            &me,
            "+", 
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
            Some("Return the sum of all parameter values. Return 0 if called without any parameters."),
            |trace, mut values| match values.len() {
                0 => Ok(Integer::from(0).into()),
                1 => Ok(unsafe { values.into_iter().next().unwrap_unchecked() }),
                _ => {
                    let mut acc = match unsafe { values.pop_front().unwrap_unchecked() } {
                        Value::Integer(i) => i,
                        _ => return Err(trace.error("wrong-type-arg", None))
                    };

                    for v in values.into_iter() {
                        match v {
                            Value::Integer(i) => acc.add_assign(i),
                        _ => return Err(trace.error("wrong-type-arg", None))
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
            |trace, mut values| match values.len() {
                0 => unreachable!(),
                1 => match unshift(&mut values) {
                    Value::Integer(i) => Ok(i.neg().into()),
                    _ => Err(trace.error("wrong-type-arg", None)),
                },
                _ => {
                    let mut acc = match unshift(&mut values) {
                        Value::Integer(i) => i,
                        _ => return Err(trace.error("wrong-type-arg", None)),
                    };

                    for v in values.into_iter() {
                        match v {
                            Value::Integer(i) => acc.sub_assign(i),
                            _ => return Err(trace.error("wrong-type-arg", None)),
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
            |_trace, values| {
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
            |_trace, values| {
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
            |_trace, values| Ok(values.into()),
        );

        define_fn(
            &me,
            "string->symbol",
            Parameters::Exact(1),
            Some("Return the symbol whose name is STRING."),
            |trace, mut values| match unsafe { values.pop_front().unwrap_unchecked() } {
                Value::String(s) => Ok(Value::Symbol(Symbol::Name(s))),
                _ => Err(trace.error("wrong-type-arg", None)),
            },
        );

        define_fn(
            &me,
            "eval",
            Parameters::Exact(2),
            Some("Evaluate expression in the given environment."),
            |trace, mut values| {
                let l = unshift(&mut values);
                match (&l, unshift(&mut values)) {
                    (Value::List(_), Value::Environment(env)) => l.eval(trace, env),
                    _ => Err(trace.error("wrong-type-arg", None)),
                }
            },
        );

        define_fn(
            &me,
            "procedure-documentation",
            Parameters::Exact(1),
            Some("Return the documentation string associated with `proc'."),
            |trace, mut values| {
                if let Value::Fn(p) = unshift(&mut values) {
                    Ok(p.doc().map(Value::from).unwrap_or(Value::Boolean(false)))
                } else {
                    Err(trace.error("wrong-type-arg", None))
                }
            },
        );

        define_fn(
            &me,
            "procedure-name",
            Parameters::Exact(1),
            Some("Return the name of the procedure."),
            |trace, mut values| {
                if let Value::Fn(p) = unshift(&mut values) {
                    Ok(p.name().into())
                } else {
                    Err(trace.error("wrong-type-arg", None))
                }
            },
        );

        define_fn(
            &me,
            "apply",
            Parameters::Exact(2),
            Option::<&str>::None,
            |trace, mut values| {
                let f = unshift(&mut values);
                if let Value::List(args) = unshift(&mut values) {
                    f.apply(trace, args)
                } else {
                    Err(trace.error("wrong-type-arg", None))
                }
            },
        );

        define_fn(
            &me,
            "eq?",
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
            Option::<&str>::None,
            |_trace, mut values| {
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
            |trace, mut values| {
                if let Some(first) = values.pop_front() {
                    while let Some(other) = values.pop_front() {
                        if core::mem::discriminant(&first) != core::mem::discriminant(&other) {
                            return Err(trace.error("wrong-type-arg", None));
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

        define_fn(
            &me,
            "begin",
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
            Option::<&str>::None,
            |_trace, mut values| {
                if let Some(last) = values.pop_back() {
                    Ok(last)
                } else {
                    Ok(Value::Unspecified)
                }
            },
        );

        define_fn(
            &me,
            "nil?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_trace, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_nil().into())
            },
        );

        define_fn(
            &me,
            "string?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_trace, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_string().into())
            },
        );

        define_fn(
            &me,
            "bool?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_trace, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_boolean().into())
            },
        );

        define_fn(
            &me,
            "char?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_trace, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_character().into())
            },
        );

        define_fn(
            &me,
            "int?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_trace, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_integer().into())
            },
        );

        define_fn(
            &me,
            "sym?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_trace, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_symbol().into())
            },
        );

        define_fn(
            &me,
            "fn?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_trace, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_fn().into())
            },
        );

        define_fn(
            &me,
            "macro?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_trace, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_macro().into())
            },
        );

        define_fn(
            &me,
            "list?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_trace, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_list().into())
            },
        );

        define_fn(
            &me,
            "var?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_trace, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_var().into())
            },
        );

        define_fn(
            &me,
            "env?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_trace, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_environment().into())
            },
        );

        define_fn(
            &me,
            "error?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_trace, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_error().into())
            },
        );

        define_fn(
            &me,
            "backtrace?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_trace, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_backtrace().into())
            },
        );

        define_fn(
            &me,
            "frame?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_trace, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_frame().into())
            },
        );

        define_macro(
            &me,
            "primitive-eval",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_trace, mut values| {
                Ok(vector![
                    Symbol::Name("eval".into()).into(),
                    values.remove(0),
                    vector![Symbol::Name("current-environment".into()).into()].into()
                ]
                .into())
            },
        );

        define_fn(
            &me,
            "backtrace",
            Parameters::Exact(0),
            Option::<&str>::None,
            |trace, _values| Ok(trace.parent().into()),
        );

        me
    }
}
