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

    fn parse_null(context: &mut Context) -> Result<Value, ParseError> {
        let bytes = context.json.as_bytes();
        assert_eq!(*bytes.first().unwrap(), b'n');
        if bytes[0] != b'n' || bytes[1] != b'u' || bytes[2] != b'l' || bytes[3] != b'l' {
            return Err(ParseError::InvalidValue);
        }
        context.json = std::str::from_utf8(&bytes[4..]).unwrap();
        Ok(Value::Null)
    }
    fn parse_true(context: &mut Context) -> Result<Value, ParseError> {
        let bytes = context.json.as_bytes();
        assert_eq!(*bytes.first().unwrap(), b't');
        if bytes[0] != b't' || bytes[1] != b'r' || bytes[2] != b'u' || bytes[3] != b'e' {
            return Err(ParseError::InvalidValue);
        }
        context.json = std::str::from_utf8(&bytes[4..]).unwrap();
        Ok(Value::Bool(true))
    }
    fn parse_false(context: &mut Context) -> Result<Value, ParseError> {
        let bytes = context.json.as_bytes();
        assert_eq!(*bytes.first().unwrap(), b'f');
        if bytes[0] != b'f' || bytes[1] != b'a' || bytes[2] != b'l' || bytes[3] != b's' || bytes[4] != b'e' {
            return Err(ParseError::InvalidValue);
        }
        context.json = std::str::from_utf8(&bytes[5..]).unwrap();
        Ok(Value::Bool(false))
    }

    fn parse_value(context: &mut Context) -> Result<Value, ParseError> {
        let bytes = context.json.as_bytes();
        match bytes.first() {
            Some(byte) => match *byte {
                b'n' => Value::parse_null(context),
                b't' => Value::parse_true(context),
                b'f' => Value::parse_false(context),
                _ => Err(ParseError::InvalidValue),
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
