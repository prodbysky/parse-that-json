use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Value<'a> {
    Null,
    Bool(bool),
    Number(f64),
    String(&'a str),
    Array(Vec<Value<'a>>),
    Object(HashMap<&'a str, Value<'a>>),
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

    if let Some((value, remaining)) = parse_array(src) {
        return Some((Some(Value::Array(value)), remaining));
    }
    if let Some((value, remaining)) = parse_object(src) {
        return Some((Some(Value::Object(value)), remaining));
    }

    None
}

impl std::fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::String(str) => write!(f, "{str}"),
            Self::Number(num) => write!(f, "{num}"),
            Self::Array(arr) => {
                writeln!(f, "[")?;
                for e in arr {
                    writeln!(f, "  {e}")?;
                }
                writeln!(f, "]")
            }
            Self::Object(values) => {
                writeln!(f, "{{")?;
                for (k, v) in values.iter() {
                    writeln!(f, "{k}: {v}")?;
                }

                writeln!(f, "}}")
            }
        }
    }
}

type ElementParseOption<'a, T> = Option<(T, Option<&'a str>)>;

fn parse_array(src: &str) -> ElementParseOption<Vec<Value<'_>>> {
    let mut remaining = src.trim_start();

    if !remaining.starts_with('[') {
        return None;
    }

    remaining = remaining[1..].trim_start();

    let mut elements = Vec::new();

    loop {
        if remaining.starts_with(']') {
            remaining = remaining[1..].trim_start();
            return Some((
                elements,
                if remaining.is_empty() {
                    None
                } else {
                    Some(remaining)
                },
            ));
        }

        let (element, next_remaining) = match parse(remaining) {
            Some((Some(e), r)) => (e, r),
            _ => return None,
        };

        elements.push(element);

        remaining = match next_remaining {
            Some(r) => r.trim_start(),
            None => "",
        };

        if remaining.starts_with(',') {
            remaining = remaining[1..].trim_start();
        } else if remaining.starts_with(']') {
            continue;
        } else {
            return None;
        }
    }
}

fn parse_object(src: &str) -> ElementParseOption<HashMap<&'_ str, Value<'_>>> {
    let mut remaining = src.trim_start();

    if !remaining.starts_with('{') {
        return None;
    }

    remaining = remaining[1..].trim_start();

    let mut map = HashMap::new();

    loop {
        if remaining.starts_with('}') {
            remaining = remaining[1..].trim_start();
            return Some((
                map,
                if remaining.is_empty() {
                    None
                } else {
                    Some(remaining)
                },
            ));
        }

        let (key, next_remaining) = match parse_string(remaining) {
            Some((k, next)) => (k, next),
            _ => return None,
        };

        remaining = match next_remaining {
            Some(r) => r.trim_start(),
            None => return None,
        };

        if !remaining.starts_with(':') {
            return None;
        }

        remaining = remaining[1..].trim_start();

        let (value, next_remaining_value) = match parse(remaining) {
            Some((Some(v), next)) => (v, next),
            _ => return None,
        };

        map.insert(key, value);

        remaining = match next_remaining_value {
            Some(r) => r.trim_start(),
            None => "",
        };

        if remaining.starts_with(',') {
            remaining = remaining[1..].trim_start();
        } else if remaining.starts_with('}') {
            continue;
        } else {
            return None;
        }
    }
}

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

fn parse_string<'a>(src: &'a str) -> ElementParseOption<'a, &'a str> {
    if !src.starts_with('"') {
        return None;
    }

    let bytes = src.as_bytes();
    let mut pos = 1;
    let mut escaped = false;

    while pos < bytes.len() {
        if escaped {
            match bytes[pos] {
                b'u' => {
                    if pos + 4 >= bytes.len() {
                        return None;
                    }
                    pos += 4;
                }
                _ => {
                    pos += 1;
                }
            }
            escaped = false;
        } else {
            match bytes[pos] {
                b'\\' => {
                    escaped = true;
                    pos += 1;
                }
                b'"' => {
                    let string_slice = &src[1..pos];
                    let remaining = if pos + 1 > src.len() {
                        None
                    } else {
                        match &src[pos + 1..] {
                            x if x.is_empty() => None,
                            x => Some(x),
                        }
                    };
                    return Some((string_slice, remaining));
                }
                c if c < 0x20 => return None,
                _ => pos += 1,
            }
        }
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
        assert_eq!(parse("\"asd\""), Some((Some(Value::String("asd")), None)));
    }
}
