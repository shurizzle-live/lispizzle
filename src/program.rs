use im_rc::vector;

use crate::{Environment, Error, Str, Symbol, TraceFrame, Value};

pub struct Program {
    vars: Vec<Str>,
    exprs: Vec<Value>,
}

impl Program {
    pub fn new(mut exprs: Vec<Value>) -> Result<Self, Error> {
        let mut vars = Vec::<Str>::new();
        let mut i = 0;
        while i < exprs.len() {
            let delete = {
                let expr = &mut exprs[i];

                if let Value::List(list) = expr {
                    let is_define = if let Some(Value::Symbol(Symbol::Name(sym))) = list.get(0) {
                        sym == "define"
                    } else {
                        false
                    };

                    if is_define {
                        match list.len() {
                            2 => {
                                if let Value::Symbol(Symbol::Name(sym)) =
                                    unsafe { list.get(1).unwrap_unchecked() }
                                {
                                    vars.push(sym.clone());
                                    true
                                } else {
                                    return Err(Error::new(
                                        "syntax-error".into(),
                                        None,
                                        vector![TraceFrame::main()],
                                    ));
                                }
                            }
                            3 => {
                                if let Value::Symbol(Symbol::Name(sym)) =
                                    unsafe { list.get(1).unwrap_unchecked() }
                                {
                                    vars.push(sym.clone());
                                } else {
                                    return Err(Error::new(
                                        "syntax-error".into(),
                                        None,
                                        vector![TraceFrame::main()],
                                    ));
                                }
                                list[0] = Value::Symbol(Symbol::Name("set!".into()));

                                false
                            }
                            _ => {
                                return Err(Error::new(
                                    "syntax-error".into(),
                                    None,
                                    vector![TraceFrame::main()],
                                ));
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

        Ok(Self { vars, exprs })
    }

    pub fn eval(&self, env: Environment) -> Result<Value, Error> {
        for v in &self.vars {
            env.define(v.clone(), Value::Unspecified);
        }

        let mut res = Value::Unspecified;
        for exp in &self.exprs {
            res = exp.clone().eval(env.clone())?;
        }

        Ok(res)
    }
}
