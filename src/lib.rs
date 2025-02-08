#[derive(Debug, PartialEq)]
pub enum ParseError {}
pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
}

type ElementParseOption<'a, T> = Option<(T, &'a str)>;

fn parse_null(src: &str) -> ElementParseOption<()> {
    if src.starts_with("null") {
        Some(((), src.split_at(4).1))
    } else {
        None
    }
}

fn parse_bool(src: &str) -> ElementParseOption<bool> {
    match src {
        _t if src.starts_with("true") => Some((true, src.split_at(4).1)),
        _f if src.starts_with("false") => Some((false, src.split_at(5).1)),
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
        .map(|n| (n, &src[pos..]))
}

fn parse_string(src: &str) -> Option<(String, &str)> {
    if !src.starts_with('"') {
        return None;
    }

    let mut pos = 1;
    let mut buffer = String::new();
    let bytes = src.as_bytes();

    while pos < bytes.len() {
        match bytes[pos] {
            b'"' => return Some((buffer, &src[pos + 1..])),
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
pub fn parse(src: &str) -> ParseResult<(Option<Value>, &str)> {
    let src = src.trim();

    if src.is_empty() {
        return Ok((None, src));
    }

    if let Some(((), remaining)) = parse_null(src) {
        return Ok((Some(Value::Null), remaining));
    }

    if let Some((value, remaining)) = parse_bool(src) {
        return Ok((Some(Value::Bool(value)), remaining));
    }

    if let Some((value, remaining)) = parse_number(src) {
        return Ok((Some(Value::Number(value)), remaining));
    }
    if let Some((value, remaining)) = parse_string(src) {
        return Ok((Some(Value::String(value)), remaining));
    }

    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_nothing() {
        assert_eq!(parse(""), Ok((None, "")))
    }

    #[test]
    fn parse_null() {
        assert_eq!(parse("  null asd"), Ok((Some(Value::Null), " asd")))
    }

    #[test]
    fn parse_bool() {
        assert_eq!(parse("false asd"), Ok((Some(Value::Bool(false)), " asd")));
        assert_eq!(parse("true das"), Ok((Some(Value::Bool(true)), " das")));
    }

    #[test]
    fn parse_numbers() {
        assert_eq!(parse("123"), Ok((Some(Value::Number(123.0)), "")));
        assert_eq!(parse("-123"), Ok((Some(Value::Number(-123.0)), "")));
        assert_eq!(parse("0.123"), Ok((Some(Value::Number(0.123)), "")));
        assert_eq!(parse("-0.123"), Ok((Some(Value::Number(-0.123)), "")));
        assert_eq!(parse("1e1"), Ok((Some(Value::Number(10.0)), "")));
        assert_eq!(parse("1e-1"), Ok((Some(Value::Number(0.1)), "")));
        assert_eq!(parse("-1e-1"), Ok((Some(Value::Number(-0.1)), "")));
        assert_eq!(parse("1.1e1"), Ok((Some(Value::Number(11.0)), "")));
        assert_eq!(parse("-1.1e1"), Ok((Some(Value::Number(-11.0)), "")));
    }

    #[test]
    fn parse_string() {
        assert_eq!(
            parse("\"asd\""),
            Ok((Some(Value::String("asd".to_string())), ""))
        );
    }
}
