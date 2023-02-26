use std::{
    borrow::Borrow,
    ops::{Deref, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
    path::Path,
};

use crate::parser::{Error, Message};

use super::str_reader::*;

#[derive(Debug, Clone)]
pub struct Input<'a> {
    path: Option<&'a Path>,
    inner: StringReader<'a>,
    need_ws: bool,
}

impl<'a> Input<'a> {
    pub fn new(path: Option<&'a Path>, text: &'a str) -> Self {
        Self {
            path,
            inner: StringReader::new(text),
            need_ws: false,
        }
    }

    #[inline]
    pub fn needs_ws(&self) -> bool {
        self.need_ws
    }

    #[inline]
    pub fn set_needs_ws(mut self) -> Self {
        self.need_ws = true;
        self
    }

    #[inline]
    pub fn unset_needs_ws(mut self) -> Self {
        self.need_ws = false;
        self
    }

    #[inline(always)]
    pub fn peek(&self) -> Option<char> {
        self.get(0)
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    pub fn ltrim(self) -> Self {
        Self {
            path: self.path,
            inner: self.inner.ltrim(),
            need_ws: self.need_ws,
        }
    }

    pub fn err<M: Into<Message>>(self, message: M) -> Error {
        Error {
            path: self
                .path
                .and_then(|p| p.to_str())
                .map(|s| s.to_string().into_boxed_str()),
            message: message.into(),
            location: self.location(),
            line: self.line_str().to_string().into_boxed_str(),
        }
    }

    pub fn ok<T>(self, value: T) -> std::result::Result<(Self, T), Error> {
        Ok((self, value))
    }
}

impl<'a> Deref for Input<'a> {
    type Target = StringReader<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> AsRef<StringReader<'a>> for Input<'a> {
    #[inline]
    fn as_ref(&self) -> &StringReader<'a> {
        self
    }
}

impl<'a> Borrow<StringReader<'a>> for Input<'a> {
    #[inline]
    fn borrow(&self) -> &StringReader<'a> {
        self
    }
}

impl<'a> AsRef<str> for Input<'a> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.inner.as_str()
    }
}

impl<'a> Borrow<str> for Input<'a> {
    #[inline]
    fn borrow(&self) -> &str {
        self.inner.as_str()
    }
}

impl Slice<usize> for Input<'_> {
    type Output = char;

    #[inline]
    fn get(&self, index: usize) -> Option<Self::Output> {
        self.inner.get(index)
    }
}

impl Slice<RangeFrom<usize>> for Input<'_> {
    type Output = Self;

    #[inline]
    fn get(&self, index: RangeFrom<usize>) -> Option<Self::Output> {
        Some(Self {
            path: self.path,
            inner: self.inner.get(index)?,
            need_ws: self.need_ws,
        })
    }
}

impl Slice<RangeInclusive<usize>> for Input<'_> {
    type Output = Self;

    #[inline]
    fn get(&self, index: RangeInclusive<usize>) -> Option<Self::Output> {
        Some(Self {
            path: self.path,
            inner: self.inner.get(index)?,
            need_ws: self.need_ws,
        })
    }
}

impl Slice<Range<usize>> for Input<'_> {
    type Output = Self;

    #[inline]
    fn get(&self, index: Range<usize>) -> Option<Self::Output> {
        Some(Self {
            path: self.path,
            inner: self.inner.get(index)?,
            need_ws: self.need_ws,
        })
    }
}

impl Slice<RangeTo<usize>> for Input<'_> {
    type Output = Self;

    #[inline]
    fn get(&self, index: RangeTo<usize>) -> Option<Self::Output> {
        Some(Self {
            path: self.path,
            inner: self.inner.get(index)?,
            need_ws: self.need_ws,
        })
    }
}

impl Slice<RangeToInclusive<usize>> for Input<'_> {
    type Output = Self;

    #[inline]
    fn get(&self, index: RangeToInclusive<usize>) -> Option<Self::Output> {
        Some(Self {
            path: self.path,
            inner: self.inner.get(index)?,
            need_ws: self.need_ws,
        })
    }
}

impl Slice<RangeFull> for Input<'_> {
    type Output = Self;

    #[inline]
    fn get(&self, index: RangeFull) -> Option<Self::Output> {
        Some(Self {
            path: self.path,
            inner: self.inner.get(index)?,
            need_ws: self.need_ws,
        })
    }
}
