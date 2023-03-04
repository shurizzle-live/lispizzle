use im_rc::Vector;

use crate::{util::eval_block, Context, Environment, Error, Value};

pub struct Program(Vector<Value>);

impl Program {
    pub fn new(exprs: Vector<Value>) -> Self {
        Self(exprs)
    }

    pub fn eval(&self, ctx: Context, env: Environment) -> Result<Value, Error> {
        eval_block(&self.0, ctx, env)
    }
}
