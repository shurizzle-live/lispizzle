use crate::{BackTrace, Environment, Error, Symbol, Value};

pub struct Program(Vec<Value>);

impl Program {
    pub fn new(exprs: Vec<Value>) -> Self {
        Self(exprs)
    }

    fn filter_define(
        trace: BackTrace,
        env: Environment,
        exp: Value,
    ) -> Result<Option<Value>, Error> {
        let mut list = if let Value::List(list) = exp {
            list
        } else {
            return Ok(Some(exp));
        };

        let mut args = if let Some(Value::Symbol(Symbol::Name(name))) = list.get(0) {
            if name == "def" {
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
            _ => return Err(trace.error("syntax-error", None)),
        };

        let name = if let Value::Symbol(sym) = name {
            sym
        } else {
            return Err(trace.error("syntax-error", None));
        };

        let mut value = exp.eval(trace, env.clone())?;

        match value {
            Value::Macro(ref mut proc) | Value::Fn(ref mut proc) => proc.set_name(name.clone()),
            _ => (),
        }

        env.define(name, value);

        Ok(None)
    }

    pub fn eval(&self, trace: BackTrace, env: Environment) -> Result<Value, Error> {
        let mut last = Value::Unspecified;
        for exp in self.0.iter() {
            let exp = exp.clone().macroexpand(trace.clone(), env.clone())?;

            if let Some(exp) = Self::filter_define(trace.clone(), env.clone(), exp)? {
                last = exp.eval(trace.clone(), env.clone())?;
            }
        }

        Ok(last)
    }
}
