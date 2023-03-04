use std::num::NonZeroUsize;

use im_rc::Vector;

use crate::{proc::Parameters, Context, Environment, Error, Proc, Str, Symbol, Value};

pub fn define_fn<F, S1, S2>(
    env: &Environment,
    name: S1,
    ps: Parameters<usize, NonZeroUsize>,
    doc: Option<S2>,
    f: F,
) where
    F: (Fn(Context, Vector<Value>) -> Result<Value, Error>) + 'static,
    S1: Into<Str>,
    S2: Into<Str>,
{
    let mut lambda = Proc::from_native(ps, doc.map(|s| s.into()), f);
    let name: Str = name.into();
    lambda.set_name(name.clone());
    env.define(Symbol::Name(name), Value::Fn(lambda));
}

#[allow(dead_code)]
pub fn define_macro<F, S1, S2>(
    env: &Environment,
    name: S1,
    ps: Parameters<usize, NonZeroUsize>,
    doc: Option<S2>,
    f: F,
) where
    F: (Fn(Context, Vector<Value>) -> Result<Value, Error>) + 'static,
    S1: Into<Str>,
    S2: Into<Str>,
{
    let mut lambda = Proc::from_native(ps, doc.map(|s| s.into()), f);
    let name: Str = name.into();
    lambda.set_name(name.clone());
    env.define(Symbol::Name(name), Value::Macro(lambda));
}
