use std::fmt;

use im_rc::Vector;

use crate::{Context, Environment, Error, Value};

pub fn print_list_debug<T>(
    f: &mut fmt::Formatter<'_>,
    iiter: impl IntoIterator<Item = T>,
    lh: impl fmt::Display,
    rh: impl fmt::Display,
) -> fmt::Result
where
    T: fmt::Debug,
{
    fmt::Display::fmt(&lh, f)?;

    let mut it = iiter.into_iter();
    if let Some(e) = it.next() {
        fmt::Debug::fmt(&e, f)?;

        for e in it {
            write!(f, " {:?}", e)?;
        }
    }

    fmt::Display::fmt(&rh, f)
}

pub fn print_list_display<T>(
    f: &mut fmt::Formatter<'_>,
    iiter: impl IntoIterator<Item = T>,
    lh: impl fmt::Display,
    rh: impl fmt::Display,
) -> fmt::Result
where
    T: fmt::Display,
{
    fmt::Display::fmt(&lh, f)?;

    let mut it = iiter.into_iter();
    if let Some(e) = it.next() {
        fmt::Display::fmt(&e, f)?;

        for e in it {
            write!(f, " {}", e)?;
        }
    }

    fmt::Display::fmt(&rh, f)
}

pub fn eval_block(exprs: &Vector<Value>, ctx: Context, env: Environment) -> Result<Value, Error> {
    if exprs.is_empty() {
        return Err(ctx.trace().error("syntax-error", None));
    }

    fn eval_exp(exp: &Value, ctx: Context, env: Environment) -> Result<Value, Error> {
        exp.clone()
            .macroexpand(ctx.clone(), env.clone(), true)?
            .eval(ctx, env, true)
    }

    let last = exprs.len() - 1;
    for exp in exprs.iter().take(last) {
        eval_exp(exp, ctx.clone(), env.clone())?;
    }

    eval_exp(unsafe { exprs.get(last).unwrap_unchecked() }, ctx, env)
}
