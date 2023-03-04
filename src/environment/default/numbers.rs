use std::{
    num::NonZeroUsize,
    ops::{AddAssign, DivAssign, MulAssign, Neg, SubAssign},
};

use im_rc::vector;
use rug::Integer;

use super::util::{define_fn, define_macro};
use crate::{proc::Parameters, Environment, Str, Symbol, Value};

pub fn add(me: &Environment) {
    define_fn(
        me,
        "int?",
        Parameters::Exact(1),
        Option::<&str>::None,
        |_ctx, mut values| {
            let x = values.remove(0);
            Ok(x.is_integer().into())
        },
    );

    define_fn(
        me,
        "+",
        Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
        Some("Return the sum of all parameter values. Return 0 if called without any parameters."),
        |ctx, mut values| match values.len() {
            0 => Ok(Integer::from(0).into()),
            1 => Ok(values.remove(0)),
            _ => {
                let mut acc = match values.remove(0) {
                    Value::Integer(i) => i,
                    _ => return Err(ctx.trace().error("wrong-type-arg", None)),
                };

                for v in values {
                    match v {
                        Value::Integer(i) => acc.add_assign(i),
                        _ => return Err(ctx.trace().error("wrong-type-arg", None)),
                    }
                }

                Ok(acc.into())
            }
        },
    );

    define_fn(
        me,
        "-",
        Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
        Some(
            "If called with one argument Z1, -Z1 returned. Otherwise the sum of all but the first \
                argument are subtracted from the first argument.",
        ),
        |ctx, mut values| match values.len() {
            0 => unreachable!(),
            1 => match values.remove(0) {
                Value::Integer(i) => Ok(i.neg().into()),
                _ => Err(ctx.trace().error("wrong-type-arg", None)),
            },
            _ => {
                let mut acc = match values.remove(0) {
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
        me,
        "*",
        Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
        Some("Return the product of all arguments.  If called without arguments, 1 is returned."),
        |ctx, mut values| match values.len() {
            0 => Ok(Integer::from(1).into()),
            1 => Ok(values.remove(0)),
            _ => {
                let mut acc = match values.remove(0) {
                    Value::Integer(i) => i,
                    _ => return Err(ctx.trace().error("wrong-type-arg", None)),
                };

                for v in values {
                    match v {
                        Value::Integer(i) => acc.mul_assign(i),
                        _ => return Err(ctx.trace().error("wrong-type-arg", None)),
                    }
                }

                Ok(acc.into())
            }
        },
    );

    define_fn(
        me,
        "/",
        Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(3) }),
        Some("Divide the first argument by the product of the remaining arguments."),
        |ctx, mut values| {
            let mut acc = if let Value::Integer(i) = values.remove(0) {
                i
            } else {
                return Err(ctx.trace().error("wrong-type-arg", None));
            };

            for v in values {
                let i = if let Value::Integer(i) = v {
                    i
                } else {
                    return Err(ctx.trace().error("wrong-type-arg", None));
                };
                acc.div_assign(i);
            }

            Ok(Value::Integer(acc))
        },
    );

    fn define_cmp<S1, S2, F>(env: &Environment, name: S1, doc: Option<S2>, f: F)
    where
        F: (Fn(&Integer, &Integer) -> bool) + 'static,
        S1: Into<Str>,
        S2: Into<Str>,
    {
        define_fn(
            env,
            name,
            Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
            doc,
            move |ctx, mut values| {
                if let Some(prev) = values.pop_front() {
                    if let Value::Integer(mut prev) = prev {
                        while let Some(next) = values.pop_front() {
                            if let Value::Integer(next) = next {
                                if !f(&prev, &next) {
                                    return Ok(false.into());
                                }
                                prev = next;
                            } else {
                                return Err(ctx.trace().error("wrong-type-arg", None));
                            }
                        }
                    } else {
                        return Err(ctx.trace().error("wrong-type-arg", None));
                    }
                }

                Ok(true.into())
            },
        );
    }

    define_cmp(
        me,
        "<",
        Some("Return `#t' if the list of parameters is monotonically increasing."),
        Integer::lt,
    );

    define_cmp(
        me,
        "<=",
        Some("Return `#t' if the list of parameters is monotonically non-decreasing."),
        Integer::le,
    );

    define_cmp(
        me,
        ">",
        Some("Return `#t' if the list of parameters is monotonically decreasing."),
        Integer::gt,
    );

    define_cmp(
        me,
        ">=",
        Some("Return `#t' if the list of parameters is monotonically non-increasing."),
        Integer::ge,
    );

    define_fn(
        me,
        "1+",
        Parameters::Exact(1),
        Option::<&str>::None,
        |ctx, mut args| {
            if let Value::Integer(mut n) = args.remove(0) {
                n.add_assign(1);
                Ok(n.into())
            } else {
                Err(ctx.trace().error("wrong-type-arg", None))
            }
        },
    );

    define_macro(
        me,
        "inc",
        Parameters::Exact(1),
        Option::<&str>::None,
        |_ctx, mut args| {
            let name = args.remove(0);
            Ok(vector![
                Symbol::Name(Str::from("set!")).into(),
                name.clone(),
                vector![
                    Symbol::Name(Str::from("+")).into(),
                    Value::Integer(1.into()),
                    name,
                ]
                .into()
            ]
            .into())
        },
    );
}
