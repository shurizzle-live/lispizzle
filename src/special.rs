use im_rc::{vector, Vector};

use crate::{Context, Environment, Error, Str, Symbol, Value};

use std::mem;

pub fn transform(
    ctx: Context,
    env: Environment,
    name: Str,
    args: Vector<Value>,
) -> Option<Result<Value, Error>> {
    match name.as_str() {
        "quote" => Some(quote(ctx, env, args)),
        "quasiquote" => Some(quasiquote(ctx, env, args)),
        "if" => Some(iff(ctx, env, args)),
        "set!" => Some(set_em_(ctx, env, args)),
        "current-environment" => Some(current_environment(ctx, env, args)),
        _ => None,
    }
}

#[inline(always)]
fn unshift(args: &mut Vector<Value>) -> Value {
    unsafe { args.pop_front().unwrap_unchecked() }
}

#[inline(always)]
fn grab(args: &mut Vector<Value>, n: usize) -> Value {
    args.remove(n)
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
                        grab(&mut list, 1).eval(ctx, env).map(Res::Value)
                    } else {
                        Err(ctx.trace().error("syntax-error", None))
                    };
                } else if name == "unquote-splicing" {
                    return if list.len() == 2 {
                        grab(&mut list, 1).eval(ctx, env).map(|x| {
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
                    }
                    Res::Splice(values) => {
                        list.remove(i);
                        for (off, v) in values.into_iter().enumerate() {
                            list.insert(i + off, v);
                        }
                    }
                }
                i += 1;
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

fn iff(ctx: Context, env: Environment, mut args: Vector<Value>) -> Result<Value, Error> {
    if args.len() == 2 {
        if unshift(&mut args).eval(ctx.clone(), env.clone())?.to_bool() {
            unshift(&mut args).eval(ctx, env)
        } else {
            Ok(Value::Unspecified)
        }
    } else if args.len() == 3 {
        if unshift(&mut args).eval(ctx.clone(), env.clone())?.to_bool() {
            grab(&mut args, 0).eval(ctx, env)
        } else {
            grab(&mut args, 1).eval(ctx, env)
        }
    } else {
        Err(ctx.trace().error("syntax-error", None))
    }
}

fn set_em_(ctx: Context, env: Environment, mut args: Vector<Value>) -> Result<Value, Error> {
    if args.len() == 2 {
        let key = if let Value::Symbol(s) = unshift(&mut args) {
            s
        } else {
            return Err(ctx.trace().error("syntax-error", None));
        };

        let value = unshift(&mut args).eval(ctx.clone(), env.clone())?;
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
) -> Result<Value, Error> {
    if args.is_empty() {
        Ok(Value::Environment(env))
    } else {
        Err(ctx.trace().error("syntax-error", None))
    }
}
