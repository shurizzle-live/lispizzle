use crate::{Environment, Error, Symbol, Value};

pub struct Program(Vec<Value>);

impl Program {
    pub fn new(exprs: Vec<Value>) -> Self {
        Self(exprs)
    }

    fn filter_define(env: Environment, exp: Value) -> Result<Option<Value>, Error> {
        let mut list = if let Value::List(list) = exp {
            list
        } else {
            return Ok(Some(exp));
        };

        let mut args = if let Some(Value::Symbol(Symbol::Name(name))) = list.get(0) {
            if name == "define" {
                list.remove(0);
                list
            } else {
                return Ok(Some(Value::List(list)));
            }
        } else {
            return Ok(Some(Value::List(list)));
        };

        let (name, exp) = match args.len() {
            1 | 2 => (
                unsafe { args.pop_front().unwrap_unchecked() },
                args.pop_front().unwrap_or(Value::Unspecified),
            ),
            _ => return Err(env.error("syntax-error", None)),
        };

        let name = if let Value::Symbol(sym) = name {
            sym
        } else {
            return Err(env.error("syntax-error", None));
        };

        let mut value = exp.eval(env.clone())?;

        if let Value::Proc(ref mut proc) = value {
            proc.set_name(name.clone());
        }

        env.define(name, value);

        Ok(None)
    }

    pub fn eval(&self, env: Environment) -> Result<Value, Error> {
        let mut last = Value::Unspecified;
        for exp in self.0.iter() {
            let exp = exp.clone().macroexpand(env.clone())?;

            if let Some(exp) = Self::filter_define(env.clone(), exp)? {
                last = exp.eval(env.clone())?;
            }
        }

        Ok(last)
    }
}
