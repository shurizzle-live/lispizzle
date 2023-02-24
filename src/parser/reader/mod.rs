mod input;
mod str_reader;
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

fn skip_ws(i: Input<'_>) -> std::result::Result<Input<'_>, Error> {
    if i.needs_ws() {
        if let Some(c) = i.get(0) {
            if !(c.is_whitespace() || c == '(' || c == ')') {
                return Err(i.err("expected space or list"));
            }
        }
    }

    let skipped = i
        .as_str()
        .chars()
        .take_while(|&c| c.is_whitespace())
        .count();

    Ok(i.get(skipped..).unwrap().unset_needs_ws())
}

const INVALID_SYM_CHARS: &str = ",'@`()\"|#";

#[inline(always)]
fn is_valid_sym_char(c: char) -> bool {
    !(INVALID_SYM_CHARS.contains(c) || c.is_whitespace())
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

    let parsed = unsafe { i.get_unchecked(..len) };
    let i = unsafe { i.get_unchecked(len..) }.set_needs_ws();

    if is_integer {
        let e = i.clone();
        i.ok(Integer::parse(parsed.as_str())
            .map_err(|_| e.err("invalid number"))?
            .complete()
            .into())
    } else {
        i.ok(Value::Symbol(parsed.as_str().into()))
    }
}

fn needs_char(i: Input<'_>, need: char) -> std::result::Result<Input<'_>, Error> {
    if let Some(c) = i.get(0) {
        if c == need {
            Ok(unsafe { i.get_unchecked(1..) })
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

fn needs_char_ci(i: Input<'_>, need: char) -> std::result::Result<Input<'_>, Error> {
    if let Some(c) = i.get(0) {
        if char_equals_ci(c, need) {
            Ok(unsafe { i.get_unchecked(1..) })
        } else {
            Err(i.err("unexpected character"))
        }
    } else {
        Err(i.err("unexpected EOF"))
    }
}

fn string(i: Input<'_>) -> Result<Value> {
    let mut i = needs_char(i, '"')?;
    let mut escaping = false;
    let mut res = EcoString::new();

    loop {
        if let Some(c) = i.get(0) {
            i = unsafe { i.get_unchecked(1..) };

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
                    _ => return Err(i.err("unexpected escaped symbol")),
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

    if let Some(c) = i.get(0) {
        let i = unsafe { i.get_unchecked(1..) };

        match c {
            't' | 'T' => i.set_needs_ws().ok(true.into()),
            'f' | 'F' => i.set_needs_ws().ok(false.into()),
            'n' => needs_char(needs_char(i, 'i')?, 'l')?
                .set_needs_ws()
                .ok(Value::Nil),
            _ => Err(i.err("unexpected character")),
        }
    } else {
        Err(i.err("unexpected EOF"))
    }
}

fn literal(i: Input<'_>) -> Result<Value> {
    if let Some(c) = i.get(0) {
        match c {
            '"' => return string(i),
            '#' => return nil_or_bool(i),
            _ => (),
        }
    }

    symbol_or_integer(i)
}

fn list(mut i: Input<'_>) -> Result<Value> {
    i = if let Some(c) = i.get(0) {
        if c == '(' {
            i.get(1..).unwrap().unset_needs_ws()
        } else {
            return Err(i.err("expected `('"));
        }
    } else {
        return Err(i.err("unexpected EOF"));
    };

    let mut values = Vector::new();

    loop {
        i = skip_ws(i)?;
        if let Some(c) = i.get(0) {
            if c == ')' {
                return Ok((i.get(1..).unwrap().unset_needs_ws(), Value::List(values)));
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

fn expression(mut i: Input<'_>) -> Result<Value> {
    i = skip_ws(i)?;

    if let Some(c) = i.get(0) {
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
}
