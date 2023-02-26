mod input;
mod str_reader;
#[cfg(feature = "benchmarking")]
pub mod util;
#[cfg(not(feature = "benchmarking"))]
mod util;

use ecow::EcoString;
use im_rc::Vector;
pub use input::*;
pub use str_reader::*;

use rug::{Complete, Integer};

use crate::Value;

use input::Input;

use super::Error;

// ❌ Character  #\...
//                 <octal>
//                 x<hex>
//                 name
//                 char
// ✔️  Nil        #nil
// ✔️  Boolean    #[tf]
// ✔️  String     "([^"]|\\["vtrn\\])"
// ✔️  Integer    [+-][0-9]+
// ✔️  Symbol     [^\s,'@`()\"|#]+
// ✔️  List       ((list|literal)*)

type Result<'a, T> = std::result::Result<(Input<'a>, T), Error>;

const INVALID_SYM_CHARS: &str = ",'@`()\"|#";

fn skip_ws(mut i: Input<'_>) -> std::result::Result<Input<'_>, Error> {
    let old_len = i.len();
    i = i.ltrim();

    if i.len() < old_len {
        i = i.unset_needs_ws();
    }

    if i.needs_ws() {
        if let Some(c) = i.peek() {
            if c == '(' || c == ')' {
                i = i.unset_needs_ws();
            } else {
                return Err(i.err("expected space character or list"));
            }
        }
    }

    Ok(i.unset_needs_ws())
}

fn split_at(i: Input<'_>, index: usize) -> Option<(Input<'_>, Input<'_>)> {
    i.get(..index)
        .map(|i1| (i1, unsafe { i.get_unchecked(index..) }))
}

fn next_char(i: Input<'_>) -> Option<(char, Input<'_>)> {
    i.peek().map(|c| (c, unsafe { i.get_unchecked(1..) }))
}

#[inline(always)]
fn is_valid_sym_char(c: char) -> bool {
    !(INVALID_SYM_CHARS.contains(c) || c.is_whitespace())
}

fn starts_with_ci(haystack: &str, need: &str) -> bool {
    let mut haystack = haystack.chars();
    let mut need = need.chars();

    loop {
        match (haystack.next(), need.next()) {
            (Some(a), Some(b)) => {
                if !char_equals_ci(a, b) {
                    return false;
                }
            }
            (Some(_), None) | (None, None) => return true,
            (None, Some(_)) => return false,
        }
    }
}

fn istarts_with<'a>(i: Input<'a>, need: &str) -> Option<Input<'a>> {
    if i.as_str().starts_with(need) {
        Some(unsafe { i.get_unchecked(need.len()..) })
    } else {
        None
    }
}

fn istarts_with_ci<'a>(i: Input<'a>, need: &str) -> Option<Input<'a>> {
    if starts_with_ci(i.as_str(), need) {
        Some(unsafe { i.get_unchecked(need.len()..) })
    } else {
        None
    }
}

