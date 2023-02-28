mod input;
mod str_reader;
#[cfg(feature = "benchmarking")]
pub mod util;
#[cfg(not(feature = "benchmarking"))]
mod util;

use ecow::EcoString;
use im_rc::Vector;
use phf::phf_map;

use rug::{Complete, Integer};

use crate::{Str, Symbol, Value};

pub use input::Input;

use super::Error;

use str_reader::*;

// ✔️  Character  #\...
//                 <octal>
//                 x<hex>
//                 name
//                 char
// ✔️  Nil        #nil
// ✔️  Boolean    #[tf]
// ✔️  String     "([^"]|\\(["vtrn\\]|x[0-9a-zA-Z]{2}|u([0-9a-zA-Z]{4}|\{u[0-9a-zA-Z]+\})))"
// ✔️  Integer    [+-][0-9]+
// ✔️  Symbol     [^\s,'@`()\"|#]+
// ✔️  List       ((list|literal)*)

type Result<'a, T> = std::result::Result<(Input<'a>, T), Error>;

const INVALID_SYM_CHARS: &str = ",'@`()\"|#";

const CHAR_NAME_TO_CODEPOINT: phf::Map<&'static str, char> = phf_map! {
    "nul" => 0x00 as char,
    "null" => 0x00 as char,
    "soh" => 0x01 as char,
    "stx" => 0x02 as char,
    "etx" => 0x03 as char,
    "eot" => 0x04 as char,
    "enq" => 0x05 as char,
    "ack" => 0x06 as char,
    "bel" => 0x07 as char,
    "bell" => 0x07 as char,
    "bs" => 0x08 as char,
    "backspace" => 0x08 as char,
    "ht" => 0x09 as char,
    "tab" => 0x09 as char,
    "linefeed" => 0x0A as char,
    "nl" => 0x0A as char,
    "newline" => 0x0A as char,
    "vt" => 0x0B as char,
    "page" => 0x0C as char,
    "np" => 0x0C as char,
    "return" => 0x0D as char,
    "cr" => 0x0D as char,
    "so" => 0x0E as char,
    "si" => 0x0F as char,
    "dle" => 0x10 as char,
    "dc1" => 0x11 as char,
    "dc2" => 0x12 as char,
    "dc3" => 0x13 as char,
    "dc4" => 0x14 as char,
    "nak" => 0x15 as char,
    "syn" => 0x16 as char,
    "etb" => 0x17 as char,
    "can" => 0x18 as char,
    "em" => 0x19 as char,
    "sub" => 0x1A as char,
    "esc" => 0x1B as char,
    "escape" => 0x1B as char,
    "fs" => 0x1C as char,
    "gs" => 0x1D as char,
    "rs" => 0x1E as char,
    "us" => 0x1F as char,
    "space" => 0x20 as char,
    "sp" => 0x20 as char,
    "rubout" => 0x7F as char,
    "delete" => 0x7F as char,
    "del" => 0x7F as char,
};

fn skip_ws(mut i: Input) -> std::result::Result<Input, Error> {
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

fn split_at(i: Input, index: usize) -> Option<(Input, Input)> {
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
        i.ok(Value::Symbol(Symbol::Name(
            EcoString::from(parsed.as_str()).into(),
        )))
    }
}

fn parse_nibble(i: Input) -> Option<(Input, u8)> {
    next_char(i).and_then(|(c, i)| c.to_digit(16).map(|c| (i, c as u8)))
}

fn parse_4hex_char(i: Input) -> Option<(Input, char)> {
    parse_nibble(i).and_then(|(i, c1)| {
        parse_nibble(i).and_then(|(i, c2)| {
            parse_nibble(i).and_then(|(i, c3)| {
                parse_nibble(i).map(|(i, c4)| {
                    (i, unsafe {
                        char::from_u32_unchecked(
                            ((((((c1 as u32) << 4) | (c2 as u32)) << 4) | (c3 as u32)) << 4)
                                | (c4 as u32),
                        )
                    })
                })
            })
        })
    })
}

#[inline]
fn parse_codepoint(i: Input) -> Option<char> {
    if let Ok(c) = u32::from_str_radix(i.as_str(), 16) {
        char::from_u32(c)
    } else {
        None
    }
}

fn parse_string_codepoint<'a>(init: Input<'a>, i: Input<'a>) -> Result<'a, char> {
    const ERR: &str = "invalid UTF-8 character escape sequence";
    if let Some(('{', i)) = next_char(i.clone()) {
        i.split_at(|c| c == '}')
            .and_then(|(codepoint, i)| parse_codepoint(codepoint).map(|c| (i, c)))
            .ok_or_else(|| init.err(ERR))
    } else if let Some((i, c)) = parse_4hex_char(i) {
        i.ok(c)
    } else {
        Err(init.err(ERR))
    }
}

