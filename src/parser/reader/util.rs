use ecow::{EcoString, EcoVec};

const USIZE_BYTES: usize = (usize::BITS / u8::BITS) as usize;
const USIZE_COUNT_MASK: usize = usize::from_be_bytes([0x40_u8; USIZE_BYTES]);
const NEWLINE: u8 = b'\n';

// non-boundary bytes starts with 0b10xxxxxx
// boundary bytes start with 0b0xxxxxxx or 0b11xxxxxx
#[inline(always)]
const fn is_boundary(c: u8) -> bool {
    c & 0b11000000 != 0b10000000
}

#[inline]
fn _count_u8(s: &[u8]) -> usize {
    let mut count = 0;
    for &c in s {
        count += is_boundary(c) as usize;
    }
    count
}

// If we take only the first 2 bits of each byte (the first is `a`, the second
// is `b`) we can see that the only (!a|b) combination that gives 0 is 10, the
// non-boundary one. So we can apply this concept to n (n | (!n >> 1)), filter
// all the non relevants bytes and count the ones. Karnaugh map below.
//
// OK: 00 -> !0|0 -> 1|0 -> 1
// OK: 01 -> !0|1 -> 1|1 -> 1
// KO: 10 -> !1|0 -> 0|0 -> 0
// OK: 11 -> !1|1 -> 0|1 -> 1
#[inline(always)]
const fn count_boundaries(n: usize) -> usize {
    ((n | (!n >> 1)) & USIZE_COUNT_MASK).count_ones() as usize
}

#[inline]
fn _count_usize(s: &[usize]) -> usize {
    let mut sum = 0;
    for &c in s {
        sum += count_boundaries(c);
    }
    sum
}

pub fn count_chars(s: &[u8]) -> usize {
    let (pre, mid, post) = match s.len() {
        15..=usize::MAX => unsafe {
            let (pre, mid, post) = s.align_to::<usize>();
            (pre, _count_usize(mid), post)
        },
        1 => return 1,
        0 => return 0,
        _ => return _count_u8(s),
    };

    _count_u8(pre) + _count_u8(post) + mid
}

fn _skip_chars_u8(s: &[u8], mut n: usize) -> Option<&[u8]> {
    if n == 0 {
        return Some(s);
    }

    for (off, c) in s.iter().enumerate() {
        if is_boundary(*c) {
            if n == 0 {
                return Some(unsafe { s.get_unchecked(off..) });
            }

            n -= 1;
        }
    }

    if n == 0 {
        Some(unsafe { s.get_unchecked(s.len()..) })
    } else {
        None
    }
}

pub fn skip_chars(s: &[u8], mut n: usize) -> Option<&[u8]> {
    if n > s.len() {
        return None;
    }

    let (mut haystack, post) = match n {
        15..=usize::MAX => unsafe {
            let (pre, mid, post) = s.align_to::<usize>();
            n -= _count_u8(pre);
            (mid, post)
        },
        0 => return Some(s),
        _ => return _skip_chars_u8(s, n),
    };

    while n >= USIZE_BYTES && !haystack.is_empty() {
        unsafe {
            n -= count_boundaries(*haystack.get_unchecked(0));
            haystack = haystack.get_unchecked(1..);
        }
    }

    let rest = if haystack.is_empty() {
        post
    } else {
        let len = (post.as_ptr() as *const _ as usize) - (haystack.as_ptr() as *const _ as usize)
            + post.len();
        unsafe { std::slice::from_raw_parts(haystack.as_ptr() as *const u8, len) }
    };

    _skip_chars_u8(rest, n)
}

#[inline(always)]
const fn mask_count_char(n: usize, c: u8) -> usize {
    let usize_mask = usize::from_be_bytes([!c; USIZE_BYTES]);

    let x = n ^ usize_mask;
    let x1 = x;
    let x2 = x1 << 1;
    let x3 = x2 << 1;
    let x4 = x3 << 1;
    let x5 = x4 << 1;
    let x6 = x5 << 1;
    let x7 = x6 << 1;
    let x8 = x7 << 1;

    x1 & x2 & x3 & x4 & x5 & x6 & x7 & x8
}

#[inline(always)]
const fn mask_first_bits(n: usize) -> usize {
    n & usize::from_be_bytes([0b10000000; USIZE_BYTES])
}

#[inline(always)]
const fn count_first_bits(n: usize) -> usize {
    mask_first_bits(n).count_ones() as usize
}

#[inline(always)]
const fn count_char(n: usize, c: u8) -> usize {
    count_first_bits(mask_count_char(n, c))
}

#[inline(always)]
const fn count_nl(n: usize) -> usize {
    count_char(n, b'\n')
}

