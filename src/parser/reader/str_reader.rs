use std::{
    borrow::{Borrow, Cow},
    fmt,
    ops::{Deref, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
};

use unicode_width::UnicodeWidthStr;

use super::{
    super::Location,
    util::{CountChars, SkipChars},
};

pub trait Slice<T> {
    type Output;

    fn get(&self, index: T) -> Option<Self::Output>;

    unsafe fn get_unchecked(&self, index: T) -> Self::Output {
        self.get(index).unwrap_unchecked()
    }
}

pub trait Reader:
    Slice<usize, Output = char>
    + Slice<RangeFull, Output = Self>
    + Slice<RangeInclusive<usize>, Output = Self>
    + Slice<Range<usize>, Output = Self>
    + Slice<RangeTo<usize>, Output = Self>
    + Slice<RangeToInclusive<usize>, Output = Self>
    + Slice<RangeFrom<usize>, Output = Self>
    + Clone
{
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn as_str(&self) -> &str;

    /// zero-based line number
    fn line(&self) -> usize;

    /// zero-based column number
    fn column(&self) -> usize;

    /// Line string as-is
    fn line_str(&self) -> Cow<str>;

    fn location(&self) -> Location {
        Location {
            line: self.line(),
            column: self.column(),
        }
    }

    fn ltrim(self) -> Self;

    fn skip_until_nl(self) -> Self;

    fn split_at<F: Fn(char) -> bool>(self, f: F) -> Option<(Self, Self)>;
}

#[derive(Clone, Debug)]
pub struct StringReader<'a> {
    offset: usize,
    last: usize,
    line: usize,
    char_count: usize,
    text: &'a str,
}

impl<'a> StringReader<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            offset: 0,
            last: text.len(),
            line: 1,
            char_count: text.chars().count(),
            text,
        }
    }

    #[inline]
    #[allow(dead_code)]
    pub const fn offset(&self) -> usize {
        self.offset
    }

    #[inline]
    #[allow(dead_code)]
    pub const fn byte_len(&self) -> usize {
        self.last - self.offset
    }

    fn line_and_offset(&self) -> (Cow<'a, str>, usize) {
        let start = memchr::memrchr(b'\n', self.text[..self.offset].as_bytes())
            .map(|x| x + 1)
            .unwrap_or(0);
        let stop = memchr::memchr(b'\n', self.text[self.offset..].as_bytes())
            .map(|x| self.offset + x)
            .unwrap_or(self.text.len());

        let slice = Cow::Borrowed(&self.text[start..stop]);
        let col = self.offset - start;

        (slice, col)
    }
}

impl Slice<usize> for StringReader<'_> {
    type Output = char;

    fn get(&self, index: usize) -> Option<Self::Output> {
        self.as_str()
            .skip_chars(index)
            .and_then(|rest| rest.chars().next())
    }
}

impl Slice<RangeFrom<usize>> for StringReader<'_> {
    type Output = Self;

    fn get(&self, index: RangeFrom<usize>) -> Option<Self::Output> {
        let init = self.as_str();
        init.skip_chars_count_nl(index.start).map(|(s, nl)| Self {
            offset: self.offset + (init.len() - s.len()),
            last: self.last,
            line: self.line + nl,
            char_count: self.char_count - index.start,
            text: self.text,
        })
    }
}

impl Slice<RangeInclusive<usize>> for StringReader<'_> {
    type Output = Self;

    fn get(&self, index: RangeInclusive<usize>) -> Option<Self::Output> {
        if *index.start() > *index.end()
            || *index.start() > self.char_count
            || *index.end() >= self.char_count
        {
            return None;
        }

        let char_count = *index.end() - *index.start() + 1;
        let init = self.as_str();
        init.skip_chars_count_nl(*index.start())
            .and_then(|(s1, nl)| {
                let offset = self.offset + (init.len() - s1.len());

                s1.skip_chars(char_count).map(|s2| Self {
                    offset,
                    last: offset + (s1.len() - s2.len()),
                    line: self.line + nl,
                    char_count,
                    text: self.text,
                })
            })
    }
}

