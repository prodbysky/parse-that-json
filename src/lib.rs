#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
}

pub fn parse(src: &str) -> ElementParseOption<Option<Value>> {
    let src = src.trim();

    if src.is_empty() {
        return Some((None, None));
    }

    if let Some(((), remaining)) = parse_null(src) {
        return Some((Some(Value::Null), remaining));
    }

    if let Some((value, remaining)) = parse_bool(src) {
        return Some((Some(Value::Bool(value)), remaining));
    }

    if let Some((value, remaining)) = parse_number(src) {
        return Some((Some(Value::Number(value)), remaining));
    }
    if let Some((value, remaining)) = parse_string(src) {
        return Some((Some(Value::String(value)), remaining));
    }

    unreachable!()
}

type ElementParseOption<'a, T> = Option<(T, Option<&'a str>)>;

fn parse_null(src: &str) -> ElementParseOption<()> {
    if src.starts_with("null") {
        Some((
            (),
            match src.split_at(4).1 {
                x if x.is_empty() => None,
                x => Some(x),
            },
        ))
    } else {
        None
    }
}

fn parse_bool(src: &str) -> ElementParseOption<bool> {
    match src {
        _t if src.starts_with("true") => Some((
            true,
            match src.split_at(4).1 {
                x if x.is_empty() => None,
                x => Some(x),
            },
        )),
        _f if src.starts_with("false") => Some((
            false,
            match src.split_at(5).1 {
                x if x.is_empty() => None,
                x => Some(x),
            },
        )),
        _ => None,
    }
}

fn parse_number(src: &str) -> ElementParseOption<f64> {
    let bytes = src.as_bytes();
    let mut pos = 0;
    let _len = bytes.len();

    if bytes.get(pos) == Some(&b'-') {
        pos += 1;
    }

    match bytes.get(pos) {
        Some(b'0') => {
            pos += 1;
            if bytes.get(pos).map_or(false, |c| c.is_ascii_digit()) {
                return None;
            }
        }
        Some(c) if c.is_ascii_digit() => {
            pos += 1;
            while bytes.get(pos).map_or(false, |c| c.is_ascii_digit()) {
                pos += 1;
            }
        }
        _ => return None,
    }

    if bytes.get(pos) == Some(&b'.') {
        pos += 1;
        let digits_start = pos;
        while bytes.get(pos).map_or(false, |c| c.is_ascii_digit()) {
            pos += 1;
        }
        if pos == digits_start {
            return None;
        }
    }

    if bytes
        .get(pos)
        .filter(|c| **c == b'e' || **c == b'E')
        .is_some()
    {
        pos += 1;
        if bytes
            .get(pos)
            .filter(|c| **c == b'+' || **c == b'-')
            .is_some()
        {
            pos += 1;
        }
        let digits_start = pos;
        while bytes.get(pos).map_or(false, |c| c.is_ascii_digit()) {
            pos += 1;
        }
        if pos == digits_start {
            return None;
        }
    }

    (!src.is_empty() && pos > 0)
        .then(|| src[..pos].parse().ok())
        .flatten()
        .map(|n| {
            (
                n,
                match &src[pos..] {
                    x if x.is_empty() => None,
                    x => Some(x),
                },
            )
        })
}

fn parse_string(src: &str) -> ElementParseOption<String> {
    if !src.starts_with('"') {
        return None;
    }

    let mut pos = 1;
    let mut buffer = String::new();
    let bytes = src.as_bytes();

    while pos < bytes.len() {
        match bytes[pos] {
            b'"' => {
                return Some((
                    buffer,
                    match &src[pos + 1..] {
                        x if x.is_empty() => None,
                        x => Some(x),
                    },
                ))
            }
            b'\\' => {
                pos += 1;
                match bytes.get(pos)? {
                    b'"' => buffer.push('"'),
                    b'\\' => buffer.push('\\'),
                    b'/' => buffer.push('/'),
                    b'b' => buffer.push('\u{0008}'),
                    b'f' => buffer.push('\u{000C}'),
                    b'n' => buffer.push('\n'),
                    b'r' => buffer.push('\r'),
                    b't' => buffer.push('\t'),
                    b'u' => {
                        pos += 1;
                        let hex = src.get(pos..pos + 4)?;
                        let code = u32::from_str_radix(hex, 16).ok()?;
                        buffer.push(std::char::from_u32(code)?);
                        pos += 3;
                    }
                    _ => return None,
                }
            }
            c if c < 0x20 => return None,
            c => buffer.push(c as char),
        }
        pos += 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_nothing() {
        assert_eq!(parse(""), Some((None, None)))
    }

    #[test]
    fn parse_null() {
        assert_eq!(parse("  null asd"), Some((Some(Value::Null), Some(" asd"))))
    }

    #[test]
    fn parse_bool() {
        assert_eq!(
            parse("false asd"),
            Some((Some(Value::Bool(false)), Some(" asd")))
        );
        assert_eq!(
            parse("true das"),
            Some((Some(Value::Bool(true)), Some(" das")))
        );
    }

    #[test]
    fn parse_numbers() {
        assert_eq!(parse("123"), Some((Some(Value::Number(123.0)), None)));
        assert_eq!(parse("-123"), Some((Some(Value::Number(-123.0)), None)));
        assert_eq!(parse("0.123"), Some((Some(Value::Number(0.123)), None)));
        assert_eq!(parse("-0.123"), Some((Some(Value::Number(-0.123)), None)));
        assert_eq!(parse("1e1"), Some((Some(Value::Number(10.0)), None)));
        assert_eq!(parse("1e-1"), Some((Some(Value::Number(0.1)), None)));
        assert_eq!(parse("-1e-1"), Some((Some(Value::Number(-0.1)), None)));
        assert_eq!(parse("1.1e1"), Some((Some(Value::Number(11.0)), None)));
        assert_eq!(parse("-1.1e1"), Some((Some(Value::Number(-11.0)), None)));
    }

    #[test]
    fn parse_string() {
        assert_eq!(
            parse("\"asd\""),
            Some((Some(Value::String("asd".to_string())), None))
        );
    }
}
