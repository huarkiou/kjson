use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, PartialEq)]
pub enum Number {
    Int(i64),
    Float(f64),
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

#[derive(PartialEq, Debug)]
pub enum ParseError {
    ExpectValue,
    InvalidValue,
    RootNotSingular,
    NumberTooBig,
    MissQuotationMark,
    InvalidStringEscape,
    InvalidUnicodeHex,
    InvalidUnicodeSurrogate,
}

struct Context<'a> {
    bytes: &'a [u8],
    stack: VecDeque<u8>,
}

impl<'a> Context<'a> {
    fn new(json: &'a str) -> Self {
        Self {
            bytes: json.as_bytes(),
            stack: VecDeque::<u8>::new(),
        }
    }

    fn len(&self) -> usize {
        self.stack.len()
    }

    fn push_bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.stack.push_back(b);
        }
    }

    fn push_byte(&mut self, b: u8) {
        self.stack.push_back(b);
    }

    fn pop_bytes(&mut self, size: usize) -> Vec<u8> {
        assert!(size <= self.stack.len());
        let mut ret: Vec<u8> = Vec::with_capacity(size);
        for _ in 0..size {
            if let Some(byte) = self.stack.pop_back() {
                ret.push(byte);
            }
        }
        ret.reverse();
        ret
    }

    fn pop_byte(&mut self) -> u8 {
        assert!(self.stack.len() != 0);
        self.stack.pop_back().unwrap()
    }
}

