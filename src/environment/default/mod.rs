mod numbers;
mod procs;
mod strings;
mod util;

use im_rc::vector;

use crate::{environment::proc, eval, Environment, Symbol, Value};

impl Default for Environment {
    fn default() -> Self {
        use crate::proc::Parameters;
        use std::num::NonZeroUsize;
        use util::{define_fn, define_macro};

        let me = Self::new();

        numbers::add(&me);
        strings::add(&me);
        procs::add(&me);

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
            "eval",
            Parameters::Exact(2),
            Some("Evaluate expression in the given environment."),
            |ctx, mut values| {
                let l = values.remove(0);
                match (&l, values.remove(0)) {
                    (Value::List(_), Value::Environment(env)) => l
                        .macroexpand(ctx.clone(), env.clone(), true)?
                        .eval(ctx, env, true),
                    _ => Err(ctx.trace().error("wrong-type-arg", None)),
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
                }
                Ok(true.into())
            },
        );

        define_fn(
            &me,
            "not",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| Ok((!values.remove(0).to_bool()).into()),
        );

        define_fn(
            &me,
            "not=",
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
            Option::<&str>::None,
            |ctx, mut values| {
                if let Some(first) = values.pop_front() {
                    while let Some(other) = values.pop_front() {
                        if core::mem::discriminant(&first) != core::mem::discriminant(&other) {
                            return Err(ctx.trace().error("wrong-type-arg", None));
                        }

                        if first.eq(&other) {
                            return Ok(false.into());
                        }
                    }
                }
                Ok(true.into())
            },
        );

        define_fn(
            &me,
            "nil?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
                let x = values.remove(0);
                Ok(x.is_nil().into())
            },
        );

        define_fn(
            &me,
            "bool?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
                let x = values.remove(0);
                Ok(x.is_boolean().into())
            },
        );

        define_fn(
            &me,
            "char?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
                let x = values.remove(0);
                Ok(x.is_character().into())
            },
        );

        define_fn(
            &me,
            "sym?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
                let x = values.remove(0);
                Ok(x.is_symbol().into())
            },
        );

        define_fn(
            &me,
            "list?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
                let x = values.remove(0);
                Ok(x.is_list().into())
            },
        );

        define_fn(
            &me,
            "var?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
                let x = values.remove(0);
                Ok(x.is_var().into())
            },
        );

        define_fn(
            &me,
            "env?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
                let x = values.remove(0);
                Ok(x.is_environment().into())
            },
        );

        define_fn(
            &me,
            "error?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
                let x = values.remove(0);
                Ok(x.is_error().into())
            },
        );

        define_fn(
            &me,
            "backtrace?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
                let x = values.remove(0);
                Ok(x.is_backtrace().into())
            },
        );

        define_fn(
            &me,
            "frame?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |_ctx, mut values| {
                let x = values.remove(0);
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

                let name = if let Value::Symbol(Symbol::Name(str)) = values.remove(0) {
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

                let err = match eval::apply(f1, ctx.clone(), vector![]) {
                    Ok(v) => return Ok(v),
                    Err(err) => err,
                };

                eval::apply(f2, ctx, vector![err.into()])
            },
        );

        define_fn(
            &me,
            "null?",
            Parameters::Exact(1),
            Option::<&str>::None,
            |ctx, mut values| {
                if let Value::List(l) = values.remove(0) {
                    Ok(l.is_empty().into())
                } else {
                    Err(ctx.trace().error("wrong-type-arg", None))
                }
            },
        );

        me
    }
}
