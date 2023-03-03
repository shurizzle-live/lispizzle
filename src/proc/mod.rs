mod lisp;
mod native;

pub use lisp::UnboundProc;

use std::{fmt, num::NonZeroUsize};

use im_rc::Vector;

use crate::{Context, Error, Str, Symbol, TraceFrame, Value};

pub trait Callable {
    fn call(&self, ctx: Context, parameters: Vector<Value>) -> Result<Value, Error>;
}

#[derive(Clone, Copy, Debug)]
pub enum Parameters<T1, T2> {
    Exact(T1),
    Variadic(T2),
}

#[derive(Clone)]
enum Repr {
    Native(native::NativeProc),
    Lisp(lisp::LispProc),
}

impl Repr {
    #[inline]
    fn from_native<F>(parameters: Parameters<usize, NonZeroUsize>, doc: Option<Str>, fun: F) -> Self
    where
        F: (std::ops::Fn(Context, Vector<Value>) -> Result<Value, Error>) + 'static,
    {
        Self::Native(native::NativeProc::new(parameters, doc, fun))
    }

    #[inline]
    pub fn doc(&self) -> Option<Str> {
        match self {
            Self::Native(ref f) => f.doc(),
            Self::Lisp(ref f) => f.doc(),
        }
    }

    #[inline]
    pub fn min_arity(&self) -> usize {
        match self {
            Self::Native(ref f) => f.min_arity(),
            Self::Lisp(ref f) => f.min_arity(),
        }
    }

    #[inline]
    pub fn source(&self) -> Value {
        match self {
            Self::Native(_) => false.into(),
            Self::Lisp(p) => p.source(),
        }
    }
}

impl Callable for Repr {
    #[inline]
    fn call(&self, ctx: Context, parameters: Vector<Value>) -> Result<Value, Error> {
        match self {
            Self::Native(f) => f.call(ctx, parameters),
            Self::Lisp(f) => f.call(ctx, parameters),
        }
    }
}

impl PartialEq for Repr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Native(l0), Self::Native(r0)) => l0 == r0,
            (Self::Lisp(l0), Self::Lisp(r0)) => l0 == r0,
            _ => false,
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
        F: (std::ops::Fn(Context, Vector<Value>) -> Result<Value, Error>) + 'static,
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
            Repr::Lisp(l) => &*l.0 as *const _ as *const u8 as usize,
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

    #[inline]
    pub fn source(&self) -> Value {
        self.repr.source()
    }

    pub fn fmt<T: fmt::Display>(&self, f: &mut fmt::Formatter<'_>, what: T) -> fmt::Result {
        write!(f, "#<{} ", what)?;

        if let Some(name) = self.name.as_ref() {
            write!(f, "{} ", name)?;
        } else {
            write!(f, "{:x} ", self._addr())?;
        }

        match &self.repr {
            Repr::Native(l) => l.fmt_parameters(f)?,
            Repr::Lisp(l) => l.fmt_parameters(f)?,
        }
        write!(f, ">")
    }
}

impl Callable for Proc {
    #[inline]
    fn call(&self, ctx: Context, parameters: Vector<Value>) -> Result<Value, Error> {
        self.repr.call(ctx.with_frame(self.frame()), parameters)
    }
}

impl PartialEq for Proc {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.repr == other.repr
    }
}

impl Eq for Proc {}

pub(self) fn fmt_parameters<T, I>(
    f: &mut fmt::Formatter<'_>,
    variadic: bool,
    len: usize,
    iiter: I,
) -> fmt::Result
where
    T: fmt::Display,
    I: IntoIterator<Item = T>,
{
    let mut it = iiter.into_iter();
    write!(f, "(")?;

    if variadic {
        if len > 0 {
            let last = len - 1;

            for (i, e) in it.enumerate() {
                if i == 0 && i == last {
                    write!(f, ". {}", e)?;
                } else if i == 0 {
                    write!(f, "{}", e)?;
                } else if i == last {
                    write!(f, " . {}", e)?;
                } else {
                    write!(f, " {}", e)?;
                }
            }
        }
    } else if let Some(e) = it.next() {
        write!(f, "{}", e)?;
        for e in it {
            write!(f, " {}", e)?;
        }
    }

    write!(f, ")")
}

impl From<native::NativeProc> for Proc {
    #[inline]
    fn from(value: native::NativeProc) -> Self {
        Self {
            name: None,
            repr: Repr::Native(value),
        }
    }
}

impl From<lisp::LispProc> for Proc {
    #[inline]
    fn from(value: lisp::LispProc) -> Self {
        Self {
            name: None,
            repr: Repr::Lisp(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{num::NonZeroUsize, ops::AddAssign};

    use im_rc::{vector, Vector};
    use rug::Integer;

    use super::Proc;

    use super::{Callable, Parameters};
    use crate::{Context, Error, Value};

    fn add(ctx: Context, pars: Vector<Value>) -> Result<Value, Error> {
        let mut res = Integer::from(0);

        for e in pars {
            if let Value::Integer(i) = e {
                res.add_assign(i);
            } else {
                return Err(ctx.trace().error("wrong-type-arg", None));
            }
        }

        Ok(res.into())
    }

    #[test]
    fn run() {
        let ctx = Context::new();
        let lambda = Proc::from_native(
            Parameters::Variadic(NonZeroUsize::new(1).unwrap()),
            None,
            add,
        );
        assert!(lambda == lambda);
        assert_eq!(lambda.call(ctx.clone(), vector![]).unwrap(), 0.into());
        assert_eq!(
            lambda.call(ctx.clone(), vector![1.into()]).unwrap(),
            1.into()
        );
        assert_eq!(
            lambda
                .call(ctx.clone(), vector![1.into(), 2.into()])
                .unwrap(),
            3.into()
        );
        assert_eq!(
            lambda
                .call(ctx, vector![1.into(), 2.into(), 3.into()])
                .unwrap(),
            6.into()
        );
    }

    // #[test]
    // fn fmt() {
    //     {
    //         let mut lambda = Proc::from_native(
    //             Parameters::Variadic(NonZeroUsize::new(1).unwrap()),
    //             None,
    //             add,
    //         );
    //         assert_eq!(
    //             format!("{:?}", lambda),
    //             format!("#<procedure {:x} (. _)>", lambda.addr())
    //         );
    //         lambda.set_name(Symbol::Name("test".into()));
    //         assert_eq!(format!("{:?}", lambda), "#<procedure test (. _)>");
    //     }
    //     {
    //         let mut lambda = Proc::from_native(
    //             Parameters::Variadic(NonZeroUsize::new(2).unwrap()),
    //             None,
    //             add,
    //         );
    //         assert_eq!(
    //             format!("{:?}", lambda),
    //             format!("#<procedure {:x} (_ . _)>", lambda.addr())
    //         );
    //         lambda.set_name(Symbol::Name("test".into()));
    //         assert_eq!(format!("{:?}", lambda), "#<procedure test (_ . _)>");
    //     }
    //     {
    //         let mut lambda = Proc::from_native(Parameters::Exact(2), None, add);
    //         assert_eq!(
    //             format!("{:?}", lambda),
    //             format!("#<procedure {:x} (_ _)>", lambda.addr())
    //         );
    //         lambda.set_name(Symbol::Name("test".into()));
    //         assert_eq!(format!("{:?}", lambda), "#<procedure test (_ _)>");
    //     }
    // }
}
