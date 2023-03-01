mod native;

use std::{fmt, num::NonZeroUsize};

use im_rc::Vector;

use crate::{BackTrace, Error, Str, Symbol, TraceFrame, Value};

pub trait Callable {
    fn call(&self, trace: BackTrace, parameters: Vector<Value>) -> Result<Value, Error>;
}

#[derive(Clone, Copy, Debug)]
pub enum Parameters<T1, T2> {
    Exact(T1),
    Variadic(T2),
}

#[derive(Clone)]
enum Repr {
    Native(native::NativeProc),
}

impl Repr {
    #[inline]
    fn from_native<F>(parameters: Parameters<usize, NonZeroUsize>, doc: Option<Str>, fun: F) -> Self
    where
        F: (std::ops::Fn(BackTrace, Vector<Value>) -> Result<Value, Error>) + 'static,
    {
        Self::Native(native::NativeProc::new(parameters, doc, fun))
    }

    #[inline]
    pub fn doc(&self) -> Option<Str> {
        match self {
            Self::Native(ref f) => f.doc(),
        }
    }

    #[inline]
    pub fn min_arity(&self) -> usize {
        match self {
            Self::Native(ref f) => f.min_arity(),
        }
    }
}

impl Callable for Repr {
    #[inline]
    fn call(&self, trace: BackTrace, parameters: Vector<Value>) -> Result<Value, Error> {
        match self {
            Self::Native(f) => f.call(trace, parameters),
        }
    }
}

impl PartialEq for Repr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Native(l0), Self::Native(r0)) => l0 == r0,
        }
    }
}

impl Eq for Repr {}

#[derive(Clone)]
pub struct Proc {
    name: Option<Symbol>,
    repr: Repr,
}

impl Proc {
    #[inline]
    pub fn from_native<F>(
        parameters: Parameters<usize, NonZeroUsize>,
        doc: Option<Str>,
        fun: F,
    ) -> Self
    where
        F: (std::ops::Fn(BackTrace, Vector<Value>) -> Result<Value, Error>) + 'static,
    {
        Self {
            name: None,
            repr: (Repr::from_native(parameters, doc, fun)),
        }
    }

    #[inline]
    pub fn name(&self) -> Option<Symbol> {
        self.name.clone()
    }

    pub fn set_name<I: Into<Symbol>>(&mut self, name: I) {
        let name = name.into();
        self.name = Some(name);
    }

    #[inline]
    pub fn unset_name(&mut self) {
        self.name = None;
    }

    #[inline]
    pub fn doc(&self) -> Option<Str> {
        self.repr.doc()
    }

    #[inline]
    pub fn min_arity(&self) -> usize {
        self.repr.min_arity()
    }

    fn _addr(&self) -> usize {
        match &self.repr {
            Repr::Native(l) => &*l.0 as *const _ as *const u8 as usize,
        }
    }

    #[cfg(test)]
    #[inline]
    pub fn addr(&self) -> usize {
        self._addr()
    }

    #[inline]
    pub fn frame(&self) -> TraceFrame {
        if let Some(name) = self.name() {
            TraceFrame::named(self._addr(), name)
        } else {
            TraceFrame::unnamed(self._addr())
        }
    }
}

impl Callable for Proc {
    #[inline]
    fn call(&self, trace: BackTrace, parameters: Vector<Value>) -> Result<Value, Error> {
        self.repr.call(trace.with_frame(self.frame()), parameters)
    }
}

impl PartialEq for Proc {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.repr == other.repr
    }
}

impl Eq for Proc {}

impl fmt::Debug for Proc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#<procedure ")?;

        if let Some(name) = self.name.as_ref() {
            write!(f, "{} ", name)?;
        } else {
            write!(f, "{:x} ", self._addr())?;
        }

        match &self.repr {
            Repr::Native(l) => l.fmt_parameters(f)?,
        }
        write!(f, ">")
    }
}

#[cfg(test)]
mod tests {
    use std::{num::NonZeroUsize, ops::AddAssign};

    use im_rc::{vector, Vector};
    use rug::Integer;

    use super::Proc;

    use crate::{BackTrace, Callable, Error, Parameters, Symbol, Value};

    fn add(trace: BackTrace, pars: Vector<Value>) -> Result<Value, Error> {
        let mut res = Integer::from(0);

        for e in pars {
            if let Value::Integer(i) = e {
                res.add_assign(i);
            } else {
                return Err(trace.error("wrong-type-arg", None));
            }
        }

        Ok(res.into())
    }

    #[test]
    fn run() {
        let trace = BackTrace::new();
        let lambda = Proc::from_native(
            Parameters::Variadic(NonZeroUsize::new(1).unwrap()),
            None,
            add,
        );
        assert!(lambda == lambda);
        assert_eq!(lambda.call(trace.clone(), vector![]).unwrap(), 0.into());
        assert_eq!(
            lambda.call(trace.clone(), vector![1.into()]).unwrap(),
            1.into()
        );
        assert_eq!(
            lambda
                .call(trace.clone(), vector![1.into(), 2.into()])
                .unwrap(),
            3.into()
        );
        assert_eq!(
            lambda
                .call(trace, vector![1.into(), 2.into(), 3.into()])
                .unwrap(),
            6.into()
        );
    }

    #[test]
    fn fmt() {
        {
            let mut lambda = Proc::from_native(
                Parameters::Variadic(NonZeroUsize::new(1).unwrap()),
                None,
                add,
            );
            assert_eq!(
                format!("{:?}", lambda),
                format!("#<procedure {:x} (. _)>", lambda.addr())
            );
            lambda.set_name(Symbol::Name("test".into()));
            assert_eq!(format!("{:?}", lambda), "#<procedure test (. _)>");
        }
        {
            let mut lambda = Proc::from_native(
                Parameters::Variadic(NonZeroUsize::new(2).unwrap()),
                None,
                add,
            );
            assert_eq!(
                format!("{:?}", lambda),
                format!("#<procedure {:x} (_ . _)>", lambda.addr())
            );
            lambda.set_name(Symbol::Name("test".into()));
            assert_eq!(format!("{:?}", lambda), "#<procedure test (_ . _)>");
        }
        {
            let mut lambda = Proc::from_native(Parameters::Exact(2), None, add);
            assert_eq!(
                format!("{:?}", lambda),
                format!("#<procedure {:x} (_ _)>", lambda.addr())
            );
            lambda.set_name(Symbol::Name("test".into()));
            assert_eq!(format!("{:?}", lambda), "#<procedure test (_ _)>");
        }
    }
}
