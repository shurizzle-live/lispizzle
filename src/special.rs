use im_rc::{vector, Vector};

use crate::{environment::Bag, eval, Context, Environment, Error, Str, Symbol, Value};

use std::mem;

pub const NAMES: &[&str] = [
    "apply",
    "quote",
    "quasiquote",
    "unquote",
    "unquote-splicing",
    "if",
    "def",
    "set!",
    "current-environment",
    "let",
    "let*",
    "letrec",
    "letrec*",
    "begin",
]
.as_slice();

pub fn transform_fn<T, F>(
    ctx: Context,
    env: Environment,
    name: Str,
    args: Vector<Value>,
    in_block: bool,
    apply_fn: F,
) -> Option<Result<T, Error>>
where
    T: From<Value>,
    F: (Fn(Value, Context, Vector<Value>) -> Result<T, Error>) + Clone,
{
    match name.as_str() {
        "apply" => Some(apply(ctx, args, apply_fn)),
        "quote" => Some(quote(ctx, env, args).map(Into::into)),
        "quasiquote" => Some(quasiquote(ctx, env, args).map(Into::into)),
        "if" => Some(iff(ctx, env, args, apply_fn)),
        "def" => Some(def(ctx, env, args, in_block).map(Into::into)),
        "set!" => Some(set_em_(ctx, env, args, in_block).map(Into::into)),
        "current-environment" => {
            Some(current_environment(ctx, env, args, in_block).map(Into::into))
        }
        "let" => Some(r#let(ctx, env, args, apply_fn)),
        "let*" => Some(r#let_star_(ctx, env, args, apply_fn)),
        "letrec" => Some(r#letrec(ctx, env, args, apply_fn)),
        "letrec*" => Some(r#letrec_star_(ctx, env, args, apply_fn)),
        "begin" => Some(begin(ctx, env, args, apply_fn)),
        _ => None,
    }
}

#[inline(always)]
fn unshift(args: &mut Vector<Value>) -> Value {
    unsafe { args.pop_front().unwrap_unchecked() }
}

fn apply<T, F>(ctx: Context, mut args: Vector<Value>, apply: F) -> Result<T, Error>
where
    T: From<Value>,
    F: (FnOnce(Value, Context, Vector<Value>) -> Result<T, Error>) + Clone,
{
    if args.len() != 2 {
        return Err(ctx.trace().error("syntax-error", None));
    }

    let f = args.remove(0);
    if let Value::List(args) = args.remove(0) {
        apply(f, ctx, args)
    } else {
        Err(ctx.trace().error("wrong-type-arg", None))
    }
}

fn quote(ctx: Context, _env: Environment, mut args: Vector<Value>) -> Result<Value, Error> {
    if args.len() == 1 {
        Ok(unshift(&mut args))
    } else {
        Err(ctx.trace().error("syntax-error", None))
    }
}

fn quasiquote(ctx: Context, env: Environment, mut args: Vector<Value>) -> Result<Value, Error> {
    if args.len() != 1 {
        return Err(ctx.trace().error("syntax-error", None));
    }

    enum Res {
        Value(Value),
        Splice(Vector<Value>),
    }

    fn scan(ctx: Context, env: Environment, v: Value) -> Result<Res, Error> {
        if let Value::List(mut list) = v {
            if let Some(Value::Symbol(Symbol::Name(name))) = list.get(0) {
                if name == "unquote" {
                    return if list.len() == 2 {
                        list.remove(1).eval(ctx, env, false).map(Res::Value)
                    } else {
                        Err(ctx.trace().error("syntax-error", None))
                    };
                } else if name == "unquote-splicing" {
                    return if list.len() == 2 {
                        list.remove(1).eval(ctx, env, false).map(|x| {
                            if let Value::List(l) = x {
                                Res::Splice(l)
                            } else {
                                Res::Value(x)
                            }
                        })
                    } else {
                        Err(ctx.trace().error("syntax-error", None))
                    };
                }
            }

            let mut i = 0;
            while i < list.len() {
                let mut v = Value::Nil;
                mem::swap(&mut v, &mut list[i]);
                match scan(ctx.clone(), env.clone(), v)? {
                    Res::Value(mut v) => {
                        mem::swap(&mut list[i], &mut v);
                        i += 1;
                    }
                    Res::Splice(values) => {
                        list.remove(i);
                        for v in values.into_iter() {
                            list.insert(i, v);
                            i += 1;
                        }
                    }
                }
            }

            Ok(Res::Value(Value::List(list)))
        } else {
            Ok(Res::Value(v))
        }
    }

    match scan(ctx.clone(), env, unshift(&mut args))? {
        Res::Value(v) => Ok(v),
        _ => Err(ctx.trace().error("syntax-error", None)),
    }
}

fn iff<T, F>(ctx: Context, env: Environment, mut args: Vector<Value>, apply: F) -> Result<T, Error>
where
    T: From<Value>,
    F: (Fn(Value, Context, Vector<Value>) -> Result<T, Error>) + Clone,
{
    if args.len() == 2 {
        if eval::value(args.remove(0), ctx.clone(), env.clone(), false)?.to_bool() {
            eval::value_fn(args.remove(0), ctx, env, false, apply)
        } else {
            Ok(Value::Unspecified.into())
        }
    } else if args.len() == 3 {
        if unshift(&mut args)
            .eval(ctx.clone(), env.clone(), false)?
            .to_bool()
        {
            eval::value_fn(args.remove(0), ctx, env, false, apply)
        } else {
            eval::value_fn(args.remove(1), ctx, env, false, apply)
        }
    } else {
        Err(ctx.trace().error("syntax-error", None))
    }
}

fn set_em_(
    ctx: Context,
    env: Environment,
    mut args: Vector<Value>,
    _in_block: bool,
) -> Result<Value, Error> {
    if args.len() == 2 {
        let key = if let Value::Symbol(s) = unshift(&mut args) {
            s
        } else {
            return Err(ctx.trace().error("syntax-error", None));
        };

        let value = unshift(&mut args).eval(ctx.clone(), env.clone(), false)?;
        if env.set(&key, value).is_ok() {
            Ok(Value::Unspecified)
        } else {
            Err(ctx
                .trace()
                .error("unbound-variable", vector![key.into()].into()))
        }
    } else {
        Err(ctx.trace().error("syntax-error", None))
    }
}

fn current_environment(
    ctx: Context,
    env: Environment,
    args: Vector<Value>,
    _in_block: bool,
) -> Result<Value, Error> {
    if args.is_empty() {
        Ok(Value::Environment(env))
    } else {
        Err(ctx.trace().error("syntax-error", None))
    }
}

fn def(
    ctx: Context,
    env: Environment,
    mut args: Vector<Value>,
    in_block: bool,
) -> Result<Value, Error> {
    if !in_block {
        return Err(ctx.trace().error("syntax-error", None));
    }

    let (name, exp) = match args.len() {
        1 | 2 => (
            unsafe { args.pop_front().unwrap_unchecked() },
            args.pop_front().unwrap_or(Value::Unspecified),
        ),
        _ => return Err(ctx.trace().error("syntax-error", None)),
    };

    let name = if let Value::Symbol(sym) = name {
        sym
    } else {
        return Err(ctx.trace().error("syntax-error", None));
    };

    let mut value = exp.eval(ctx, env.clone(), true)?;

    match value {
        Value::Fn(ref mut p) | Value::Macro(ref mut p) => {
            p.set_name(name.clone());
        }
        _ => (),
    }

    env.define(name, value);

    Ok(Value::Unspecified)
}

fn r#let<T, F>(
    ctx: Context,
    env: Environment,
    mut args: Vector<Value>,
    apply: F,
) -> Result<T, Error>
where
    T: From<Value>,
    F: (Fn(Value, Context, Vector<Value>) -> Result<T, Error>) + Clone,
{
    if args.len() < 2 {
        return Err(ctx.trace().error("syntax-error", None));
    }

    let mut bindings = if let Value::List(b) = args.remove(0) {
        b
    } else {
        return Err(ctx.trace().error("syntax-error", None));
    };

    if bindings.len() % 2 != 0 {
        return Err(ctx.trace().error("syntax-error", None));
    }

    let block_env = env.child::<Symbol, _>([]);
    while !bindings.is_empty() {
        let name = if let Value::Symbol(sym) = bindings.remove(0) {
            sym
        } else {
            return Err(ctx.trace().error("syntax-error", None));
        };

        let value = bindings
            .remove(0)
            .macroexpand(ctx.clone(), env.clone(), false)?
            .eval(ctx.clone(), env.clone(), false)?;

        block_env.define(name, value);
    }

    eval::block_fn(&args, ctx, block_env, apply)
}

fn r#let_star_<T, F>(
    ctx: Context,
    env: Environment,
    mut args: Vector<Value>,
    apply: F,
) -> Result<T, Error>
where
    T: From<Value>,
    F: (Fn(Value, Context, Vector<Value>) -> Result<T, Error>) + Clone,
{
    if args.len() < 2 {
        return Err(ctx.trace().error("syntax-error", None));
    }

    let mut bindings = if let Value::List(b) = args.remove(0) {
        b
    } else {
        return Err(ctx.trace().error("syntax-error", None));
    };

    if bindings.len() % 2 != 0 {
        return Err(ctx.trace().error("syntax-error", None));
    }

    let block_env = env.child::<Symbol, _>([]);
    while !bindings.is_empty() {
        let name = if let Value::Symbol(sym) = bindings.remove(0) {
            sym
        } else {
            return Err(ctx.trace().error("syntax-error", None));
        };

        let value = bindings
            .remove(0)
            .macroexpand(ctx.clone(), block_env.clone(), false)?
            .eval(ctx.clone(), block_env.clone(), false)?;

        block_env.define(name, value);
    }

    eval::block_fn(&args, ctx, block_env, apply)
}

fn r#letrec<T, F>(
    ctx: Context,
    env: Environment,
    mut args: Vector<Value>,
    apply: F,
) -> Result<T, Error>
where
    T: From<Value>,
    F: (Fn(Value, Context, Vector<Value>) -> Result<T, Error>) + Clone,
{
    if args.len() < 2 {
        return Err(ctx.trace().error("syntax-error", None));
    }

    let mut bindings = if let Value::List(b) = args.remove(0) {
        b
    } else {
        return Err(ctx.trace().error("syntax-error", None));
    };

    if bindings.len() % 2 != 0 {
        return Err(ctx.trace().error("syntax-error", None));
    }

    let mut total = Bag::new();
    let block_env = env.child::<Symbol, _>([]);
    while !bindings.is_empty() {
        let name = if let Value::Symbol(sym) = bindings.remove(0) {
            sym
        } else {
            return Err(ctx.trace().error("syntax-error", None));
        };

        block_env.define(name.clone(), Value::Unspecified);

        let value = bindings
            .remove(0)
            .macroexpand(ctx.clone(), block_env.clone(), false)?
            .eval(ctx.clone(), block_env.clone(), false)?;

        _ = block_env.set(name, value);
        total.merge(unsafe { block_env.take_bag() });
    }
    unsafe { block_env.set_bag(total) };

    eval::block_fn(&args, ctx, block_env, apply)
}

fn r#letrec_star_<T, F>(
    ctx: Context,
    env: Environment,
    mut args: Vector<Value>,
    apply: F,
) -> Result<T, Error>
where
    T: From<Value>,
    F: (Fn(Value, Context, Vector<Value>) -> Result<T, Error>) + Clone,
{
    if args.len() < 2 {
        return Err(ctx.trace().error("syntax-error", None));
    }

    let mut bindings = if let Value::List(b) = args.remove(0) {
        b
    } else {
        return Err(ctx.trace().error("syntax-error", None));
    };

    if bindings.len() % 2 != 0 {
        return Err(ctx.trace().error("syntax-error", None));
    }

    let block_env = env.child::<Symbol, _>([]);
    while !bindings.is_empty() {
        let name = if let Value::Symbol(sym) = bindings.remove(0) {
            sym
        } else {
            return Err(ctx.trace().error("syntax-error", None));
        };

        block_env.define(name.clone(), Value::Unspecified);

        let value = bindings
            .remove(0)
            .macroexpand(ctx.clone(), block_env.clone(), false)?
            .eval(ctx.clone(), block_env.clone(), false)?;

        _ = block_env.set(name, value);
    }

    eval::block_fn(&args, ctx, block_env, apply)
}

#[inline]
fn begin<T, F>(ctx: Context, env: Environment, exprs: Vector<Value>, apply: F) -> Result<T, Error>
where
    T: From<Value>,
    F: (Fn(Value, Context, Vector<Value>) -> Result<T, Error>) + Clone,
{
    eval::block_fn(&exprs, ctx, env, apply)
}
