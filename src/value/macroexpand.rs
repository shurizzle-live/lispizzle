use std::{mem, ops::ControlFlow};
use ControlFlow::*;

use crate::{proc::Callable, Context, Environment, Error, Symbol, Value};

type Expanded1 = ControlFlow<Value, (Value, bool)>;

type Expanded = ControlFlow<Value, Value>;

fn _expand(me: Value, ctx: Context, env: Environment) -> Result<Expanded1, Error> {
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
                return Err(ctx.trace().error("syntax-error", None));
            }
            l.remove(0);
            return expand_quasiquote(unsafe { l.pop_front().unwrap_unchecked() }, ctx, env)
                .map(Break);
        } else if let Some(var) = env.get(sym) {
            var.get()
        } else {
            return Ok(Continue((Value::List(l), false)));
        }
    } else {
        return Ok(Continue((Value::List(l), false)));
    };

    let r#macro = if let Value::Macro(proc) = value {
        proc
    } else {
        return Ok(Continue((Value::List(l), false)));
    };

    l.remove(0);
    r#macro.call(ctx, l).map(|x| Continue((x, true)))
}

fn expand_quasiquote(me: Value, ctx: Context, env: Environment) -> Result<Value, Error> {
    if let Value::List(mut list) = me {
        if let Some(Value::Symbol(Symbol::Name(name))) = list.get(0) {
            if name == "unquote" || name == "unquote-splicing" {
                if list.len() == 2 {
                    let exp = list.remove(1).macroexpand(ctx, env, false)?;
                    list.insert(1, exp);
                    Ok(Value::List(list))
                } else {
                    Err(ctx.trace().error("syntax-error", None))
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

fn is_def(me: &Value) -> bool {
    if let Value::List(l) = me {
        if let Some(Value::Symbol(Symbol::Name(name))) = l.get(0) {
            if name == "def" {
                return true;
            }
        }
    }
    false
}

fn expand(
    mut me: Value,
    ctx: Context,
    env: Environment,
    in_block: bool,
) -> Result<Expanded, Error> {
    while {
        let expanded;
        if in_block && is_def(&me) {
            return Ok(Continue(me));
        }

        (me, expanded) = match _expand(me, ctx.clone(), env.clone())? {
            Continue(x) => x,
            Break(x) => return Ok(Break(x)),
        };
        expanded
    } {}
    Ok(Continue(me))
}

pub fn macroexpand(
    me: Value,
    ctx: Context,
    env: Environment,
    in_block: bool,
) -> Result<Value, Error> {
    let me = match expand(me, ctx.clone(), env.clone(), in_block)? {
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

        exp = exp.macroexpand(ctx.clone(), env.clone(), false)?;

        mem::swap(&mut exp, unsafe { me.get_mut(i).unwrap_unchecked() });
        i += 1;
    }

    Ok(Value::List(me))
}
