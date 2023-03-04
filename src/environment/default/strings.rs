use std::num::NonZeroUsize;

use super::util::define_fn;
use crate::{proc::Parameters, Context, Environment, Error, Str, Symbol, Value};

pub fn add(me: &Environment) {
    define_fn(
        me,
        "string->sym",
        Parameters::Exact(1),
        Some("Return the symbol whose name is STRING."),
        |ctx, mut values| match values.remove(0) {
            Value::String(s) => Ok(Value::Symbol(Symbol::Name(s))),
            _ => Err(ctx.trace().error("wrong-type-arg", None)),
        },
    );

    define_fn(
        me,
        "sym->string",
        Parameters::Exact(1),
        Some("Return the name of SYMBOL as a string."),
        |mut ctx, mut values| match values.remove(0) {
            Value::Symbol(Symbol::Name(s)) => Ok(Value::String(s)),
            Value::Symbol(Symbol::Gensym(n)) => {
                Ok(Value::String(ctx.make_string(format!("gensym({})", n))))
            }
            _ => Err(ctx.trace().error("wrong-type-arg", None)),
        },
    );

    define_fn(
        me,
        "string?",
        Parameters::Exact(1),
        Option::<&str>::None,
        |_ctx, mut values| {
            let x = values.remove(0);
            Ok(x.is_string().into())
        },
    );

    define_fn(
        me,
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
        me,
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
}
