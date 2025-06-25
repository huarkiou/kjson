use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

#[derive(Debug, PartialEq)]
pub enum Number {
    Int(i64),
    Float(f64),
}

#[derive(PartialEq, Debug)]
pub enum ParseError {
    ExpectValue,
    InvalidValue,
    RootNotSingular,
}

impl Value {
    pub fn parse(json: &str) -> Result<Value, ParseError> {
        let mut c: Context = Context { json };
        Value::parse_whitespace(&mut c).unwrap();
        match Value::parse_value(&mut c) {
            Ok(v) => {
                Value::parse_whitespace(&mut c).unwrap();
                if c.json.is_empty() {
                    Ok(v)
                } else {
                    Err(ParseError::RootNotSingular)
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn parse_inplace(&mut self, json: &str) -> Result<(), ParseError> {
        *self = Value::parse(json)?;
        Ok(())
    }

    fn parse_whitespace(context: &mut Context) -> Result<(), ParseError> {
        let bytes = context.json.as_bytes();
        for (i, &c) in bytes.iter().enumerate() {
            if !(c == b' ' || c == b'\t' || c == b'\n' || c == b'\r') {
                context.json = std::str::from_utf8(&bytes[i..]).unwrap();
                return Ok(());
            }
        }
        context.json = &context.json[context.json.len()..];
        Ok(())
    }

    fn parse_literal(context: &mut Context) -> Result<Value, ParseError> {
        let bytes = context.json.as_bytes();
        match *bytes.first().unwrap() {
            b'n' => {
                const NULL_LITERAL: &str = "null";
                if bytes.len() >= NULL_LITERAL.len() && std::str::from_utf8(&bytes[0..NULL_LITERAL.len()]).unwrap() == NULL_LITERAL {
                    context.json = std::str::from_utf8(&bytes[NULL_LITERAL.len()..]).unwrap();
                    Ok(Value::Null)
                } else {
                    Err(ParseError::InvalidValue)
                }
            }
            b't' => {
                const TRUE_LITERAL: &str = "true";
                if bytes.len() >= TRUE_LITERAL.len() && std::str::from_utf8(&bytes[0..TRUE_LITERAL.len()]).unwrap() == TRUE_LITERAL {
                    context.json = std::str::from_utf8(&bytes[TRUE_LITERAL.len()..]).unwrap();
                    Ok(Value::Bool(true))
                } else {
                    Err(ParseError::InvalidValue)
                }
            }
            b'f' => {
                const FALSE_LITERAL: &str = "false";
                if bytes.len() >= FALSE_LITERAL.len() && std::str::from_utf8(&bytes[0..FALSE_LITERAL.len()]).unwrap() == FALSE_LITERAL {
                    context.json = std::str::from_utf8(&bytes[FALSE_LITERAL.len()..]).unwrap();
                    Ok(Value::Bool(false))
                } else {
                    Err(ParseError::InvalidValue)
                }
            }
            _ => Err(ParseError::InvalidValue),
        }
    }

    fn skip_following_digits(bytes: &[u8], start: usize) -> usize {
        if start >= bytes.len() {
            return 0 as usize;
        }
        let mut count: usize = 0;
        for &b in bytes[start..].iter() {
            if !(b.is_ascii_digit()) {
                break;
            }
            count += 1;
        }
        return count;
    }

    fn parse_number(context: &mut Context) -> Result<Value, ParseError> {
        let bytes = context.json.as_bytes();
        // assert!(bytes.first().unwrap().is_ascii_digit() || *bytes.first().unwrap() == b'-');
        let mut index_end: usize = 0;
        let mut is_float: bool = false;
        // 整数部分
        if *bytes.first().unwrap() == b'-' {
            index_end += 1;
        }
        {
            let len_int = Value::skip_following_digits(bytes, index_end);
            if (bytes[index_end] == b'0' && len_int > 1) || len_int == 0 {
                return Err(ParseError::InvalidValue);
            }
            index_end += len_int;
        }

        // 小数部分
        if index_end < bytes.len() && bytes[index_end] == b'.' {
            index_end += 1;
            is_float = true;
            let len_int = Value::skip_following_digits(bytes, index_end);
            if len_int == 0 {
                return Err(ParseError::InvalidValue);
            }
            index_end += len_int;
        }

        // 指数部分
        if index_end < bytes.len() && (bytes[index_end] == b'e' || bytes[index_end] == b'E') {
            index_end += 1;
            is_float = true;
            // 正负号
            if index_end < bytes.len() && (bytes[index_end] == b'+' || bytes[index_end] == b'-') {
                index_end += 1;
            }
            let len_int = Value::skip_following_digits(bytes, index_end);
            if len_int == 0 {
                return Err(ParseError::InvalidValue);
            }
            index_end += len_int;
        }

        // 转换为二进制返回
        let number_str = std::str::from_utf8(&bytes[0..index_end]).unwrap();
        context.json = std::str::from_utf8(&bytes[index_end..]).unwrap();
        if is_float {
            Ok(Value::Number(Number::Float(number_str.parse::<f64>().unwrap())))
        } else {
            Ok(Value::Number(Number::Int(number_str.parse::<i64>().unwrap())))
        }
    }

    fn parse_value(context: &mut Context) -> Result<Value, ParseError> {
        let bytes = context.json.as_bytes();
        match bytes.first() {
            Some(byte) => match *byte {
                b'n' | b't' | b'f' => Value::parse_literal(context),
                _ => Value::parse_number(context),
            },
            None => Err(ParseError::ExpectValue),
        }
    }
}

struct Context<'a> {
    json: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_null() {
        assert_eq!(Value::parse("null").ok().unwrap(), Value::Null);
        assert_eq!(Value::parse(" \t\r\n\nnull").ok().unwrap(), Value::Null);
        assert_eq!(Value::parse("null \t\r\n\n").ok().unwrap(), Value::Null);
        assert_eq!(Value::parse(" \t\r\n\nnull \t\r\n\n").ok().unwrap(), Value::Null);
    }

    #[test]
    fn parse_bool() {
        assert_eq!(Value::parse("true").ok().unwrap(), Value::Bool(true));
        assert_eq!(Value::parse(" \t\r\n\ntrue").ok().unwrap(), Value::Bool(true));
        assert_eq!(Value::parse("true \t\r\n\n").ok().unwrap(), Value::Bool(true));
        assert_eq!(Value::parse(" \t\r\n\ntrue \t\r\n\n").ok().unwrap(), Value::Bool(true));

        assert_eq!(Value::parse("false").ok().unwrap(), Value::Bool(false));
        assert_eq!(Value::parse(" \t\r\n\nfalse").ok().unwrap(), Value::Bool(false));
        assert_eq!(Value::parse("false \t\r\n\n").ok().unwrap(), Value::Bool(false));
        assert_eq!(Value::parse(" \t\r\n\nfalse \t\r\n\n").ok().unwrap(), Value::Bool(false));
    }

    #[test]
    fn parse_number() {
        // ok
        assert_eq!(Value::parse("0").ok().unwrap(), Value::Number(Number::Int(0)));
        assert_eq!(Value::parse("-0").ok().unwrap(), Value::Number(Number::Int(0)));
        assert_eq!(Value::parse("1").ok().unwrap(), Value::Number(Number::Int(1)));
        assert_eq!(Value::parse("-1").ok().unwrap(), Value::Number(Number::Int(-1)));
        assert_eq!(Value::parse("-0.0").ok().unwrap(), Value::Number(Number::Float(0.0)));
        assert_eq!(Value::parse("-1.5").ok().unwrap(), Value::Number(Number::Float(-1.5)));
        assert_eq!(Value::parse("1.5").ok().unwrap(), Value::Number(Number::Float(1.5)));
        assert_eq!(Value::parse("3.1415926").ok().unwrap(), Value::Number(Number::Float(3.1415926)));
        assert_eq!(Value::parse("1E10").ok().unwrap(), Value::Number(Number::Float(1E10)));
        assert_eq!(Value::parse("1e10").ok().unwrap(), Value::Number(Number::Float(1e10)));
        assert_eq!(Value::parse("1E+10").ok().unwrap(), Value::Number(Number::Float(1E+10)));
        assert_eq!(Value::parse("1E-10").ok().unwrap(), Value::Number(Number::Float(1E-10)));
        assert_eq!(Value::parse("-1E10").ok().unwrap(), Value::Number(Number::Float(-1E10)));
        assert_eq!(Value::parse("-1e10").ok().unwrap(), Value::Number(Number::Float(-1e10)));
        assert_eq!(Value::parse("-1E+10").ok().unwrap(), Value::Number(Number::Float(-1E+10)));
        assert_eq!(Value::parse("-1E-10").ok().unwrap(), Value::Number(Number::Float(-1E-10)));
        assert_eq!(Value::parse("1.234E+10").ok().unwrap(), Value::Number(Number::Float(1.234E+10)));
        assert_eq!(Value::parse("1.234E-10").ok().unwrap(), Value::Number(Number::Float(1.234E-10)));
        assert_eq!(Value::parse("1e-10000").ok().unwrap(), Value::Number(Number::Float(0.0)));

        // error
        assert_eq!(Value::parse("+0").err().unwrap(), ParseError::InvalidValue);
        assert_eq!(Value::parse("+1").err().unwrap(), ParseError::InvalidValue);
        assert_eq!(Value::parse(".123").err().unwrap(), ParseError::InvalidValue);
        assert_eq!(Value::parse("1.").err().unwrap(), ParseError::InvalidValue);
        assert_eq!(Value::parse("INF").err().unwrap(), ParseError::InvalidValue);
        assert_eq!(Value::parse("inf").err().unwrap(), ParseError::InvalidValue);
        assert_eq!(Value::parse("NAN").err().unwrap(), ParseError::InvalidValue);
        assert_eq!(Value::parse("NaN").err().unwrap(), ParseError::InvalidValue);
        assert_eq!(Value::parse("nan").err().unwrap(), ParseError::InvalidValue);
    }

    #[test]
    fn parse_expect_value() {
        assert_eq!(Value::parse("").err().unwrap(), ParseError::ExpectValue);
        assert_eq!(Value::parse(" \t\r\n\n").err().unwrap(), ParseError::ExpectValue);
    }

    #[test]
    fn parse_root_not_singular() {
        assert_eq!(Value::parse(" \t\r\nnull\ntrue").err().unwrap(), ParseError::RootNotSingular);
        assert_eq!(Value::parse(" \t\r\nnull\n\r\t ").ok().unwrap(), Value::Null);
        assert_eq!(Value::parse("null\n\r \ttrue\r \t\r").err().unwrap(), ParseError::RootNotSingular);
    }
}
