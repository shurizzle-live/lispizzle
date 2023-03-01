use std::{fmt, num::NonZeroUsize, rc::Rc};

use im_rc::Vector;

use crate::{BTrace, Error, Str, Symbol, TraceFrame, Value};

pub trait Callable {
    fn call(&self, trace: BTrace, parameters: Vector<Value>) -> Result<Value, Error>;
}

#[derive(Clone, Copy, Debug)]
pub enum Parameters<T1, T2> {
    Exact(T1),
    Variadic(T2),
}

struct NativeProcRepr<T>
where
    T: (Fn(BTrace, Vector<Value>) -> Result<Value, Error>) + ?Sized + 'static,
{
    parameters: Parameters<usize, NonZeroUsize>,
    doc: Option<Str>,
    fun: T,
}

#[allow(clippy::type_complexity)]
impl<F> NativeProcRepr<F>
where
    F: (Fn(BTrace, Vector<Value>) -> Result<Value, Error>) + 'static,
{
    #[inline]
    pub fn new(
        parameters: Parameters<usize, NonZeroUsize>,
        doc: Option<Str>,
        fun: F,
    ) -> Rc<NativeProcRepr<dyn Fn(BTrace, Vector<Value>) -> Result<Value, Error>>> {
        Rc::new(NativeProcRepr {
            parameters,
            doc,
            fun,
        })
    }
}

impl NativeProcRepr<dyn Fn(BTrace, Vector<Value>) -> Result<Value, Error>> {
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

impl Callable for Rc<NativeProcRepr<dyn Fn(BTrace, Vector<Value>) -> Result<Value, Error>>> {
    #[inline(always)]
    fn call(&self, trace: BTrace, parameters: Vector<Value>) -> Result<Value, Error> {
        (self.fun)(trace, parameters)
    }
}

#[allow(clippy::type_complexity)]
struct NativeProc(Rc<NativeProcRepr<dyn Fn(BTrace, Vector<Value>) -> Result<Value, Error>>>);

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
    fn new<F>(parameters: Parameters<usize, NonZeroUsize>, doc: Option<Str>, fun: F) -> Self
    where
        F: (Fn(BTrace, Vector<Value>) -> Result<Value, Error>) + 'static,
    {
        Self(NativeProcRepr::new(parameters, doc, fun))
    }

    fn fmt_parameters(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

#[derive(Clone)]
enum ProcRepr {
    Native(NativeProc),
}

impl ProcRepr {
    #[inline]
    fn from_native<F>(parameters: Parameters<usize, NonZeroUsize>, doc: Option<Str>, fun: F) -> Self
    where
        F: (Fn(BTrace, Vector<Value>) -> Result<Value, Error>) + 'static,
    {
        Self::Native(NativeProc::new(parameters, doc, fun))
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

impl Callable for ProcRepr {
    #[inline]
    fn call(&self, trace: BTrace, parameters: Vector<Value>) -> Result<Value, Error> {
        match self {
            Self::Native(f) => f.call(trace, parameters),
        }
    }
}

impl PartialEq for ProcRepr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Native(l0), Self::Native(r0)) => l0 == r0,
        }
    }
}

impl Eq for ProcRepr {}

#[derive(Clone)]
pub struct Proc {
    name: Option<Symbol>,
    r#macro: bool,
    repr: ProcRepr,
}

impl Proc {
    #[inline]
    pub fn from_native<F>(
        parameters: Parameters<usize, NonZeroUsize>,
        doc: Option<Str>,
        r#macro: bool,
        fun: F,
    ) -> Self
    where
        F: (Fn(BTrace, Vector<Value>) -> Result<Value, Error>) + 'static,
    {
        Self {
            name: None,
            r#macro,
            repr: (ProcRepr::from_native(parameters, doc, fun)),
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

    #[inline]
    pub fn is_macro(&self) -> bool {
        self.r#macro
    }

    fn _addr(&self) -> usize {
        match &self.repr {
            ProcRepr::Native(l) => &*l.0 as *const _ as *const u8 as usize,
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
    fn call(&self, trace: BTrace, parameters: Vector<Value>) -> Result<Value, Error> {
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
            ProcRepr::Native(l) => l.fmt_parameters(f)?,
        }
        write!(f, ">")
    }
}

#[cfg(test)]
mod tests {
    use std::{num::NonZeroUsize, ops::AddAssign};

    use im_rc::{vector, Vector};
    use rug::Integer;

    use crate::{BTrace, Callable, Error, Parameters, Proc, Symbol, Value};

    fn add(trace: BTrace, pars: Vector<Value>) -> Result<Value, Error> {
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
        let trace = BTrace::new();
        let lambda = Proc::from_native(
            Parameters::Variadic(NonZeroUsize::new(1).unwrap()),
            None,
            false,
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
                false,
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
                false,
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
            let mut lambda = Proc::from_native(Parameters::Exact(2), None, false, add);
            assert_eq!(
                format!("{:?}", lambda),
                format!("#<procedure {:x} (_ _)>", lambda.addr())
            );
            lambda.set_name(Symbol::Name("test".into()));
            assert_eq!(format!("{:?}", lambda), "#<procedure test (_ _)>");
        }
    }
}