// #[cfg(any(
//     target_pointer_width = "8",
//     target_pointer_width = "16",
//     target_pointer_width = "32",
//     target_pointer_width = "64"
// ))]
// #[inline(always)]
// const fn group_first_bits(mut n: usize) -> usize {
//     let mut i = 1;
//     while i < USIZE_BYTES {
//         n = n | (n << (i * 7));
//         i += 1;
//     }
//     n
// }
//
// #[cfg(not(any(
//     target_pointer_width = "8",
//     target_pointer_width = "16",
//     target_pointer_width = "32",
//     target_pointer_width = "64"
// )))]
// #[inline(always)]
// const fn group_first_bits(n: usize) -> usize {
//     let mut res = 0;
//     let mut i = 1;
//     while i < USIZE_BYTES {
//         let shift = i * 7;
//         let mask = usize::from_be_bytes([0; USIZE_BYTES]) | (1 << shift);
//         res |= (n << shift) & mask;
//         i += 1;
//     }
//     res
// }

#[test]
fn cln() {
    for i in 0..=u8::MAX {
        let all = usize::from_be_bytes([i; USIZE_BYTES]);
        assert_eq!(count_nl(all), if i == 10 { USIZE_BYTES } else { 0 });
    }
    let mut buf = [0b11111111; USIZE_BYTES];
    buf[USIZE_BYTES / 2] = b'\n';
    assert_eq!(count_nl(usize::from_be_bytes(buf)), 1);
}

#[inline(always)]
const fn _count_nl(n: usize) -> usize {
    unsafe {
        let mut count = 0;
        let v = &n as *const _ as *const u8;
        let mut i = 0;
        while i < USIZE_BYTES {
            count += (*v.add(i) == NEWLINE) as usize;
            i += 1;
        }
        count
    }
}

#[inline(always)]
fn count_nl_u8(s: &[u8]) -> (usize, usize) {
    let mut nl = 0;
    let mut cs = 0;
    for &c in s {
        nl += (c == NEWLINE) as usize;
        cs += is_boundary(c) as usize;
    }
    (cs, nl)
}

#[inline(always)]
const fn count_boundaries_nl(n: usize) -> (usize, usize) {
    (count_boundaries(n), count_nl(n))
}

fn _skip_chars_nl_u8(s: &[u8], mut n: usize) -> Option<(&[u8], usize)> {
    if n == 0 {
        return Some((s, 0));
    }

    let mut count = 0;
    for (off, c) in s.iter().enumerate() {
        if is_boundary(*c) {
            if n == 0 {
                return Some((unsafe { s.get_unchecked(off..) }, count));
            }
            n -= 1;
            count += (*c == NEWLINE) as usize;
        }
    }

    if n == 0 {
        Some((unsafe { s.get_unchecked(s.len()..) }, count))
    } else {
        None
    }
}

pub fn skip_chars_count_nl(s: &[u8], mut n: usize) -> Option<(&[u8], usize)> {
    if n > s.len() {
        return None;
    }

    let mut nl = 0;

    let (mut haystack, post) = match n {
        15..=usize::MAX => unsafe {
            let (pre, mid, post) = s.align_to::<usize>();
            let (c1, c2) = count_nl_u8(pre);
            n -= c1;
            nl += c2;
            (mid, post)
        },
        0 => return Some((s, 0)),
        _ => return _skip_chars_nl_u8(s, n),
    };

    while n >= USIZE_BYTES && !haystack.is_empty() {
        unsafe {
            let (c1, c2) = count_boundaries_nl(*haystack.get_unchecked(0));
            n -= c1;
            nl += c2;
            haystack = haystack.get_unchecked(1..);
        }
    }

    let rest = if haystack.is_empty() {
        post
    } else {
        let len = (post.as_ptr() as *const _ as usize) - (haystack.as_ptr() as *const _ as usize)
            + post.len();
        unsafe { std::slice::from_raw_parts(haystack.as_ptr() as *const u8, len) }
    };

    _skip_chars_nl_u8(rest, n).map(|(new, c)| (new, nl + c))
}

pub trait SkipChars {
    type Output;

    fn skip_chars(self, n: usize) -> Option<Self::Output>;

    fn skip_chars_count_nl(self, n: usize) -> Option<(Self::Output, usize)>;
}

impl<'a> SkipChars for &'a [u8] {
    type Output = &'a [u8];

    #[inline(always)]
    fn skip_chars(self, n: usize) -> Option<Self::Output> {
        skip_chars(self, n)
    }

    #[inline]
    fn skip_chars_count_nl(self, n: usize) -> Option<(Self::Output, usize)> {
        skip_chars_count_nl(self, n)
    }
}

