use std::{fmt, hash::Hash, mem};

use ecow::{EcoString, EcoVec};

use crate::parser::reader::util::{CountChars, SkipChars};

#[derive(Clone)]
enum Repr {
    Empty,
    Static(&'static str, usize),
    Alloc(EcoVec<u8>, usize),
}

#[derive(Clone)]
pub struct Str(Repr);

impl Repr {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Empty => "",
            Self::Static(s, _) => s,
            Self::Alloc(s, _) => unsafe { std::str::from_utf8_unchecked(s.as_slice()) },
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Empty => true,
            Self::Static(s, _) => s.is_empty(),
            Self::Alloc(s, _) => s.is_empty(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Static(_, l) => *l,
            Self::Alloc(_, l) => *l,
        }
    }

    pub fn bytes_len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Static(s, _) => s.len(),
            Self::Alloc(s, _) => s.len(),
        }
    }

    pub fn substring(self, start: usize, rlen: Option<usize>) -> Option<Self> {
        let len = self.len();

        if start > len {
            return None;
        }
        let new_len = len - start;

        let rlen = if let Some(l) = rlen {
            if l > new_len {
                return None;
            } else {
                l
            }
        } else {
            new_len
        };

        if matches!(self, Self::Empty) {
            return Some(Self::Empty);
        }

        let s = unsafe { self.as_str().skip_chars(start).unwrap_unchecked() };
        let bstart = self.bytes_len() - s.len();
        let bsize = if new_len == rlen {
            self.bytes_len() - bstart
        } else {
            unsafe { s.skip_chars(rlen).unwrap_unchecked().len() }
        };

        match self {
            Self::Empty => unreachable!(),
            Self::Static(s, _) => Some(Self::Static(&s[bstart..(bstart + bsize)], new_len)),
            Self::Alloc(mut s, _) => {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        s.as_ptr().add(bstart),
                        s.as_ptr() as *mut _,
                        bsize,
                    )
                };
                s.truncate(bsize);

                Some(Self::Alloc(s, rlen))
            }
        }
    }

    pub fn concat(self, other: Self) -> Self {
        match (self, other) {
            (Self::Empty, other) => other,
            (me, Self::Empty) => me,
            (Self::Static(ls, ll), Self::Static(rs, rl)) => {
                let mut s = EcoVec::new();
                s.extend_from_slice(ls.as_bytes());
                s.extend_from_slice(rs.as_bytes());
                Self::Alloc(s, ll + rl)
            }
            (Self::Alloc(mut ls, ll), Self::Static(rs, rl)) => {
                ls.extend_from_slice(rs.as_bytes());
                Self::Alloc(ls, ll + rl)
            }
            (Self::Static(ls, ll), Self::Alloc(mut rs, rl)) => {
                rs.extend_from_slice(ls.as_bytes());
                unsafe { std::slice::from_raw_parts_mut(rs.as_ptr() as *mut u8, rs.len()) }
                    .rotate_right(ls.len());
                Self::Alloc(rs, ll + rl)
            }
            (Repr::Alloc(mut ls, ll), Repr::Alloc(rs, rl)) => {
                ls.extend_from_slice(rs.as_slice());
                Self::Alloc(ls, ll + rl)
            }
        }
    }
}

impl fmt::Debug for Repr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "\"\""),
            Self::Static(s, _) => fmt::Debug::fmt(s, f),
            Self::Alloc(s, _) => {
                fmt::Debug::fmt(unsafe { std::str::from_utf8_unchecked(s.as_slice()) }, f)
            }
        }
    }
}

impl fmt::Display for Repr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => Ok(()),
            Self::Static(s, _) => fmt::Display::fmt(s, f),
            Self::Alloc(s, _) => {
                fmt::Display::fmt(unsafe { std::str::from_utf8_unchecked(s.as_slice()) }, f)
            }
        }
    }
}

impl From<&'static str> for Repr {
    #[inline]
    fn from(value: &'static str) -> Self {
        if value.is_empty() {
            Self::Empty
        } else {
            Self::Static(value, value.count_chars())
        }
    }
}

impl From<EcoVec<u8>> for Repr {
    #[inline]
    fn from(value: EcoVec<u8>) -> Self {
        if value.is_empty() {
            Self::Empty
        } else {
            let len = value.count_chars();
            Self::Alloc(value, len)
        }
    }
}

