use im_rc::Vector;

use crate::{Environment, Error, Str, Symbol, Value};

use std::mem;

pub fn transform(env: Environment, name: Str, args: Vector<Value>) -> Option<Result<Value, Error>> {
    match name.as_str() {
        "quote" => Some(quote(env, args)),
        "quasiquote" => Some(quasiquote(env, args)),
        "if" => Some(iff(env, args)),
        _ => None,
    }
}

fn quote(env: Environment, args: Vector<Value>) -> Result<Value, Error> {
    if args.len() == 1 {
        Ok(args[0].clone())
    } else {
        Err(env.error("syntax-error", None))
    }
}

fn quasiquote(env: Environment, args: Vector<Value>) -> Result<Value, Error> {
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
                        list[1].clone().eval(env).map(Res::Value)
                    } else {
                        Err(env.error("syntax-error", None))
                    };
                } else if name == "unquote-splicing" {
                    return if list.len() == 2 {
                        list[1].clone().eval(env).map(|x| {
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

    match scan(env.clone(), args[0].clone())? {
        Res::Value(v) => Ok(v),
        _ => Err(env.error("syntax-error", None)),
    }
}

fn iff(env: Environment, args: Vector<Value>) -> Result<Value, Error> {
    if args.len() == 2 {
        if args[0].clone().eval(env.clone())?.to_bool() {
            args[1].clone().eval(env)
        } else {
            Ok(Value::Unspecified)
        }
    } else if args.len() == 3 {
        if args[0].clone().eval(env.clone())?.to_bool() {
            args[1].clone().eval(env)
        } else {
            args[2].clone().eval(env)
        }
    } else {
        Err(env.error("syntax-error", None))
    }
}