fn string(i: Input<'_>) -> Result<Value> {
    let mut i = needs_char(i, '"')?;
    let mut escaping = false;
    let mut res = EcoString::new();
    let mut prev_input = i.clone();

    loop {
        let pinput = i.clone();
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
                    'x' => {
                        let c;
                        (i, c) = parse_nibble(i)
                            .and_then(|(i, c1)| parse_nibble(i).map(|(i, c2)| (i, (c1 << 4) | c2)))
                            .ok_or_else(|| {
                                prev_input.err("invalid ascii character escape sequence")
                            })
                            .map(|(i, c)| (i, c as char))?;
                        c
                    }
                    'u' => {
                        let c;
                        (i, c) = parse_string_codepoint(prev_input, i)?;
                        c
                    }
                    _ => return Err(prev_input.err("unexpected escaped symbol")),
                };
                res.push(c);
            } else {
                match c {
                    '\\' => {
                        escaping = true;
                    }
                    '"' => {
                        return i.set_needs_ws().ok(Str::from(res).into());
                    }
                    _ => {
                        res.push(c);
                    }
                }
            }
        } else {
            return Err(i.err("unexpected EOF"));
        }
        prev_input = pinput;
    }
}

fn parse_char<'a>(init: Input<'a>, i: Input<'a>) -> Result<'a, Value> {
    if let Some((c, i)) = next_char(i.clone()) {
        if c == ' ' {
            return i.unset_needs_ws().ok(' '.into());
        } else if c == '(' && c == ')' {
            return i.set_needs_ws().ok(' '.into());
        }
    }

    let (i, rest) = if let Some(x) = i
        .clone()
        .split_at(|c| c.is_whitespace() || c == '(' || c == ')')
    {
        x
    } else {
        (i.clone(), unsafe { i.get(i.len()..).unwrap_unchecked() })
    };
    let rest = rest.set_needs_ws();

    if i.is_empty() {
        return Err(init.err("invalid character"));
    }

    if let Some(c) = CHAR_NAME_TO_CODEPOINT
        .get(&i.as_str().to_lowercase())
        .copied()
    {
        return rest.ok(c.into());
    }

    if i.len() == 1 {
        return rest.ok(unsafe { i.get_unchecked(0) }.into());
    }

    {
        if let Ok(c) = u32::from_str_radix(i.as_str(), 8) {
            if let Some(c) = char::from_u32(c) {
                return rest.ok(c.into());
            }
        }
    }

    if let Some(('x', i)) = next_char(i) {
        return if let Some(c) = parse_codepoint(i) {
            rest.ok(c.into())
        } else {
            Err(init.err("invalid codepoint"))
        };
    }

    Err(init.err("invalid character"))
}

fn nil_or_bool_or_char(i: Input) -> Result<Value> {
    let init = i.clone();
    let i = needs_char(i, '#')?;

    if let Some(i) = istarts_with(i.clone(), "nil") {
        i.set_needs_ws().ok(Value::Nil)
    } else if let Some(i) = istarts_with_ci(i.clone(), "t") {
        i.set_needs_ws().ok(true.into())
    } else if let Some(i) = istarts_with_ci(i.clone(), "f") {
        i.set_needs_ws().ok(false.into())
    } else if let Some(i) = istarts_with(i.clone(), "\\") {
        parse_char(init, i)
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
            '#' => return nil_or_bool_or_char(i),
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
    let res = i.is_empty();
    i.ok(res)
}

pub fn parse(mut i: Input<'_>) -> std::result::Result<Vec<Value>, Error> {
    let mut prog = Vec::new();

    while {
        let eoc;
        (i, eoc) = is_eoc(i)?;
        !eoc
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
            Value::Symbol(Symbol::Name("-0000000test".into()))
        );
    }

    #[test]
    fn parse_list() {
        let expected: Value = vector![1.into(), vector![2.into(), 3.into()].into()].into();

        assert_fp_eq!(list(Input::new(None, "(1(2 3))")), expected.clone());
        assert_fp_eq!(list(Input::new(None, "(1    (2    3)  )")), expected);
    }

    #[test]
    fn parse_string() {
        assert_fp_eq!(string(Input::new(None, "\"ciao\"")), "ciao".into());
        assert_fp_eq!(
            string(Input::new(None, "\"\\\"ciao\\\"\"")),
            "\"ciao\"".into()
        );
        assert_fp_eq!(string(Input::new(None, "\"\\xff\"")), "ÿ".into());
        assert_fp_eq!(string(Input::new(None, "\"\\u000a\"")), "\n".into());
        assert_fp_eq!(string(Input::new(None, "\"\\u{a}\"")), "\n".into());
        assert_fp_eq!(
            string(Input::new(None, "\"\\u{00000000000000000000a}\"")),
            "\n".into()
        );
    }

    #[test]
    fn char() {
        assert_fp_eq!(nil_or_bool_or_char(Input::new(None, "#\\x")), 'x'.into());
        assert_fp_eq!(
            nil_or_bool_or_char(Input::new(None, "#\\x0000000000000000a")),
            '\n'.into()
        );
        assert_fp_eq!(
            nil_or_bool_or_char(Input::new(None, "#\\nEwLiNe")),
            '\n'.into()
        );
        assert_fp_eq!(nil_or_bool_or_char(Input::new(None, "#\\12")), '\n'.into());
        assert_fp_eq!(nil_or_bool_or_char(Input::new(None, "#\\n")), 'n'.into());
        assert_fp_eq!(nil_or_bool_or_char(Input::new(None, "#\\ ")), ' '.into());
    }
}