impl PartialEq<str> for Repr {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}

impl PartialEq<Str> for Repr {
    #[inline]
    fn eq(&self, other: &Str) -> bool {
        self.eq(other.as_str())
    }
}

impl PartialEq for Repr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.eq(other.as_str())
    }
}

impl PartialEq<String> for Repr {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.eq(other.as_str())
    }
}

impl PartialEq<EcoString> for Repr {
    #[inline]
    fn eq(&self, other: &EcoString) -> bool {
        self.eq(other.as_str())
    }
}

impl Eq for Repr {}

impl PartialOrd<str> for Repr {
    #[inline]
    fn partial_cmp(&self, other: &str) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other)
    }
}

impl PartialOrd<Str> for Repr {
    #[inline]
    fn partial_cmp(&self, other: &Str) -> Option<std::cmp::Ordering> {
        self.partial_cmp(other.repr())
    }
}

impl PartialOrd for Repr {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.partial_cmp(other.as_str())
    }
}

impl PartialOrd<EcoString> for Repr {
    #[inline]
    fn partial_cmp(&self, other: &EcoString) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl PartialOrd<String> for Repr {
    #[inline]
    fn partial_cmp(&self, other: &String) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl Ord for Repr {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl Hash for Repr {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl Str {
    #[inline(always)]
    fn repr(&self) -> &Repr {
        &self.0
    }

    #[inline(always)]
    fn repr_mut(&mut self) -> &mut Repr {
        &mut self.0
    }

    #[inline(always)]
    fn mutate<R, F: FnOnce(Repr) -> (Repr, R)>(&mut self, f: F) -> R {
        let mut inner = Repr::Empty;
        mem::swap(self.repr_mut(), &mut inner);
        let res: R;
        (inner, res) = f(inner);
        mem::swap(self.repr_mut(), &mut inner);
        res
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.repr().as_str()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.repr().len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.repr().is_empty()
    }

    #[inline]
    pub fn bytes_len(&self) -> usize {
        self.repr().bytes_len()
    }

    #[inline]
    pub fn substring(mut self, start: usize, len: Option<usize>) -> Option<Self> {
        let mut inner = Repr::Empty;
        mem::swap(self.repr_mut(), &mut inner);
        inner.substring(start, len).map(Self)
    }

    #[inline]
    pub fn concat(&mut self, other: Str) {
        self.mutate(move |inner| (inner.concat(other.0), ()))
    }
}

impl fmt::Debug for Str {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.repr(), f)
    }
}

impl fmt::Display for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.repr(), f)
    }
}

impl From<Repr> for Str {
    #[inline]
    fn from(value: Repr) -> Self {
        Self(value)
    }
}

impl From<&'static str> for Str {
    #[inline]
    fn from(value: &'static str) -> Self {
        Self(value.into())
    }
}

impl From<EcoVec<u8>> for Str {
    #[inline]
    fn from(value: EcoVec<u8>) -> Self {
        Self(value.into())
    }
}

impl PartialEq<str> for Str {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.repr().eq(other)
    }
}

impl PartialEq<Repr> for Str {
    #[inline]
    fn eq(&self, other: &Repr) -> bool {
        self.repr().eq(other)
    }
}

impl PartialEq for Str {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.repr().eq(other)
    }
}

impl PartialEq<String> for Str {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.repr().eq(other)
    }
}

impl PartialEq<EcoString> for Str {
    #[inline]
    fn eq(&self, other: &EcoString) -> bool {
        self.repr().eq(other)
    }
}

impl Eq for Str {}

impl PartialOrd<str> for Str {
    #[inline]
    fn partial_cmp(&self, other: &str) -> Option<std::cmp::Ordering> {
        self.repr().partial_cmp(other)
    }
}

impl PartialOrd for Str {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.repr().partial_cmp(other)
    }
}

impl PartialOrd<EcoString> for Str {
    #[inline]
    fn partial_cmp(&self, other: &EcoString) -> Option<std::cmp::Ordering> {
        self.repr().partial_cmp(other)
    }
}

impl PartialOrd<String> for Str {
    #[inline]
    fn partial_cmp(&self, other: &String) -> Option<std::cmp::Ordering> {
        self.repr().partial_cmp(other)
    }
}

impl Ord for Str {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.repr().cmp(other.repr())
    }
}

impl Hash for Str {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.repr().hash(state)
    }
}