impl Slice<Range<usize>> for StringReader<'_> {
    type Output = Self;

    fn get(&self, index: Range<usize>) -> Option<Self::Output> {
        if index.start == index.end {
            return Slice::get(self, index.start..).map(|r| Self {
                last: r.offset,
                char_count: 0,
                ..r
            });
        }

        index
            .end
            .checked_sub(1)
            .and_then(|end| Slice::get(self, index.start..=end))
    }
}

impl Slice<RangeTo<usize>> for StringReader<'_> {
    type Output = Self;

    #[inline]
    fn get(&self, index: RangeTo<usize>) -> Option<Self::Output> {
        Slice::get(self, 0..index.end)
    }
}

impl Slice<RangeToInclusive<usize>> for StringReader<'_> {
    type Output = Self;

    #[inline]
    fn get(&self, index: RangeToInclusive<usize>) -> Option<Self::Output> {
        Slice::get(self, 0..=index.end)
    }
}

impl Slice<RangeFull> for StringReader<'_> {
    type Output = Self;

    #[inline]
    fn get(&self, _: RangeFull) -> Option<Self::Output> {
        Some(self.clone())
    }
}

impl<'a> Reader for StringReader<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.char_count
    }

    #[inline]
    fn as_str(&self) -> &str {
        &self.text[self.offset..self.last]
    }

    #[inline]
    fn line(&self) -> usize {
        self.line
    }

    fn column(&self) -> usize {
        let (line, offset) = self.line_and_offset();
        UnicodeWidthStr::width(&line[..offset]) + 1
    }

    #[inline]
    fn line_str(&self) -> Cow<'a, str> {
        self.line_and_offset().0
    }

    fn ltrim(self) -> Self {
        let mut nl = 0;
        let mut len = 0;
        let mut bytes_len = 0;
        for c in self.as_str().chars() {
            if c == '\n' {
                nl += 1;
                len += 1;
                bytes_len += c.len_utf8();
            } else if c.is_whitespace() {
                len += 1;
                bytes_len += c.len_utf8();
            } else {
                break;
            }
        }

        Self {
            offset: self.offset + bytes_len,
            last: self.last,
            line: self.line + nl,
            char_count: self.char_count - len,
            text: self.text,
        }
    }

    fn split_at<F: Fn(char) -> bool>(self, f: F) -> Option<(Self, Self)> {
        let mut nl = 0;
        let mut bytes_len = 0;
        for (char_count, c) in self.as_str().chars().enumerate() {
            if c == '\n' {
                nl += 1;
            }

            if f(c) {
                return Some((
                    Self {
                        offset: self.offset,
                        last: self.offset + bytes_len,
                        line: self.line,
                        char_count,
                        text: self.text,
                    },
                    Self {
                        offset: self.offset + bytes_len + c.len_utf8(),
                        last: self.last,
                        line: self.line + nl,
                        char_count: self.char_count - char_count - 1,
                        text: self.text,
                    },
                ));
            }

            bytes_len += c.len_utf8();
        }

        None
    }

    fn skip_until_nl(self) -> Self {
        let s = self.as_str();
        if let Some(off) = memchr::memchr(b'\n', s.as_bytes()) {
            let off = off + 1;

            let skipped_chars = s[..off].count_chars();

            Self {
                offset: self.offset + off,
                last: self.last,
                line: self.line + 1,
                char_count: self.char_count - skipped_chars,
                text: self.text,
            }
        } else {
            Self {
                offset: self.last,
                last: self.last,
                line: self.line,
                char_count: 0,
                text: self.text,
            }
        }
    }
}

impl Deref for StringReader<'_> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for StringReader<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self
    }
}

impl Borrow<str> for StringReader<'_> {
    #[inline]
    fn borrow(&self) -> &str {
        self
    }
}

impl PartialEq<str> for StringReader<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq for StringReader<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<&str> for StringReader<'_> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

impl PartialEq<&mut str> for StringReader<'_> {
    #[inline]
    fn eq(&self, other: &&mut str) -> bool {
        self == *other
    }
}

impl PartialEq<String> for StringReader<'_> {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<&String> for StringReader<'_> {
    #[inline]
    fn eq(&self, other: &&String) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<&mut String> for StringReader<'_> {
    #[inline]
    fn eq(&self, other: &&mut String) -> bool {
        self == other.as_str()
    }
}

