use std::{borrow::Borrow, fmt, ops::Deref};

use super::Location;

pub enum Message {
    Static(&'static str),
    Formatted(Box<str>),
}

impl Message {
    pub fn as_str(&self) -> &str {
        match self {
            &Self::Static(s) => s,
            Self::Formatted(s) => s,
        }
    }
}

impl Deref for Message {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for Message {
    #[inline]
    fn as_ref(&self) -> &str {
        self
    }
}

impl Borrow<str> for Message {
    #[inline]
    fn borrow(&self) -> &str {
        self
    }
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl From<&'static str> for Message {
    #[inline]
    fn from(value: &'static str) -> Self {
        Message::Static(value)
    }
}

impl From<&'static mut str> for Message {
    #[inline]
    fn from(value: &'static mut str) -> Self {
        Message::Static(value)
    }
}

impl From<Box<str>> for Message {
    #[inline]
    fn from(value: Box<str>) -> Self {
        Message::Formatted(value)
    }
}

impl From<String> for Message {
    #[inline]
    fn from(value: String) -> Self {
        value.into_boxed_str().into()
    }
}

#[derive(Debug)]
pub struct Error {
    pub(crate) path: Option<Box<str>>,
    pub(crate) message: Message,
    pub(crate) location: Location,
    pub(crate) line: Box<str>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}:{}:{}: {}\n{}",
            self.path.as_deref().unwrap_or("<unknown>"),
            self.location.line,
            self.location.column,
            self.message,
            self.line
        )?;

        for _ in 0..self.location.column.saturating_sub(1) {
            write!(f, " ")?;
        }
        writeln!(f, "^")
    }
}
