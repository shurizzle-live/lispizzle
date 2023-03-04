use im_rc::{vector, Vector};

use crate::{proc::Callable, special, Context, Environment, Error, Symbol, Value};

pub fn apply(me: Value, ctx: Context, mut args: Vector<Value>) -> Result<Value, Error> {
    match me {
        Value::Fn(l) => {
            if l.min_arity() > args.len() {
                Err(ctx.trace().error("wrong-number-of-args", None))
            } else {
                l.call(ctx, args)
            }
        }
        Value::Integer(l) => {
            if args.len() != 1 {
                return Err(ctx.trace().error("wrong-number-of-args", None));
            }
            args.remove(0).element_at(ctx, &l)
        }
        _ => Err(ctx.trace().error("wrong-type-arg", None)),
    }
}

pub fn apply_recur(me: Value, ctx: Context, mut args: Vector<Value>) -> Result<LastValue, Error> {
    match me {
        Value::Fn(l) => {
            if l.min_arity() > args.len() {
                Err(ctx.trace().error("wrong-number-of-args", None))
            } else if ctx.trace().get(0).map(|f| f == l.frame()).unwrap_or(false) {
                Ok(LastValue::Recur(args))
            } else {
                Ok(LastValue::Value(l.call(ctx, args)?))
            }
        }
        Value::Integer(l) => {
            if args.len() != 1 {
                return Err(ctx.trace().error("wrong-number-of-args", None));
            }
            args.remove(0).element_at(ctx, &l).map(Into::into)
        }
        _ => Err(ctx.trace().error("wrong-type-arg", None)),
    }
}

pub fn value_fn<T, F>(
    me: Value,
    ctx: Context,
    env: Environment,
    in_block: bool,
    apply: F,
) -> Result<T, Error>
where
    T: From<Value>,
    F: (Fn(Value, Context, Vector<Value>) -> Result<T, Error>) + Clone,
{
    match me {
        Value::UnboundFn(f) => Ok(Value::Fn(f.eval(env).into()).into()),
        Value::UnboundMacro(f) => Ok(Value::Macro(f.eval(env).into()).into()),
        Value::Unspecified
        | Value::Nil
        | Value::Boolean(_)
        | Value::Character(_)
        | Value::Integer(_)
        | Value::String(_)
        | Value::Fn(_)
        | Value::Macro(_)
        | Value::Var(_)
        | Value::Environment(_)
        | Value::Error(_)
        | Value::BackTrace(_)
        | Value::Frame(_) => Ok(me.into()),
        Value::Symbol(sym) => env.get(sym.clone()).map(|v| v.get().into()).ok_or_else(|| {
            ctx.trace()
                .error("unbound-variable", Some(vector![Value::Symbol(sym)]))
        }),
        Value::List(mut l) => {
            if let Some(first) = l.pop_front() {
                if let Value::Symbol(Symbol::Name(ref s)) = first {
                    if let Some(res) = special::transform_fn(
                        ctx.clone(),
                        env.clone(),
                        s.clone(),
                        l.clone(),
                        in_block,
                        apply.clone(),
                    ) {
                        return res.map(Into::into);
                    }
                }

                let resolved = first.eval(ctx.clone(), env.clone(), false)?;

                if resolved.is_macro() {
                    Err(ctx.trace().error("wrong-type-arg", None))
                } else {
                    let args = l
                        .into_iter()
                        .map(|v| v.eval(ctx.clone(), env.clone(), false))
                        .collect::<Result<Vector<_>, Error>>()?;

                    apply(resolved, ctx, args)
                }
            } else {
                Err(ctx.trace().error("syntax-error", None))
            }
        }
    }
}

#[inline]
pub fn value(me: Value, ctx: Context, env: Environment, in_block: bool) -> Result<Value, Error> {
    value_fn(me, ctx, env, in_block, apply)
}

pub enum LastValue {
    Value(Value),
    Recur(Vector<Value>),
}

impl From<Value> for LastValue {
    #[inline]
    fn from(value: Value) -> Self {
        Self::Value(value)
    }
}

fn expression(exp: &Value, ctx: Context, env: Environment) -> Result<Value, Error> {
    exp.clone()
        .macroexpand(ctx.clone(), env.clone(), true)?
        .eval(ctx, env, true)
}

pub fn block_fn<T, F>(
    exprs: &Vector<Value>,
    ctx: Context,
    env: Environment,
    apply: F,
) -> Result<T, Error>
where
    T: From<Value>,
    F: (Fn(Value, Context, Vector<Value>) -> Result<T, Error>) + Clone,
{
    if exprs.is_empty() {
        return Err(ctx.trace().error("syntax-error", None));
    }

    let last = exprs.len() - 1;
    for e in exprs.iter().take(last) {
        expression(e, ctx.clone(), env.clone())?;
    }

    value_fn(
        unsafe { exprs.get(last).unwrap_unchecked() }.clone(),
        ctx,
        env,
        true,
        apply,
    )
}

#[inline]
pub fn block(exprs: &Vector<Value>, ctx: Context, env: Environment) -> Result<Value, Error> {
    block_fn(exprs, ctx, env, apply)
}
