use std::{mem, ops::ControlFlow};
use ControlFlow::*;

use crate::{proc::Callable, BTrace, Environment, Error, Symbol, Value};

type Expanded1 = ControlFlow<Value, (Value, bool)>;

type Expanded = ControlFlow<Value, Value>;

fn _expand(me: Value, trace: BTrace, env: Environment) -> Result<Expanded1, Error> {
    let mut l = match me {
        Value::List(l) => l,
        _ => return Ok(Continue((me, false))),
    };

    let value = if let Some(Value::Symbol(sym)) = l.get(0) {
        let (quote, quasiquote) = if let Symbol::Name(name) = sym {
            (name == "quote", name == "quasiquote")
        } else {
            (false, false)
        };

        if quote {
            return Ok(Break(l.into()));
        } else if quasiquote {
            if l.len() != 2 {
                return Err(trace.error("syntax-error", None));
            }
            l.remove(0);
            return expand_quasiquote(unsafe { l.pop_front().unwrap_unchecked() }, trace, env)
                .map(Break);
        } else if let Some(var) = env.get(sym) {
            var.get()
        } else {
            return Ok(Continue((Value::List(l), false)));
        }
    } else {
        return Ok(Continue((Value::List(l), false)));
    };

    let r#macro = if let Value::Proc(proc) = value {
        if proc.is_macro() {
            proc
        } else {
            return Ok(Continue((Value::List(l), false)));
        }
    } else {
        return Ok(Continue((Value::List(l), false)));
    };

    l.remove(0);
    r#macro.call(trace, l).map(|x| Continue((x, true)))
}

fn expand_quasiquote(me: Value, trace: BTrace, env: Environment) -> Result<Value, Error> {
    if let Value::List(mut list) = me {
        if let Some(Value::Symbol(Symbol::Name(name))) = list.get(0) {
            if name == "unquote" || name == "unquote-splicing" {
                if list.len() == 2 {
                    let exp = list.remove(1).macroexpand(trace, env)?;
                    list.insert(1, exp);
                    Ok(Value::List(list))
                } else {
                    Err(trace.error("syntax-error", None))
                }
            } else {
                Ok(Value::List(list))
            }
        } else {
            Ok(Value::List(list))
        }
    } else {
        Ok(me)
    }
}

fn expand(mut me: Value, trace: BTrace, env: Environment) -> Result<Expanded, Error> {
    while {
        let expanded;
        (me, expanded) = match _expand(me, trace.clone(), env.clone())? {
            Continue(x) => x,
            Break(x) => return Ok(Break(x)),
        };
        expanded
    } {}
    Ok(Continue(me))
}

pub fn macroexpand(me: Value, trace: BTrace, env: Environment) -> Result<Value, Error> {
    let me = match expand(me, trace.clone(), env.clone())? {
        Continue(x) => x,
        Break(x) => return Ok(x),
    };

    let mut me = if let Value::List(me) = me {
        me
    } else {
        return Ok(me);
    };

    let mut i = 0;
    while i < me.len() {
        let mut exp = Value::Nil;
        mem::swap(&mut exp, unsafe { me.get_mut(i).unwrap_unchecked() });

        exp = exp.macroexpand(trace.clone(), env.clone())?;

        mem::swap(&mut exp, unsafe { me.get_mut(i).unwrap_unchecked() });
        i += 1;
    }

    Ok(Value::List(me))
}
