use std::ops::{AddAssign, Neg, SubAssign};

use im_rc::{vector, Vector};

use crate::{environment::proc, Context, Environment, Error, Str, Symbol, Value};

impl Default for Environment {
    fn default() -> Self {
        use crate::proc::{Parameters, Proc};
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
            F: (Fn(Context, Vector<Value>) -> Result<Value, Error>) + 'static,
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
            F: (Fn(Context, Vector<Value>) -> Result<Value, Error>) + 'static,
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
            |ctx, mut values| match values.len() {
                0 => Ok(Integer::from(0).into()),
                1 => Ok(unsafe { values.into_iter().next().unwrap_unchecked() }),
                _ => {
                    let mut acc = match unsafe { values.pop_front().unwrap_unchecked() } {
                        Value::Integer(i) => i,
                        _ => return Err(ctx.trace().error("wrong-type-arg", None))
                    };

                    for v in values.into_iter() {
                        match v {
                            Value::Integer(i) => acc.add_assign(i),
                        _ => return Err(ctx.trace().error("wrong-type-arg", None))
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
            |ctx, mut values| match values.len() {
                0 => unreachable!(),
                1 => match unshift(&mut values) {
                    Value::Integer(i) => Ok(i.neg().into()),
                    _ => Err(ctx.trace().error("wrong-type-arg", None)),
                },
                _ => {
                    let mut acc = match unshift(&mut values) {
                        Value::Integer(i) => i,
                        _ => return Err(ctx.trace().error("wrong-type-arg", None)),
                    };

                    for v in values.into_iter() {
                        match v {
                            Value::Integer(i) => acc.sub_assign(i),
                            _ => return Err(ctx.trace().error("wrong-type-arg", None)),
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
            |_ctx, values| {
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
            |_ctx, values| {
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
            |_ctx, values| Ok(values.into()),
        );

        define_fn(
            &me,
            "string->symbol",
            Parameters::Exact(1),
            Some("Return the symbol whose name is STRING."),
            |ctx, mut values| match unsafe { values.pop_front().unwrap_unchecked() } {
                Value::String(s) => Ok(Value::Symbol(Symbol::Name(s))),
                _ => Err(ctx.trace().error("wrong-type-arg", None)),
            },
        );

        define_fn(
            &me,
            "eval",
            Parameters::Exact(2),
            Some("Evaluate expression in the given environment."),
            |ctx, mut values| {
                let l = unshift(&mut values);
                match (&l, unshift(&mut values)) {
                    (Value::List(_), Value::Environment(env)) => l
                        .macroexpand(ctx.clone(), env.clone(), true)?
                        .eval(ctx, env, true),
                    _ => Err(ctx.trace().error("wrong-type-arg", None)),
                }
            },
        );

        define_fn(
            &me,
            "fn-doc",
            Parameters::Exact(1),
            Some("Return the documentation string associated with `fn'."),
            |ctx, mut values| {
                if let Value::Fn(p) = unshift(&mut values) {
                    Ok(p.doc().map(Value::from).unwrap_or(Value::Boolean(false)))
                } else {
                    Err(ctx.trace().error("wrong-type-arg", None))
                }
            },
        );

        define_fn(
            &me,
            "fn-name",
            Parameters::Exact(1),
            Some("Return the name of the fn."),
            |ctx, mut values| {
                if let Value::Fn(p) = unshift(&mut values) {
                    Ok(p.name().into())
                } else {
                    Err(ctx.trace().error("wrong-type-arg", None))
                }
            },
        );

        define_fn(
            &me,
            "fn-source",
            Parameters::Exact(1),
            Some("Return the source of the fn."),
            |ctx, mut values| {
                if let Value::Fn(p) = unshift(&mut values) {
                    Ok(p.source())
                } else {
                    Err(ctx.trace().error("wrong-type-arg", None))
                }
            },
        );

        define_fn(
            &me,
            "macro-doc",
            Parameters::Exact(1),
            Some("Return the documentation string associated with `macro'."),
            |ctx, mut values| {
                if let Value::Macro(p) = unshift(&mut values) {
                    Ok(p.doc().map(Value::from).unwrap_or(Value::Boolean(false)))
                } else {
                    Err(ctx.trace().error("wrong-type-arg", None))
                }
            },
        );

        define_fn(
            &me,
            "macro-name",
            Parameters::Exact(1),
            Some("Return the name of the macro."),
            |ctx, mut values| {
                if let Value::Macro(p) = unshift(&mut values) {
                    Ok(p.name().into())
                } else {
                    Err(ctx.trace().error("wrong-type-arg", None))
                }
            },
        );

        define_fn(
            &me,
            "macro-source",
            Parameters::Exact(1),
            Some("Return the source of the fn."),
            |ctx, mut values| {
                if let Value::Macro(p) = unshift(&mut values) {
                    Ok(p.source())
                } else {
                    Err(ctx.trace().error("wrong-type-arg", None))
                }
            },
        );

        define_fn(
            &me,
            "apply",
            Parameters::Exact(2),
            Option::<&str>::None,
            |ctx, mut values| {
                let f = unshift(&mut values);
                if let Value::List(args) = unshift(&mut values) {
                    f.apply(ctx, args)
                } else {
                    Err(ctx.trace().error("wrong-type-arg", None))
                }
            },
        );

        define_fn(
            &me,
            "eq?",
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
            Option::<&str>::None,
            |_ctx, mut values| {
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
            |ctx, mut values| {
                if let Some(first) = values.pop_front() {
                    while let Some(other) = values.pop_front() {
                        if core::mem::discriminant(&first) != core::mem::discriminant(&other) {
                            return Err(ctx.trace().error("wrong-type-arg", None));
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
            |_ctx, mut values| {
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
            |_ctx, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_nil().into())
            },
        );

        define_fn(
            &me,
            "string?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_string().into())
            },
        );

        define_fn(
            &me,
            "bool?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_boolean().into())
            },
        );

        define_fn(
            &me,
            "char?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_character().into())
            },
        );

        define_fn(
            &me,
            "int?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_integer().into())
            },
        );

        define_fn(
            &me,
            "sym?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_symbol().into())
            },
        );

        define_fn(
            &me,
            "fn?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_fn().into())
            },
        );

        define_fn(
            &me,
            "macro?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_macro().into())
            },
        );

        define_fn(
            &me,
            "list?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_list().into())
            },
        );

        define_fn(
            &me,
            "var?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_var().into())
            },
        );

        define_fn(
            &me,
            "env?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_environment().into())
            },
        );

        define_fn(
            &me,
            "error?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_error().into())
            },
        );

        define_fn(
            &me,
            "backtrace?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_backtrace().into())
            },
        );

        define_fn(
            &me,
            "frame?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
                let x = unshift(&mut values);
                Ok(x.is_frame().into())
            },
        );

        define_macro(
            &me,
            "primitive-eval",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
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
            "$macroexpand",
            Parameters::Exact(2),
            Option::<&str>::None,
            |ctx, mut values| {
                let env = if let Value::Environment(env) = values.remove(1) {
                    env
                } else {
                    return Err(ctx.trace().error("wrong-type-arg", None));
                };
                values.remove(0).macroexpand(ctx, env, true)
            },
        );

        define_macro(
            &me,
            "macroexpand",
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(2) }),
            Option::<&str>::None,
            |ctx, mut values| {
                let env = match values.len() {
                    1 => vector![Symbol::Name("current-environment".into()).into()].into(),
                    2 => values.remove(1),
                    _ => return Err(ctx.trace().error("wrong-number-of-args", None)),
                };

                Ok(vector![
                    Symbol::Name("$macroexpand".into()).into(),
                    values.remove(0),
                    env
                ]
                .into())
            },
        );

        define_fn(
            &me,
            "backtrace",
            Parameters::Exact(0),
            Option::<&str>::None,
            |ctx, _values| Ok(ctx.trace().parent().into()),
        );

        define_fn(
            &me,
            "throw",
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(2) }),
            Option::<&str>::None,
            |ctx, mut values| {
                let args = match values.len() {
                    1 => None,
                    2 => {
                        if let Value::List(list) = values.remove(1) {
                            Some(list)
                        } else {
                            return Err(ctx.trace().error("wrong-type-arg", None));
                        }
                    }
                    _ => return Err(ctx.trace().error("wrong-number-of-args", None)),
                };

                let name = if let Value::Symbol(Symbol::Name(str)) = unshift(&mut values) {
                    str
                } else {
                    return Err(ctx.trace().error("wrong-type-arg", None));
                };

                Err(unsafe { ctx.trace().parent().unwrap_unchecked() }.error(name, args))
            },
        );

        define_fn(
            &me,
            "catch-all",
            Parameters::Exact(2),
            Option::<&str>::None,
            |ctx, mut values| {
                let f1 = values.remove(0);
                let f2 = values.remove(0);

                if !f1.is_fn() || !f2.is_fn() {
                    return Err(ctx.trace().error("wrong-type-arg", None));
                }

                let err = match f1.apply(ctx.clone(), vector![]) {
                    Ok(v) => return Ok(v),
                    Err(err) => err,
                };

                f2.apply(ctx, vector![err.into()])
            },
        );

        define_fn(
            &me,
            "string-length",
            Parameters::Exact(1),
            Option::<&str>::None,
            |ctx, mut values| {
                if let Value::String(s) = values.remove(0) {
                    Ok(s.len().into())
                } else {
                    Err(ctx.trace().error("wrong-type-arg", None))
                }
            },
        );

        define_fn(
            &me,
            "substring",
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(3) }),
            Option::<&str>::None,
            |mut ctx, mut values| {
                #[inline(always)]
                fn string(ctx: &Context, v: Value) -> Result<Str, Error> {
                    if let Value::String(str) = v {
                        Ok(str)
                    } else {
                        Err(ctx.trace().error("wrong-type-arg", None))
                    }
                }

                #[inline]
                fn index(ctx: &Context, v: Value) -> Result<usize, Error> {
                    if let Value::Integer(i) = v {
                        if let Some(i) = i.to_usize() {
                            Ok(i)
                        } else {
                            Err(ctx.trace().error("out-of-range", None))
                        }
                    } else {
                        Err(ctx.trace().error("wrong-type-arg", None))
                    }
                }

                let (s, start, len) = match values.len() {
                    2 => (
                        string(&ctx, values.remove(0))?,
                        index(&ctx, values.remove(0))?,
                        None,
                    ),
                    3 => (
                        string(&ctx, values.remove(0))?,
                        index(&ctx, values.remove(0))?,
                        Some(index(&ctx, values.remove(0))?),
                    ),
                    _ => return Err(ctx.trace().error("wrong-number-of-args", None)),
                };

                s.substring_in_context(&mut ctx, start, len)
                    .map(Into::into)
                    .ok_or_else(|| ctx.trace().error("out-of-range", None))
            },
        );

        define_macro(
            &me,
            "fn",
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(3) }),
            Option::<&str>::None,
            |ctx, args| {
                let mut source = args.clone();
                source.push_front(Value::Symbol(Str::from("fn").into()));
                proc::proc_macro(ctx, Some(source), args).map(Value::UnboundFn)
            },
        );

        define_macro(
            &me,
            "macro",
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(3) }),
            Option::<&str>::None,
            |ctx, args| {
                let mut source = args.clone();
                source.push_front(Value::Symbol(Str::from("macro").into()));
                proc::proc_macro(ctx, Some(source), args).map(Value::UnboundMacro)
            },
        );

        me
    }
}
