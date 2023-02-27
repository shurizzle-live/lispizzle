use std::{cell::UnsafeCell, fmt, hash::Hash};

use ecow::EcoString;

#[derive(Clone)]
enum StrRepr {
    Static(&'static str),
    Alloc(EcoString),
}

pub struct Str {
    repr: UnsafeCell<StrRepr>,
}

impl StrRepr {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Static(s) => s,
            Self::Alloc(s) => s.as_str(),
        }
    }

    pub fn is_allocated(&self) -> bool {
        matches!(self, StrRepr::Alloc(_))
    }
}

impl fmt::Debug for StrRepr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Static(s) => fmt::Debug::fmt(s, f),
            Self::Alloc(s) => fmt::Debug::fmt(s, f),
        }
    }
}

impl fmt::Display for StrRepr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Static(s) => fmt::Display::fmt(s, f),
            Self::Alloc(s) => fmt::Display::fmt(s, f),
        }
    }
}

impl From<&'static str> for StrRepr {
    #[inline]
    fn from(value: &'static str) -> Self {
        Self::Static(value)
    }
}

impl From<EcoString> for StrRepr {
    #[inline]
    fn from(value: EcoString) -> Self {
        Self::Alloc(value)
    }
}

impl PartialEq<str> for StrRepr {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}

impl PartialEq<Str> for StrRepr {
    #[inline]
    fn eq(&self, other: &Str) -> bool {
        self.eq(other.as_str())
    }
}

impl PartialEq for StrRepr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.eq(other.as_str())
    }
}

impl PartialEq<String> for StrRepr {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.eq(other.as_str())
    }
}

impl PartialEq<EcoString> for StrRepr {
    #[inline]
    fn eq(&self, other: &EcoString) -> bool {
        self.eq(other.as_str())
    }
}

impl Eq for StrRepr {}

impl PartialOrd<str> for StrRepr {
    #[inline]
    fn partial_cmp(&self, other: &str) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other)
    }
}

impl PartialOrd<Str> for StrRepr {
    #[inline]
    fn partial_cmp(&self, other: &Str) -> Option<std::cmp::Ordering> {
        self.partial_cmp(other.get())
    }
}

impl PartialOrd for StrRepr {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.partial_cmp(other.as_str())
    }
}

impl PartialOrd<EcoString> for StrRepr {
    #[inline]
    fn partial_cmp(&self, other: &EcoString) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl PartialOrd<String> for StrRepr {
    #[inline]
    fn partial_cmp(&self, other: &String) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl Ord for StrRepr {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl Hash for StrRepr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Static(s) => s.hash(state),
            Self::Alloc(s) => s.hash(state),
        }
    }
}

impl Str {
    #[inline]
    fn get(&self) -> &StrRepr {
        unsafe { &*(self.repr.get() as *const _) }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.get().as_str()
    }

    #[allow(dead_code)]
    fn allocate(&self) {
        let inner = unsafe { &mut *self.repr.get() };

        if !inner.is_allocated() {
            unsafe { std::ptr::write(inner, StrRepr::Alloc(EcoString::from(inner.as_str()))) };
        }
    }
}

impl fmt::Debug for Str {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.get(), f)
    }
}

impl fmt::Display for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.get(), f)
    }
}

impl From<StrRepr> for Str {
    #[inline]
    fn from(value: StrRepr) -> Self {
        Self {
            repr: UnsafeCell::new(value),
        }
    }
}

impl From<&'static str> for Str {
    #[inline]
    fn from(value: &'static str) -> Self {
        Self {
            repr: UnsafeCell::new(value.into()),
        }
    }
}

impl From<EcoString> for Str {
    #[inline]
    fn from(value: EcoString) -> Self {
        Self {
            repr: UnsafeCell::new(value.into()),
        }
    }
}

impl PartialEq<str> for Str {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.get().eq(other)
    }
}

impl PartialEq<StrRepr> for Str {
    #[inline]
    fn eq(&self, other: &StrRepr) -> bool {
        self.get().eq(other)
    }
}

impl PartialEq for Str {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.get().eq(other)
    }
}

impl PartialEq<String> for Str {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.get().eq(other)
    }
}

impl PartialEq<EcoString> for Str {
    #[inline]
    fn eq(&self, other: &EcoString) -> bool {
        self.get().eq(other)
    }
}

impl Eq for Str {}

impl PartialOrd<str> for Str {
    #[inline]
    fn partial_cmp(&self, other: &str) -> Option<std::cmp::Ordering> {
        self.get().partial_cmp(other)
    }
}

impl PartialOrd for Str {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.get().partial_cmp(other)
    }
}

impl PartialOrd<EcoString> for Str {
    #[inline]
    fn partial_cmp(&self, other: &EcoString) -> Option<std::cmp::Ordering> {
        self.get().partial_cmp(other)
    }
}

impl PartialOrd<String> for Str {
    #[inline]
    fn partial_cmp(&self, other: &String) -> Option<std::cmp::Ordering> {
        self.get().partial_cmp(other)
    }
}

impl Ord for Str {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.get().cmp(other.get())
    }
}

impl Hash for Str {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get().hash(state)
    }
}

impl Clone for Str {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            repr: UnsafeCell::new(self.get().clone()),
        }
    }
}
