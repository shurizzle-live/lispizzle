use im_rc::{vector, Vector};

use crate::{Environment, Error, Str, Symbol, Value};

use std::mem;

pub fn transform(env: Environment, name: Str, args: Vector<Value>) -> Option<Result<Value, Error>> {
    match name.as_str() {
        "quote" => Some(quote(env, args)),
        "quasiquote" => Some(quasiquote(env, args)),
        "if" => Some(iff(env, args)),
        "set!" => Some(set_em_(env, args)),
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

fn quote(env: Environment, mut args: Vector<Value>) -> Result<Value, Error> {
    if args.len() == 1 {
        Ok(unshift(&mut args))
    } else {
        Err(env.error("syntax-error", None))
    }
}

fn quasiquote(env: Environment, mut args: Vector<Value>) -> Result<Value, Error> {
    if args.len() != 1 {
        return Err(env.error("syntax-error", None));
    }

    enum Res {
        Value(Value),
        Splice(Vector<Value>),
    }

    fn scan(env: Environment, v: Value) -> Result<Res, Error> {
        if let Value::List(mut list) = v {
            if let Some(Value::Symbol(Symbol::Name(name))) = list.get(0) {
                if name == "unquote" {
                    return if list.len() == 2 {
                        grab(&mut list, 1).eval(env).map(Res::Value)
                    } else {
                        Err(env.error("syntax-error", None))
                    };
                } else if name == "unquote-splicing" {
                    return if list.len() == 2 {
                        grab(&mut list, 1).eval(env).map(|x| {
                            if let Value::List(l) = x {
                                Res::Splice(l)
                            } else {
                                Res::Value(x)
                            }
                        })
                    } else {
                        Err(env.error("syntax-error", None))
                    };
                }
            }

            let mut i = 0;
            while i < list.len() {
                let mut v = Value::Nil;
                mem::swap(&mut v, &mut list[i]);
                match scan(env.clone(), v)? {
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

    match scan(env.clone(), unshift(&mut args))? {
        Res::Value(v) => Ok(v),
        _ => Err(env.error("syntax-error", None)),
    }
}

fn iff(env: Environment, mut args: Vector<Value>) -> Result<Value, Error> {
    if args.len() == 2 {
        if unshift(&mut args).eval(env.clone())?.to_bool() {
            unshift(&mut args).eval(env)
        } else {
            Ok(Value::Unspecified)
        }
    } else if args.len() == 3 {
        if unshift(&mut args).eval(env.clone())?.to_bool() {
            grab(&mut args, 0).eval(env)
        } else {
            grab(&mut args, 1).eval(env)
        }
    } else {
        Err(env.error("syntax-error", None))
    }
}

fn set_em_(env: Environment, mut args: Vector<Value>) -> Result<Value, Error> {
    if args.len() == 2 {
        let key = if let Value::Symbol(s) = unshift(&mut args) {
            s
        } else {
            return Err(env.error("syntax-error", None));
        };

        let value = unshift(&mut args).eval(env.clone())?;
        if env.set(&key, value).is_ok() {
            Ok(Value::Unspecified)
        } else {
            Err(env.error("unbound-variable", vector![key.into()].into()))
        }
    } else {
        Err(env.error("syntax-error", None))
    }
}
