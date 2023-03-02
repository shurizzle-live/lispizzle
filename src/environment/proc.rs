use im_rc::Vector;

use crate::{
    proc::{Parameters, UnboundProc},
    Context, Error, Symbol, Value,
};

pub fn subst_defs(ctx: &Context, exprs: &mut Vector<Value>) -> Result<Vector<Symbol>, Error> {
    let mut vars = Vector::<Symbol>::new();
    let mut i = 0;
    while i < exprs.len() {
        let delete = {
            let expr = &mut exprs[i];

            if let Value::List(list) = expr {
                let is_def = if let Some(Value::Symbol(Symbol::Name(sym))) = list.get(0) {
                    sym == "def"
                } else {
                    false
                };

                if is_def {
                    match list.len() {
                        2 => {
                            if let Value::Symbol(sym) = unsafe { list.get(1).unwrap_unchecked() } {
                                vars.push_back(sym.clone());
                                true
                            } else {
                                return Err(ctx.trace().error("syntax-error", None));
                            }
                        }
                        3 => {
                            if let Value::Symbol(sym) = unsafe { list.get(1).unwrap_unchecked() } {
                                vars.push_back(sym.clone());
                            } else {
                                return Err(ctx.trace().error("syntax-error", None));
                            }
                            list[0] = Value::Symbol(Symbol::Name("set!".into()));

                            false
                        }
                        _ => {
                            return Err(ctx.trace().error("syntax-error", None));
                        }
                    }
                } else {
                    false
                }
            } else {
                false
            }
        };

        if delete {
            exprs.remove(i);
        } else {
            i += 1;
        }
    }

    Ok(vars)
}

pub fn proc_macro(mut ctx: Context, mut exprs: Vector<Value>) -> Result<UnboundProc, Error> {
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

    let defs = subst_defs(&ctx, &mut exprs)?;

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

    Ok(UnboundProc::new(pars, defs, doc, exprs))
}
