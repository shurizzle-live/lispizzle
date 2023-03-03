use std::{fmt, rc::Rc};

use im_rc::Vector;

use crate::{proc::Parameters, Environment, Str, Symbol, Value};

mod unbound {
    use std::{fmt, rc::Rc};

    use im_rc::Vector;

    use crate::{proc::Parameters, Environment, Str, Symbol, Value};

    struct Repr {
        source: Option<Vector<Value>>,
        parameters: Parameters<Vector<Symbol>, Vector<Symbol>>,
        doc: Option<Str>,
        body: Vector<Value>,
    }

    pub struct LispProc(Rc<Repr>);

    impl LispProc {
        pub fn new(
            source: Option<Vector<Value>>,
            parameters: Parameters<Vector<Symbol>, Vector<Symbol>>,
            doc: Option<Str>,
            body: Vector<Value>,
        ) -> Self {
            Self(Rc::new(Repr {
                source,
                parameters,
                doc,
                body,
            }))
        }

        pub fn eval(&self, env: Environment) -> super::LispProc {
            super::LispProc::new(
                env,
                self.source(),
                self.0.parameters.clone(),
                self.0.doc.clone(),
                self.0.body.clone(),
            )
        }

        pub fn source(&self) -> Value {
            self.0
                .source
                .as_ref()
                .map(|s| s.clone().into())
                .unwrap_or(Value::Boolean(false))
        }

        pub fn fmt<T: fmt::Display>(&self, f: &mut fmt::Formatter<'_>, what: T) -> fmt::Result {
            if let Some(ref source) = self.0.source {
                fmt::Debug::fmt(&Value::from(source.clone()), f)
            } else {
                write!(f, "#<unbound-{}>", what)
            }
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
    source: Value,
    parameters: Parameters<Vector<Symbol>, Vector<Symbol>>,
    doc: Option<Str>,
    body: Vector<Value>,
}

pub struct LispProc(pub(crate) Rc<Repr>);

impl LispProc {
    pub fn new(
        env: Environment,
        source: Value,
        parameters: Parameters<Vector<Symbol>, Vector<Symbol>>,
        doc: Option<Str>,
        body: Vector<Value>,
    ) -> Self {
        Self(Rc::new(Repr {
            env,
            source,
            parameters,
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

    #[inline]
    pub fn source(&self) -> Value {
        self.0.source.clone()
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
        let fn_env = self.0.env.child(pars.into_iter());

        match self.0.parameters {
            Parameters::Exact(ref l) => {
                for sym in l {
                    _ = fn_env.set(sym, parameters.remove(0));
                }
            }
            Parameters::Variadic(ref l) => {
                let last = l.len() - 1;
                for sym in l.iter().take(last) {
                    _ = fn_env.set(sym, parameters.remove(0));
                }
                _ = fn_env.set(unsafe { l.get(last).unwrap_unchecked() }, parameters.into());
            }
        }

        let mut last = Value::Unspecified;
        for exp in self.0.body.iter().cloned() {
            last = exp.macroexpand(ctx.clone(), fn_env.clone(), true)?.eval(
                ctx.clone(),
                fn_env.clone(),
                true,
            )?;
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
