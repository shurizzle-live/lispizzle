use im_rc::Vector;

use crate::{
    proc::{Parameters, UnboundProc},
    Context, Error, Symbol, Value,
};

pub fn proc_macro(
    mut ctx: Context,
    source: Option<Vector<Value>>,
    mut exprs: Vector<Value>,
) -> Result<UnboundProc, Error> {
    let pars = if let Value::List(pars) = exprs.remove(0) {
        pars
    } else {
        return Err(ctx.trace().error("syntax-error", None));
    };

    let last = pars.len().saturating_sub(1);
    let mut variadic = false;
    let pars = pars
        .into_iter()
        .enumerate()
        .map(|(i, v)| {
            if let Value::Symbol(sym) = v {
                match sym {
                    Symbol::Name(name) => {
                        if !name.is_empty() {
                            if unsafe { name.as_str().chars().next().unwrap_unchecked() } == '&' {
                                if i == last {
                                    variadic = true;
                                    let name = unsafe {
                                        name.substring_in_context(&mut ctx, 1, None)
                                            .unwrap_unchecked()
                                    };
                                    if name.is_empty() {
                                        Err(ctx.trace().error("syntax-error", None))
                                    } else {
                                        Ok(Symbol::Name(name))
                                    }
                                } else {
                                    Err(ctx.trace().error("syntax-error", None))
                                }
                            } else {
                                Ok(Symbol::Name(name))
                            }
                        } else {
                            Err(ctx.trace().error("syntax-error", None))
                        }
                    }
                    _ => Ok(sym),
                }
            } else {
                Err(ctx.trace().error("syntax-error", None))
            }
        })
        .collect::<Result<Vector<Symbol>, Error>>()?;

    let pars = if variadic {
        Parameters::Variadic(pars)
    } else {
        Parameters::Exact(pars)
    };

    if exprs.is_empty() {
        return Err(ctx.trace().error("syntax-error", None));
    }

    let doc = if exprs.len() > 1 {
        if let Value::String(s) = unsafe { exprs.get(0).unwrap_unchecked() } {
            let s = s.clone();
            exprs.remove(0);
            Some(s)
        } else {
            None
        }
    } else {
        None
    };

    Ok(UnboundProc::new(source, pars, doc, exprs))
}