impl Value {
    pub fn parse(json: &str) -> Result<Value, ParseError> {
        let mut c: Context = Context::new(json);
        Value::parse_whitespace(&mut c).unwrap();
        match Value::parse_value(&mut c) {
            Ok(v) => {
                Value::parse_whitespace(&mut c).unwrap();
                if c.bytes.is_empty() {
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
        let bytes = context.bytes;
        for (i, &c) in bytes.iter().enumerate() {
            if !(c == b' ' || c == b'\t' || c == b'\n' || c == b'\r') {
                context.bytes = &bytes[i..];
                return Ok(());
            }
        }
        context.bytes = &context.bytes[context.bytes.len()..];
        Ok(())
    }

    fn parse_value(context: &mut Context) -> Result<Value, ParseError> {
        let bytes = context.bytes;
        match bytes.first() {
            Some(byte) => match *byte {
                b'n' | b't' | b'f' => Value::parse_literal(context),
                b'"' => Value::parse_string(context),
                _ => Value::parse_number(context),
            },
            None => Err(ParseError::ExpectValue),
        }
    }

    fn parse_literal(context: &mut Context) -> Result<Value, ParseError> {
        let bytes = context.bytes;
        match *bytes.first().unwrap() {
            b'n' => {
                const NULL_LITERAL: &str = "null";
                if bytes.len() >= NULL_LITERAL.len()
                    && std::str::from_utf8(&bytes[0..NULL_LITERAL.len()]).unwrap() == NULL_LITERAL
                {
                    context.bytes = &bytes[NULL_LITERAL.len()..];
                    Ok(Value::Null)
                } else {
                    Err(ParseError::InvalidValue)
                }
            }
            b't' => {
                const TRUE_LITERAL: &str = "true";
                if bytes.len() >= TRUE_LITERAL.len()
                    && std::str::from_utf8(&bytes[0..TRUE_LITERAL.len()]).unwrap() == TRUE_LITERAL
                {
                    context.bytes = &bytes[TRUE_LITERAL.len()..];
                    Ok(Value::Bool(true))
                } else {
                    Err(ParseError::InvalidValue)
                }
            }
            b'f' => {
                const FALSE_LITERAL: &str = "false";
                if bytes.len() >= FALSE_LITERAL.len()
                    && std::str::from_utf8(&bytes[0..FALSE_LITERAL.len()]).unwrap() == FALSE_LITERAL
                {
                    context.bytes = &bytes[FALSE_LITERAL.len()..];
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
        let bytes = context.bytes;
        // assert!(bytes.first().unwrap().is_ascii_digit() || *bytes.first().unwrap() == b'-');
        let mut index_end: usize = 0;
        let mut is_float: bool = false;
        // 整数部分
        if *bytes.first().unwrap() == b'-' {
            index_end += 1;
        }
        {
            let len_int = Value::skip_following_digits(bytes, index_end);
            if len_int == 0 {
                return Err(ParseError::InvalidValue);
            }
            if bytes[index_end] == b'0' && len_int > 1 {
                return Err(ParseError::RootNotSingular);
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
        context.bytes = &bytes[index_end..];
        let number_str = std::str::from_utf8(&bytes[0..index_end]).unwrap();
        if is_float {
            match number_str.parse::<f64>() {
                Ok(num) => {
                    if num.is_finite() {
                        Ok(Value::Number(Number::Float(num)))
                    } else {
                        Err(ParseError::NumberTooBig)
                    }
                }
                Err(_) => Err(ParseError::NumberTooBig),
            }
        } else {
            match number_str.parse::<i64>() {
                Ok(num) => Ok(Value::Number(Number::Int(num))),
                Err(_) => Err(ParseError::NumberTooBig),
            }
        }
    }

    fn hex4_to_u32(hex4: &[u8]) -> Option<u32> {
        assert_eq!(hex4.len(), 4);
        let mut value = 0u32;
        for &b in hex4 {
            let digit = match b {
                b'0'..=b'9' => b - b'0',
                b'a'..=b'f' => b - b'a' + 10,
                b'A'..=b'F' => b - b'A' + 10,
                _ => return None, // 非法字符
            };

            value = (value << 4) | u32::from(digit);
        }
        Some(value)
    }

    fn encode_utf8(context: &mut Context, c: u32) -> Option<ParseError> {
        if let Some(ch) = char::from_u32(c) {
            let mut buf = [0; 4]; // UTF-8 最多需要 4 个字节
            let bytes = ch.encode_utf8(&mut buf);
            let utf8_bytes: &[u8] = bytes.as_bytes(); // 获取 &[u8]
            context.push_bytes(utf8_bytes);
            None
        } else {
            Some(ParseError::InvalidUnicodeSurrogate)
        }
    }

    fn parse_string(context: &mut Context) -> Result<Value, ParseError> {
        assert_eq!(*context.bytes.first().unwrap(), b'"');
        let mut quotation_marked: bool = false;
        let mut i = 1;
        let cur_len = context.len();
        while i < context.bytes.len() {
            let b = context.bytes[i];
            match b {
                b'"' => {
                    quotation_marked = true;
                    break;
                }
                b'\\' => {
                    // 处理转义序列
                    if i + 1 < context.bytes.len() {
                        match context.bytes[i + 1] {
                            b'"' => context.push_byte(b'\"'),
                            b'\\' => context.push_byte(b'\\'),
                            b'/' => context.push_byte(b'/'),
                            b'b' => context.push_byte(b'\x62'),
                            b'f' => context.push_byte(b'\x66'),
                            b'n' => context.push_byte(b'\n'),
                            b'r' => context.push_byte(b'\r'),
                            b't' => context.push_byte(b'\t'),
                            b'u' => {
                                if i + 6 >= context.bytes.len() {
                                    return Err(ParseError::InvalidStringEscape);
                                }
                                match Value::hex4_to_u32(&context.bytes[i + 2..i + 6]) {
                                    Some(high_surrogate) => {
                                        if high_surrogate >= 0xD800 && high_surrogate <= 0xDBFF {
                                            // 代码对的高代理项（high surrogate）
                                            if i + 12 < context.bytes.len()
                                                && (context.bytes[i + 6] == b'\\' && context.bytes[i + 7] == b'u')
                                            {
                                                match Value::hex4_to_u32(&context.bytes[i + 8..i + 12]) {
                                                    Some(low_surrogate) => {
                                                        if let Some(e) = Value::encode_utf8(
                                                            context,
                                                            0x10000
                                                                + (high_surrogate - 0xD800) * 0x400
                                                                + (low_surrogate - 0xDC00),
                                                        ) {
                                                            return Err(e);
                                                        }
                                                    }
                                                    None => return Err(ParseError::InvalidUnicodeHex),
                                                }
                                                i += 10;
                                            } else {
                                                return Err(ParseError::InvalidUnicodeSurrogate);
                                            }
                                        } else if let Some(e) = Value::encode_utf8(context, high_surrogate) {
                                            return Err(e);
                                        } else {
                                            i += 4;
                                        }
                                    }
                                    None => return Err(ParseError::InvalidUnicodeHex),
                                }
                            }
                            _ => return Err(ParseError::InvalidStringEscape),
                        }
                        i += 2;
                    }
                }
                _ => {
                    if b < 0x20 {
                        return Err(ParseError::InvalidStringEscape);
                    }
                    context.push_byte(b);
                    i += 1;
                }
            }
        }
        if quotation_marked {
            context.bytes = &context.bytes[i + 1..];
            Ok(Value::String(
                String::from_utf8(context.pop_bytes(context.stack.len() - cur_len)).unwrap(),
            ))
        } else {
            Err(ParseError::MissQuotationMark)
        }
    }
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
        assert_eq!(
            Value::parse(" \t\r\n\nfalse \t\r\n\n").ok().unwrap(),
            Value::Bool(false)
        );
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
        assert_eq!(
            Value::parse("3.1415926").ok().unwrap(),
            Value::Number(Number::Float(3.1415926))
        );
        assert_eq!(Value::parse("1E10").ok().unwrap(), Value::Number(Number::Float(1E10)));
        assert_eq!(Value::parse("1e10").ok().unwrap(), Value::Number(Number::Float(1e10)));
        assert_eq!(Value::parse("1E+10").ok().unwrap(), Value::Number(Number::Float(1E+10)));
        assert_eq!(Value::parse("1E-10").ok().unwrap(), Value::Number(Number::Float(1E-10)));
        assert_eq!(Value::parse("-1E10").ok().unwrap(), Value::Number(Number::Float(-1E10)));
        assert_eq!(Value::parse("-1e10").ok().unwrap(), Value::Number(Number::Float(-1e10)));
        assert_eq!(
            Value::parse("-1E+10").ok().unwrap(),
            Value::Number(Number::Float(-1E+10))
        );
        assert_eq!(
            Value::parse("-1E-10").ok().unwrap(),
            Value::Number(Number::Float(-1E-10))
        );
        assert_eq!(
            Value::parse("1.234E+10").ok().unwrap(),
            Value::Number(Number::Float(1.234E+10))
        );
        assert_eq!(
            Value::parse("1.234E-10").ok().unwrap(),
            Value::Number(Number::Float(1.234E-10))
        );
        assert_eq!(
            Value::parse("1e-10000").ok().unwrap(),
            Value::Number(Number::Float(0.0))
        );
        assert_eq!(
            Value::parse("0.01171875").ok().unwrap(),
            Value::Number(Number::Float(0.01171875))
        );
        assert_eq!(
            Value::parse("2e-1074").ok().unwrap(),
            Value::Number(Number::Float(2.0e-1074))
        );
        assert_eq!(
            Value::parse("2e-1022").ok().unwrap(),
            Value::Number(Number::Float(2.0e-1022))
        );
        assert_eq!(
            Value::parse("1.0000000000000002").ok().unwrap(),
            Value::Number(Number::Float(1.0000000000000002))
        ); /* the smallest number > 1 */
        assert_eq!(
            Value::parse("4.9406564584124654e-324").ok().unwrap(),
            Value::Number(Number::Float(4.9406564584124654e-324))
        ); /* minimum denormal */
        assert_eq!(
            Value::parse("-4.9406564584124654e-324").ok().unwrap(),
            Value::Number(Number::Float(-4.9406564584124654e-324))
        );
        assert_eq!(
            Value::parse("2.2250738585072009e-308").ok().unwrap(),
            Value::Number(Number::Float(2.2250738585072009e-308))
        ); /* Max subnormal double */
        assert_eq!(
            Value::parse("-2.2250738585072009e-308").ok().unwrap(),
            Value::Number(Number::Float(-2.2250738585072009e-308))
        );
        assert_eq!(
            Value::parse("2.2250738585072014e-308").ok().unwrap(),
            Value::Number(Number::Float(2.2250738585072014e-308))
        ); /* Min normal positive double */
        assert_eq!(
            Value::parse("-2.2250738585072014e-308").ok().unwrap(),
            Value::Number(Number::Float(-2.2250738585072014e-308))
        );
        assert_eq!(
            Value::parse("1.7976931348623157e+308").ok().unwrap(),
            Value::Number(Number::Float(1.7976931348623157e+308))
        ); /* Max double */
        assert_eq!(
            Value::parse("-1.7976931348623157e+308").ok().unwrap(),
            Value::Number(Number::Float(-1.7976931348623157e+308))
        );

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
    fn parse_string() {
        assert_eq!(Value::parse(r#""""#).ok().unwrap(), Value::String("".to_string()));
        assert_eq!(
            Value::parse(r#""Hello""#).ok().unwrap(),
            Value::String("Hello".to_string())
        );
        assert_eq!(
            Value::parse(r#""Hello\nWorld""#).ok().unwrap(),
            Value::String("Hello\nWorld".to_string())
        );
        assert_eq!(
            Value::parse(r#""\" \\ / \b \f \n \r \t""#).ok().unwrap(),
            Value::String("\" \\ / \x62 \x66 \n \r \t".to_string())
        );
        assert_eq!(
            Value::parse(r#""\u0024""#).ok().unwrap(),
            Value::String("$".to_string())
        );
        assert_eq!(
            Value::parse(r#""\u00A2""#).ok().unwrap(),
            Value::String("¢".to_string())
        );
        assert_eq!(
            Value::parse(r#""\u20AC""#).ok().unwrap(),
            Value::String("€".to_string())
        );
        assert_eq!(
            Value::parse(r#""\uD834\uDD1E""#).ok().unwrap(),
            Value::String("𝄞".to_string())
        );
        assert_eq!(
            Value::parse(r#""\ud834\udd1e""#).ok().unwrap(),
            Value::String("𝄞".to_string())
        );
    }

    #[test]
    fn parse_number_too_big() {
        assert_eq!(Value::parse("1e309").err().unwrap(), ParseError::NumberTooBig);
        assert_eq!(Value::parse("-1e309").err().unwrap(), ParseError::NumberTooBig);
    }

    #[test]
    fn parse_expect_value() {
        assert_eq!(Value::parse("").err().unwrap(), ParseError::ExpectValue);
        assert_eq!(Value::parse(" \t\r\n\n").err().unwrap(), ParseError::ExpectValue);
    }

    #[test]
    fn parse_root_not_singular() {
        assert_eq!(
            Value::parse(" \t\r\nnull\ntrue").err().unwrap(),
            ParseError::RootNotSingular
        );
        assert_eq!(Value::parse(" \t\r\nnull\n\r\t ").ok().unwrap(), Value::Null);
        assert_eq!(
            Value::parse("null\n\r \ttrue\r \t\r").err().unwrap(),
            ParseError::RootNotSingular
        );
        assert_eq!(Value::parse("0123").err().unwrap(), ParseError::RootNotSingular);
        assert_eq!(Value::parse("0x0").err().unwrap(), ParseError::RootNotSingular);
        assert_eq!(Value::parse("0x123").err().unwrap(), ParseError::RootNotSingular);
    }
}
