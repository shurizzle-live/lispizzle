use std::{fmt, rc::Rc};

use im_rc::Vector;

use crate::{proc::Parameters, Environment, Str, Symbol, Value};

mod unbound {
    use std::rc::Rc;

    use im_rc::Vector;

    use crate::{proc::Parameters, Environment, Str, Symbol, Value};

    struct Repr {
        parameters: Parameters<Vector<Symbol>, Vector<Symbol>>,
        defs: Vector<Symbol>,
        doc: Option<Str>,
        body: Vector<Value>,
    }

    pub struct LispProc(Rc<Repr>);

    impl LispProc {
        pub fn new(
            parameters: Parameters<Vector<Symbol>, Vector<Symbol>>,
            defs: Vector<Symbol>,
            doc: Option<Str>,
            body: Vector<Value>,
        ) -> Self {
            Self(Rc::new(Repr {
                parameters,
                defs,
                doc,
                body,
            }))
        }

        pub fn eval(&self, env: Environment) -> super::LispProc {
            super::LispProc::new(
                env,
                self.0.parameters.clone(),
                self.0.defs.clone(),
                self.0.doc.clone(),
                self.0.body.clone(),
            )
        }
    }

    impl Clone for LispProc {
        #[inline]
        fn clone(&self) -> Self {
            Self(Rc::clone(&self.0))
        }
    }

    impl PartialEq for LispProc {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            std::ptr::eq(&*self.0 as *const _, &*other.0 as *const _)
        }
    }

    impl Eq for LispProc {}
}

pub use unbound::LispProc as UnboundProc;

use super::{fmt_parameters, Callable};

pub(crate) struct Repr {
    env: Environment,
    parameters: Parameters<Vector<Symbol>, Vector<Symbol>>,
    defs: Vector<Symbol>,
    doc: Option<Str>,
    body: Vector<Value>,
}

pub struct LispProc(pub(crate) Rc<Repr>);

impl LispProc {
    pub fn new(
        env: Environment,
        parameters: Parameters<Vector<Symbol>, Vector<Symbol>>,
        defs: Vector<Symbol>,
        doc: Option<Str>,
        body: Vector<Value>,
    ) -> Self {
        Self(Rc::new(Repr {
            env,
            parameters,
            defs,
            doc,
            body,
        }))
    }

    #[inline]
    pub fn doc(&self) -> Option<Str> {
        self.0.doc.clone()
    }

    pub fn min_arity(&self) -> usize {
        match self.0.parameters {
            Parameters::Exact(ref l) => l.len(),
            Parameters::Variadic(ref l) => l.len() - 1,
        }
    }

    pub fn fmt_parameters(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.parameters {
            Parameters::Exact(ref l) => {
                let len = l.len();
                fmt_parameters(f, false, len, l.iter())
            }
            Parameters::Variadic(ref l) => {
                let len = l.len();
                fmt_parameters(f, true, len, l.iter())
            }
        }
    }
}

impl Callable for LispProc {
    fn call(
        &self,
        ctx: crate::Context,
        mut parameters: Vector<Value>,
    ) -> Result<Value, crate::Error> {
        let pars = match self.0.parameters {
            Parameters::Exact(ref l) => l.clone(),
            Parameters::Variadic(ref l) => l.clone(),
        };
        let fn_env = self
            .0
            .env
            .child(pars.into_iter().chain(self.0.defs.clone().into_iter()));

        match self.0.parameters {
            Parameters::Exact(ref l) => {
                for sym in l {
                    _ = fn_env.set(sym, parameters.remove(0));
                }
            }
            Parameters::Variadic(ref l) => {
                let last = l.len() - 1;
                for (i, sym) in l.iter().enumerate() {
                    if i == last {
                        _ = fn_env.set(sym, parameters.clone().into());
                    } else {
                        _ = fn_env.set(sym, parameters.remove(0));
                    }
                }
            }
        }

        let mut last = Value::Unspecified;
        for exp in self.0.body.iter().cloned() {
            last = exp.eval(ctx.clone(), fn_env.clone())?;
        }
        Ok(last)
    }
}

impl PartialEq for LispProc {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&*self.0 as *const _, &*other.0 as *const _)
    }
}

impl Eq for LispProc {}

impl Clone for LispProc {
    #[inline]
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}