impl Eq for StringReader<'_> {}

impl PartialOrd<str> for StringReader<'_> {
    #[inline]
    fn partial_cmp(&self, other: &str) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other)
    }
}

impl PartialOrd for StringReader<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.partial_cmp(other.as_str())
    }
}

impl PartialOrd<&str> for StringReader<'_> {
    #[inline]
    fn partial_cmp(&self, other: &&str) -> Option<std::cmp::Ordering> {
        self.partial_cmp(*other)
    }
}

impl PartialOrd<&mut str> for StringReader<'_> {
    #[inline]
    fn partial_cmp(&self, other: &&mut str) -> Option<std::cmp::Ordering> {
        self.partial_cmp(*other)
    }
}

impl PartialOrd<String> for StringReader<'_> {
    #[inline]
    fn partial_cmp(&self, other: &String) -> Option<std::cmp::Ordering> {
        self.partial_cmp(other.as_str())
    }
}

impl PartialOrd<&String> for StringReader<'_> {
    #[inline]
    fn partial_cmp(&self, other: &&String) -> Option<std::cmp::Ordering> {
        self.partial_cmp(other.as_str())
    }
}

impl PartialOrd<&mut String> for StringReader<'_> {
    #[inline]
    fn partial_cmp(&self, other: &&mut String) -> Option<std::cmp::Ordering> {
        self.partial_cmp(other.as_str())
    }
}

impl Ord for StringReader<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

// impl fmt::Debug for StringReader<'_> {
//     #[inline]
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         fmt::Debug::fmt(self.as_str(), f)
//     }
// }

impl fmt::Display for StringReader<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice() {
        let reader = StringReader::new("ぁあぃ");
        assert_eq!(reader.len(), 3);

        {
            let sub2 = {
                // testing lifetime
                let sub1 = reader.get(1..);
                assert!(sub1.is_some());
                let sub1 = sub1.unwrap();
                assert_eq!(sub1.len(), 2);
                assert_eq!(sub1, "あぃ");
                sub1.get(1..)
            };

            assert!(sub2.is_some());
            let sub2 = sub2.unwrap();
            assert_eq!(sub2.len(), 1);
            assert_eq!(sub2, "ぃ");
        }

        assert!(reader.get(4..).is_none());
        assert!(reader.get(..4).is_none());
        assert!(reader.get(..).is_some());
        assert_eq!(reader.get(..).unwrap(), reader.as_str());
        assert!(reader.get(1..=4).is_none());
        assert!(reader.get(..=4).is_none());
        assert!(reader.get(1..2).is_some());
        assert_eq!(reader.get(1..2).unwrap(), "あ");
    }

    #[test]
    fn line() {
        let reader = StringReader::new("\n\n\n\nあtest\n");
        assert_eq!(reader.line(), 1);
        assert!(reader.get(1..).is_some());
        assert_eq!(reader.get(1..).unwrap().line(), 2);
        assert!(reader.get(2..).is_some());
        assert_eq!(reader.get(2..).unwrap().line(), 3);
        assert!(reader.get(3..).is_some());
        assert_eq!(reader.get(3..).unwrap().line(), 4);
        assert!(reader.get(4..).is_some());
        assert_eq!(reader.get(4..).unwrap().line(), 5);
        assert!(reader.get(4..).is_some());
        assert_eq!(reader.get(4..).unwrap().line(), 5);
        assert!(reader.get(8..).is_some());
        assert_eq!(reader.get(8..).unwrap().line(), 5);

        assert_eq!(reader.get(4..).unwrap().line_str(), "あtest");
        assert_eq!(reader.get(4..).unwrap().column(), 1);
        assert_eq!(reader.get(5..).unwrap().column(), 3);

        let r = reader.get(10..10);
        assert!(r.is_some());
        let r = r.unwrap();
        assert_eq!(r.line(), 6);
        assert_eq!(r.len(), 0);
        assert_eq!(r, "");
        assert_eq!(r.as_str().len(), 0);
        assert_eq!(r.location(), Location { line: 6, column: 1 });
    }
}
