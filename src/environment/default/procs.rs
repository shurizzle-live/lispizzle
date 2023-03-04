use std::num::NonZeroUsize;

use im_rc::vector;

use super::util::{define_fn, define_macro};
use crate::{proc::Parameters, Environment, Str, Symbol, Value};

pub fn add(me: &Environment) {
    define_fn(
        me,
        "fn-doc",
        Parameters::Exact(1),
        Some("Return the documentation string associated with `fn'."),
        |ctx, mut values| {
            if let Value::Fn(p) = values.remove(0) {
                Ok(p.doc().map(Value::from).unwrap_or(Value::Boolean(false)))
            } else {
                Err(ctx.trace().error("wrong-type-arg", None))
            }
        },
    );

    define_fn(
        me,
        "fn-name",
        Parameters::Exact(1),
        Some("Return the name of the fn."),
        |ctx, mut values| {
            if let Value::Fn(p) = values.remove(0) {
                Ok(p.name().into())
            } else {
                Err(ctx.trace().error("wrong-type-arg", None))
            }
        },
    );

    define_fn(
        me,
        "fn-source",
        Parameters::Exact(1),
        Some("Return the source of the fn."),
        |ctx, mut values| {
            if let Value::Fn(p) = values.remove(0) {
                Ok(p.source())
            } else {
                Err(ctx.trace().error("wrong-type-arg", None))
            }
        },
    );

    define_fn(
        me,
        "macro-doc",
        Parameters::Exact(1),
        Some("Return the documentation string associated with `macro'."),
        |ctx, mut values| {
            if let Value::Macro(p) = values.remove(0) {
                Ok(p.doc().map(Value::from).unwrap_or(Value::Boolean(false)))
            } else {
                Err(ctx.trace().error("wrong-type-arg", None))
            }
        },
    );

    define_fn(
        me,
        "macro-name",
        Parameters::Exact(1),
        Some("Return the name of the macro."),
        |ctx, mut values| {
            if let Value::Macro(p) = values.remove(0) {
                Ok(p.name().into())
            } else {
                Err(ctx.trace().error("wrong-type-arg", None))
            }
        },
    );

    define_fn(
        me,
        "macro-source",
        Parameters::Exact(1),
        Some("Return the source of the fn."),
        |ctx, mut values| {
            if let Value::Macro(p) = values.remove(0) {
                Ok(p.source())
            } else {
                Err(ctx.trace().error("wrong-type-arg", None))
            }
        },
    );

    define_fn(
        me,
        "fn?",
        Parameters::Exact(1),
        Option::<&str>::None,
        |_ctx, mut values| {
            let x = values.remove(0);
            Ok(x.is_fn().into())
        },
    );

    define_fn(
        me,
        "macro?",
        Parameters::Exact(1),
        Option::<&str>::None,
        |_ctx, mut values| {
            let x = values.remove(0);
            Ok(x.is_macro().into())
        },
    );

    define_fn(
        me,
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
        me,
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

    define_macro(
        me,
        "fn",
        Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(3) }),
        Option::<&str>::None,
        |ctx, args| {
            let mut source = args.clone();
            source.push_front(Value::Symbol(Str::from("fn").into()));
            super::proc::proc_macro(ctx, Some(source), args).map(Value::UnboundFn)
        },
    );

    define_macro(
        me,
        "macro",
        Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(3) }),
        Option::<&str>::None,
        |ctx, args| {
            let mut source = args.clone();
            source.push_front(Value::Symbol(Str::from("macro").into()));
            super::proc::proc_macro(ctx, Some(source), args).map(Value::UnboundMacro)
        },
    );

    define_fn(
        me,
        "gensym",
        Parameters::Exact(0),
        Option::<&str>::None,
        |ctx, _args| Ok(ctx.make_sym().into()),
    );

    define_macro(
        me,
        "defmacro",
        Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(2) }),
        Option::<&str>::None,
        |ctx, mut args| {
            let name = args.remove(0);
            let mut source = args.clone();
            source.push_front(Value::Symbol(Str::from("macro").into()));
            let r#macro =
                super::proc::proc_macro(ctx, Some(source), args).map(Value::UnboundMacro)?;
            Ok(vector![Value::Symbol(Symbol::Name(Str::from("def"))), name, r#macro].into())
        },
    );

    define_macro(
        me,
        "defn",
        Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(2) }),
        Option::<&str>::None,
        |ctx, mut args| {
            let name = args.remove(0);
            let mut source = args.clone();
            source.push_front(Value::Symbol(Str::from("fn").into()));
            let r#macro = super::proc::proc_macro(ctx, Some(source), args).map(Value::UnboundFn)?;
            Ok(vector![Value::Symbol(Symbol::Name(Str::from("def"))), name, r#macro].into())
        },
    );
}