impl<'a> SkipChars for &'a mut [u8] {
    type Output = &'a mut [u8];

    #[inline(always)]
    fn skip_chars(self, n: usize) -> Option<Self::Output> {
        unsafe { std::mem::transmute(skip_chars(self, n)) }
    }

    #[inline]
    #[allow(mutable_transmutes)]
    fn skip_chars_count_nl(self, n: usize) -> Option<(Self::Output, usize)> {
        skip_chars_count_nl(self, n).map(|(s, c)| (unsafe { std::mem::transmute(s) }, c))
    }
}

impl<'a> SkipChars for &'a Vec<u8> {
    type Output = &'a [u8];

    #[inline]
    fn skip_chars(self, n: usize) -> Option<Self::Output> {
        self.as_slice().skip_chars(n)
    }

    #[inline]
    fn skip_chars_count_nl(self, n: usize) -> Option<(Self::Output, usize)> {
        self.as_slice().skip_chars_count_nl(n)
    }
}

impl<'a> SkipChars for &'a mut Vec<u8> {
    type Output = &'a mut [u8];

    #[inline]
    fn skip_chars(self, n: usize) -> Option<Self::Output> {
        self.as_mut_slice().skip_chars(n)
    }

    #[inline]
    fn skip_chars_count_nl(self, n: usize) -> Option<(Self::Output, usize)> {
        self.as_mut_slice().skip_chars_count_nl(n)
    }
}

impl<'a> SkipChars for &'a str {
    type Output = &'a str;

    #[inline]
    fn skip_chars(self, n: usize) -> Option<Self::Output> {
        unsafe {
            Some(std::str::from_utf8_unchecked(
                self.as_bytes().skip_chars(n)?,
            ))
        }
    }

    #[inline]
    fn skip_chars_count_nl(self, n: usize) -> Option<(Self::Output, usize)> {
        self.as_bytes()
            .skip_chars_count_nl(n)
            .map(|(s, n)| (unsafe { std::str::from_utf8_unchecked(s) }, n))
    }
}

impl<'a> SkipChars for &'a mut str {
    type Output = &'a mut str;

    #[inline]
    fn skip_chars(self, n: usize) -> Option<Self::Output> {
        unsafe {
            Some(std::str::from_utf8_unchecked_mut(
                self.as_bytes_mut().skip_chars(n)?,
            ))
        }
    }

    #[inline]
    fn skip_chars_count_nl(self, n: usize) -> Option<(Self::Output, usize)> {
        unsafe { self.as_bytes_mut() }
            .skip_chars_count_nl(n)
            .map(|(s, n)| (unsafe { std::str::from_utf8_unchecked_mut(s) }, n))
    }
}

impl<'a> SkipChars for &'a String {
    type Output = &'a str;

    #[inline]
    fn skip_chars(self, n: usize) -> Option<Self::Output> {
        self.as_str().skip_chars(n)
    }

    #[inline]
    fn skip_chars_count_nl(self, n: usize) -> Option<(Self::Output, usize)> {
        self.as_str().skip_chars_count_nl(n)
    }
}

impl<'a> SkipChars for &'a mut String {
    type Output = &'a mut str;

    #[inline]
    fn skip_chars(self, n: usize) -> Option<Self::Output> {
        self.as_mut_str().skip_chars(n)
    }

    #[inline]
    fn skip_chars_count_nl(self, n: usize) -> Option<(Self::Output, usize)> {
        self.as_mut_str().skip_chars_count_nl(n)
    }
}

impl<'a> SkipChars for &'a EcoString {
    type Output = &'a str;

    #[inline]
    fn skip_chars(self, n: usize) -> Option<Self::Output> {
        self.as_str().skip_chars(n)
    }

    #[inline]
    fn skip_chars_count_nl(self, n: usize) -> Option<(Self::Output, usize)> {
        self.as_str().skip_chars_count_nl(n)
    }
}

pub trait CountChars {
    fn count_chars(&self) -> usize;
}

impl CountChars for [u8] {
    #[inline]
    fn count_chars(&self) -> usize {
        count_chars(self)
    }
}

impl CountChars for str {
    #[inline]
    fn count_chars(&self) -> usize {
        self.as_bytes().count_chars()
    }
}

impl CountChars for String {
    #[inline]
    fn count_chars(&self) -> usize {
        self.as_bytes().count_chars()
    }
}

impl CountChars for EcoVec<u8> {
    #[inline]
    fn count_chars(&self) -> usize {
        self.as_slice().count_chars()
    }
}

impl CountChars for EcoString {
    #[inline]
    fn count_chars(&self) -> usize {
        self.as_bytes().count_chars()
    }
}
