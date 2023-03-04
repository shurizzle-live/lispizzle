use std::num::NonZeroUsize;

use super::util::define_fn;
use crate::{proc::Parameters, Environment, Value};

pub fn add(me: &Environment) {
    define_fn(
        me,
        "list",
        Parameters::Variadic(unsafe { NonZeroUsize::new_unchecked(1) }),
        Some("Create a list."),
        |_ctx, values| Ok(values.into()),
    );

    define_fn(
        me,
        "list?",
        Parameters::Exact(1),
        Option::<&str>::None,
        |_ctx, mut values| {
            let x = values.remove(0);
            Ok(x.is_list().into())
        },
    );

    define_fn(
        me,
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

    define_fn(
        me,
        "length",
        Parameters::Exact(1),
        Some("Return the number of elements in list LST."),
        |ctx, mut values| {
            if let Value::List(l) = values.remove(0) {
                Ok(l.len().into())
            } else {
                Err(ctx.trace().error("wrong-type-arg", None))
            }
        },
    );
}
