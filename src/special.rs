use im_rc::{vector, Vector};

use crate::{BackTrace, Environment, Error, Str, Symbol, Value};

use std::mem;

pub fn transform(
    trace: BackTrace,
    env: Environment,
    name: Str,
    args: Vector<Value>,
) -> Option<Result<Value, Error>> {
    match name.as_str() {
        "quote" => Some(quote(trace, env, args)),
        "quasiquote" => Some(quasiquote(trace, env, args)),
        "if" => Some(iff(trace, env, args)),
        "set!" => Some(set_em_(trace, env, args)),
        "current-environment" => Some(current_environment(trace, env, args)),
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

fn quote(trace: BackTrace, _env: Environment, mut args: Vector<Value>) -> Result<Value, Error> {
    if args.len() == 1 {
        Ok(unshift(&mut args))
    } else {
        Err(trace.error("syntax-error", None))
    }
}

fn quasiquote(trace: BackTrace, env: Environment, mut args: Vector<Value>) -> Result<Value, Error> {
    if args.len() != 1 {
        return Err(trace.error("syntax-error", None));
    }

    enum Res {
        Value(Value),
        Splice(Vector<Value>),
    }

    fn scan(trace: BackTrace, env: Environment, v: Value) -> Result<Res, Error> {
        if let Value::List(mut list) = v {
            if let Some(Value::Symbol(Symbol::Name(name))) = list.get(0) {
                if name == "unquote" {
                    return if list.len() == 2 {
                        grab(&mut list, 1).eval(trace, env).map(Res::Value)
                    } else {
                        Err(trace.error("syntax-error", None))
                    };
                } else if name == "unquote-splicing" {
                    return if list.len() == 2 {
                        grab(&mut list, 1).eval(trace, env).map(|x| {
                            if let Value::List(l) = x {
                                Res::Splice(l)
                            } else {
                                Res::Value(x)
                            }
                        })
                    } else {
                        Err(trace.error("syntax-error", None))
                    };
                }
            }

            let mut i = 0;
            while i < list.len() {
                let mut v = Value::Nil;
                mem::swap(&mut v, &mut list[i]);
                match scan(trace.clone(), env.clone(), v)? {
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

    match scan(trace.clone(), env, unshift(&mut args))? {
        Res::Value(v) => Ok(v),
        _ => Err(trace.error("syntax-error", None)),
    }
}

fn iff(trace: BackTrace, env: Environment, mut args: Vector<Value>) -> Result<Value, Error> {
    if args.len() == 2 {
        if unshift(&mut args)
            .eval(trace.clone(), env.clone())?
            .to_bool()
        {
            unshift(&mut args).eval(trace, env)
        } else {
            Ok(Value::Unspecified)
        }
    } else if args.len() == 3 {
        if unshift(&mut args)
            .eval(trace.clone(), env.clone())?
            .to_bool()
        {
            grab(&mut args, 0).eval(trace, env)
        } else {
            grab(&mut args, 1).eval(trace, env)
        }
    } else {
        Err(trace.error("syntax-error", None))
    }
}

fn set_em_(trace: BackTrace, env: Environment, mut args: Vector<Value>) -> Result<Value, Error> {
    if args.len() == 2 {
        let key = if let Value::Symbol(s) = unshift(&mut args) {
            s
        } else {
            return Err(trace.error("syntax-error", None));
        };

        let value = unshift(&mut args).eval(trace.clone(), env.clone())?;
        if env.set(&key, value).is_ok() {
            Ok(Value::Unspecified)
        } else {
            Err(trace.error("unbound-variable", vector![key.into()].into()))
        }
    } else {
        Err(trace.error("syntax-error", None))
    }
}

fn current_environment(
    trace: BackTrace,
    env: Environment,
    args: Vector<Value>,
) -> Result<Value, Error> {
    if args.is_empty() {
        Ok(Value::Environment(env))
    } else {
        Err(trace.error("syntax-error", None))
    }
}