fn needs_char(i: Input<'_>, need: char) -> std::result::Result<Input<'_>, Error> {
    if let Some((c, rest)) = next_char(i.clone()) {
        if c == need {
            Ok(rest)
        } else {
            Err(i.err("unexpected character"))
        }
    } else {
        Err(i.err("unexpected EOF"))
    }
}

fn char_equals_ci(a: char, b: char) -> bool {
    let a = a.to_lowercase();
    let b = b.to_lowercase();

    a.len() == b.len() && a.zip(b).all(|(a, b)| a == b)
}

fn str_equals_ci(a: &str, b: &str) -> bool {
    let mut a = a.chars();
    let mut b = b.chars();

    loop {
        match (a.next(), b.next()) {
            (Some(a), Some(b)) => {
                if !char_equals_ci(a, b) {
                    return false;
                }
            }
            (Some(_), None) | (None, Some(_)) => return false,
            (None, None) => return true,
        }
    }
}

#[test]
fn eqci() {
    assert!(char_equals_ci('ẞ', 'ß'));
    assert!(str_equals_ci("aẞ", "Aß"));
}

fn needs_char_ci(i: Input<'_>, need: char) -> std::result::Result<Input<'_>, Error> {
    if let Some((c, rest)) = next_char(i.clone()) {
        if char_equals_ci(c, need) {
            Ok(rest)
        } else {
            Err(i.err("unexpected character"))
        }
    } else {
        Err(i.err("unexpected EOF"))
    }
}

fn symbol_or_integer(i: Input<'_>) -> Result<Value> {
    let (len, is_integer) = {
        let mut is_integer = false;
        let mut sign = false;
        let mut len = 0;

        for (i, c) in i
            .as_str()
            .chars()
            .take_while(|&c| is_valid_sym_char(c))
            .enumerate()
        {
            if is_integer || sign || i == 0 {
                is_integer = c.is_ascii_digit();
            }

            if i == 0 && (c == '-' || c == '+') {
                sign = true;
            }

            len = i + 1;
        }

        (len, is_integer)
    };

    if len == 0 {
        return Err(i.err("unexpected character"));
    }

    let (parsed, i) = unsafe { split_at(i, len).unwrap_unchecked() };

    if is_integer {
        i.ok(Integer::parse(parsed.as_str())
            .map_err(|_| parsed.err("invalid number"))?
            .complete()
            .into())
    } else {
        i.ok(Value::Symbol(parsed.as_str().into()))
    }
}

fn string(i: Input<'_>) -> Result<Value> {
    let mut i = needs_char(i, '"')?;
    let mut escaping = false;
    let mut res = EcoString::new();

    loop {
        let prev_input = i.clone();
        if let Some((c, new_i)) = next_char(i.clone()) {
            i = new_i;

            if escaping {
                escaping = false;
                let c = match c {
                    'v' => '\u{0b}',
                    't' => '\t',
                    'r' => '\r',
                    'n' => '\n',
                    'a' => '\u{7}',
                    '\\' => '\\',
                    '0' => '\0',
                    '"' => '"',
                    _ => return Err(prev_input.err("unexpected escaped symbol")),
                };
                res.push(c);
            } else {
                match c {
                    '\\' => {
                        escaping = true;
                    }
                    '"' => {
                        return i.set_needs_ws().ok(res.into());
                    }
                    _ => {
                        res.push(c);
                    }
                }
            }
        } else {
            return Err(i.err("unexpected EOF"));
        }
    }
}

fn nil_or_bool(i: Input<'_>) -> Result<Value> {
    let i = needs_char(i, '#')?;

    if let Some(i) = istarts_with(i.clone(), "nil") {
        i.set_needs_ws().ok(Value::Nil)
    } else if let Some(i) = istarts_with_ci(i.clone(), "t") {
        i.set_needs_ws().ok(true.into())
    } else if let Some(i) = istarts_with_ci(i.clone(), "f") {
        i.set_needs_ws().ok(false.into())
    } else if i.is_empty() {
        Err(i.err("unexpected EOF"))
    } else {
        Err(i.err("unexpected character"))
    }
}

fn literal(i: Input<'_>) -> Result<Value> {
    if let Some(c) = i.peek() {
        match c {
            '"' => return string(i),
            '#' => return nil_or_bool(i),
            _ => (),
        }
    }

    symbol_or_integer(i)
}

fn list(mut i: Input<'_>) -> Result<Value> {
    i = if let Some(i) = istarts_with(i.clone(), "(") {
        i.unset_needs_ws()
    } else if i.is_empty() {
        return Err(i.err("unexpected EOF"));
    } else {
        return Err(i.err("expected `('"));
    };

    let mut values = Vector::new();

    loop {
        i = skip_ws(i)?;
        if let Some((c, new_i)) = next_char(i.clone()) {
            if c == ')' {
                return Ok((new_i.unset_needs_ws(), Value::List(values)));
            } else {
                let v;
                (i, v) = expression(i)?;
                values.push_back(v);
            }
        } else {
            return Err(i.err("expected `)'"));
        }
    }
}

fn expression(i: Input<'_>) -> Result<Value> {
    if let Some(c) = i.peek() {
        if c == '(' {
            list(i)
        } else {
            literal(i)
        }
    } else {
        Err(i.err("unexpected EOF"))
    }
}

fn is_eoc(mut i: Input<'_>) -> Result<bool> {
    i = skip_ws(i)?;
    let res = !i.is_empty();
    i.ok(res)
}

pub fn parse(mut i: Input<'_>) -> std::result::Result<Vec<Value>, Error> {
    let mut prog = Vec::new();

    while {
        let ok;
        (i, ok) = is_eoc(i)?;
        ok
    } {
        let e;
        (i, e) = list(i)?;
        prog.push(e);
    }

    Ok(prog)
}

#[cfg(test)]
mod tests {
    use im_rc::vector;

    use super::*;

    macro_rules! assert_fp_eq {
        ($e1:expr, $e2:expr) => {{
            let res = $e1;
            match res {
                Ok((i, res)) => {
                    if !i.is_empty() {
                        panic!(
                            "assertion failed: `{}` has not been fully parsed",
                            stringify!($e1)
                        );
                    }
                    let expected = $e2;
                    if res != expected {
                        panic!(
                            "assertion failed: `(left == right)`\n  left: `{:?}`\n right: `{:?}`",
                            res, expected
                        );
                    }
                }
                Err(e) => {
                    panic!(
                        "assertion failed: `{}` returned an error\n error: `{:?}`",
                        stringify!($e1),
                        e
                    )
                }
            }
        }};
    }

    #[test]
    fn int() {
        assert_fp_eq!(symbol_or_integer(Input::new(None, "-0000000")), 0.into());
        assert_fp_eq!(symbol_or_integer(Input::new(None, "-1")), (-1).into());
        assert_fp_eq!(symbol_or_integer(Input::new(None, "1")), 1.into());
    }

    #[test]
    fn symbol() {
        assert_fp_eq!(
            symbol_or_integer(Input::new(None, "-0000000test")),
            Value::Symbol("-0000000test".into())
        );
    }

    #[test]
    fn parse_list() {
        let expected: Value = vector![1.into(), vector![2.into(), 3.into()].into()].into();

        assert_fp_eq!(list(Input::new(None, "(1(2 3))")), expected.clone());
        assert_fp_eq!(list(Input::new(None, "(1    (2    3)  )")), expected);
    }
}
