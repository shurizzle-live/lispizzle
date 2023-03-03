use crate::{Context, Environment, Error, Value};

pub struct Program(Vec<Value>);

impl Program {
    pub fn new(exprs: Vec<Value>) -> Self {
        Self(exprs)
    }

    pub fn eval(&self, ctx: Context, env: Environment) -> Result<Value, Error> {
        let mut last = Value::Unspecified;
        for exp in self.0.iter() {
            let exp = exp.clone().macroexpand(ctx.clone(), env.clone(), true)?;
            last = exp.eval(ctx.clone(), env.clone(), true)?;
        }

        Ok(last)
    }
}
