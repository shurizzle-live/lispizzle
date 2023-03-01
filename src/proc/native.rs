use std::{fmt, num::NonZeroUsize, rc::Rc};

use im_rc::Vector;

use super::{Callable, Parameters};
use crate::{BTrace, Error, Str, Value};

pub struct Repr<T>
where
    T: (Fn(BTrace, Vector<Value>) -> Result<Value, Error>) + ?Sized + 'static,
{
    parameters: Parameters<usize, NonZeroUsize>,
    doc: Option<Str>,
    fun: T,
}

#[allow(clippy::type_complexity)]
impl<F> Repr<F>
where
    F: (Fn(BTrace, Vector<Value>) -> Result<Value, Error>) + 'static,
{
    #[inline]
    pub fn new(
        parameters: Parameters<usize, NonZeroUsize>,
        doc: Option<Str>,
        fun: F,
    ) -> Rc<Repr<dyn Fn(BTrace, Vector<Value>) -> Result<Value, Error>>> {
        Rc::new(Repr {
            parameters,
            doc,
            fun,
        })
    }
}

impl Repr<dyn Fn(BTrace, Vector<Value>) -> Result<Value, Error>> {
    #[inline]
    pub fn doc(&self) -> Option<Str> {
        self.doc.clone()
    }

    #[inline]
    pub fn min_arity(&self) -> usize {
        match self.parameters {
            Parameters::Exact(n) => n,
            Parameters::Variadic(n) => n.get() - 1,
        }
    }
}

impl Callable for Rc<Repr<dyn Fn(BTrace, Vector<Value>) -> Result<Value, Error>>> {
    #[inline(always)]
    fn call(&self, trace: BTrace, parameters: Vector<Value>) -> Result<Value, Error> {
        (self.fun)(trace, parameters)
    }
}

#[allow(clippy::type_complexity)]
pub struct NativeProc(pub Rc<Repr<dyn Fn(BTrace, Vector<Value>) -> Result<Value, Error>>>);

fn fmt_parameters<T, I>(
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

impl NativeProc {
    #[inline]
    pub fn new<F>(parameters: Parameters<usize, NonZeroUsize>, doc: Option<Str>, fun: F) -> Self
    where
        F: (Fn(BTrace, Vector<Value>) -> Result<Value, Error>) + 'static,
    {
        Self(Repr::new(parameters, doc, fun))
    }

    pub fn fmt_parameters(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.parameters {
            Parameters::Exact(n) => fmt_parameters(f, false, n, ["_"].into_iter().cycle().take(n)),
            Parameters::Variadic(n) => {
                fmt_parameters(f, true, n.get(), ["_"].into_iter().cycle().take(n.get()))
            }
        }
    }

    #[inline]
    pub fn doc(&self) -> Option<Str> {
        self.0.doc()
    }

    #[inline]
    pub fn min_arity(&self) -> usize {
        self.0.min_arity()
    }
}

impl PartialEq for NativeProc {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&*self.0 as *const _, &*other.0 as *const _)
    }
}

impl Eq for NativeProc {}

impl Callable for NativeProc {
    #[inline]
    fn call(&self, trace: BTrace, parameters: Vector<Value>) -> Result<Value, Error> {
        self.0.call(trace, parameters)
    }
}

impl Clone for NativeProc {
    #[inline]
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}
