use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, PartialEq)]
pub enum Number {
    Int(i64),
    Float(f64),
}

impl ToString for Number {
    fn to_string(&self) -> String {
        match self {
            Number::Int(n) => n.to_string(),
            Number::Float(n) => n.to_string(),
        }
    }
}

#[derive(Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

impl ToString for Value {
    fn to_string(&self) -> String {
        Value::stringify_value(self)
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Array(l0), Self::Array(r0)) => {
                let mut result = true;
                if l0.len() != r0.len() {
                    result = false;
                } else {
                    for i in 0..l0.len() {
                        result = result && (l0[i] == r0[i]);
                    }
                }
                result
            }
            (Self::Object(l0), Self::Object(r0)) => {
                let mut result = true;
                if l0.len() != r0.len() {
                    result = false;
                } else {
                    for key in l0.keys() {
                        result = result && (l0[key] == r0[key]);
                    }
                }
                result
            }
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum ParseError {
    ExpectValue,
    InvalidValue,
    RootNotSingular,
    NumberTooBig,
    MissQuotationMark,
    InvalidStringEscape,
    InvalidStringChar,
    InvalidUnicodeHex,
    InvalidUnicodeSurrogate,
    MissCommaOrSquareBracket,
    MissKey,
    MissColon,
    MissCommaOrCurlyBracket,
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

    fn step(&mut self) -> Option<u8> {
        let &b = self.bytes.first()?;
        self.bytes = &self.bytes[1..];
        Some(b)
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
        let len = self.stack.len();
        if size <= len {
            let removed: Vec<u8> = self.stack.drain(len - size..).collect();
            removed
        } else {
            panic!("Not enough elements in VecDeque");
        }
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
                b'[' => Value::parse_array(context),
                b'{' => Value::parse_object(context),
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
            return 0_usize;
        }
        let mut count: usize = 0;
        for &b in bytes[start..].iter() {
            if !(b.is_ascii_digit()) {
                break;
            }
            count += 1;
        }
        count
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
        if !is_float {
            match number_str.parse::<i64>() {
                Ok(num) => return Ok(Value::Number(Number::Int(num))),
                Err(_) => (),
            }
        }
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

    fn parse_string_raw(context: &mut Context) -> Result<String, ParseError> {
        if context.bytes.len() < 2 || *context.bytes.first().unwrap() != b'"' {
            return Err(ParseError::MissQuotationMark);
        }
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
                                    return Err(ParseError::InvalidUnicodeHex);
                                }
                                match Value::hex4_to_u32(&context.bytes[i + 2..i + 6]) {
                                    Some(high_surrogate) => {
                                        if (0xD800..=0xDBFF).contains(&high_surrogate) {
                                            // 代码对的高代理项（high surrogate）
                                            if i + 12 < context.bytes.len()
                                                && (context.bytes[i + 6] == b'\\' && context.bytes[i + 7] == b'u')
                                            {
                                                match Value::hex4_to_u32(&context.bytes[i + 8..i + 12]) {
                                                    Some(low_surrogate) => {
                                                        if !(0xDC00..=0xDFFF).contains(&low_surrogate) {
                                                            return Err(ParseError::InvalidUnicodeSurrogate);
                                                        }
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
                        return Err(ParseError::InvalidStringChar);
                    }
                    context.push_byte(b);
                    i += 1;
                }
            }
        }
        if quotation_marked {
            context.bytes = &context.bytes[i + 1..];
            Ok(String::from_utf8(context.pop_bytes(context.stack.len() - cur_len)).unwrap())
        } else {
            Err(ParseError::MissQuotationMark)
        }
    }

    fn parse_string(context: &mut Context) -> Result<Value, ParseError> {
        assert_eq!(*context.bytes.first().unwrap(), b'"');
        Value::parse_string_raw(context).map(|s| Value::String(s))
    }

    fn parse_array(context: &mut Context) -> Result<Value, ParseError> {
        assert_eq!(context.step().unwrap(), b'[');
        Value::parse_whitespace(context).unwrap();

        let mut arr: Vec<Value> = Vec::new();

        if *context.bytes.first().unwrap() == b']' {
            context.step();
            return Ok(Value::Array(arr));
        }

        loop {
            match Value::parse_value(context) {
                Ok(v) => arr.push(v),
                Err(e) => return Err(e),
            }
            Value::parse_whitespace(context).unwrap();
            let next_byte = context.step();
            match next_byte {
                Some(b) => match b {
                    b',' => Value::parse_whitespace(context).unwrap(),
                    b']' => return Ok(Value::Array(arr)),
                    _ => return Err(ParseError::MissCommaOrSquareBracket),
                },
                None => return Err(ParseError::MissCommaOrSquareBracket),
            }
        }
    }

    fn parse_object(context: &mut Context) -> Result<Value, ParseError> {
        assert_eq!(context.step().unwrap(), b'{');
        Value::parse_whitespace(context).unwrap();

        let mut object: BTreeMap<String, Value> = BTreeMap::new();
        if *context.bytes.first().unwrap() == b'}' {
            context.step();
            return Ok(Value::Object(object));
        }
        loop {
            // parse key
            if let Ok(str) = Value::parse_string_raw(context) {
                let key = str;
                // parse colon(:)
                Value::parse_whitespace(context).unwrap();
                if let Some(b':') = context.step() {
                } else {
                    return Err(ParseError::MissColon);
                }
                // parse value
                Value::parse_whitespace(context).unwrap();
                match Value::parse_value(context) {
                    Ok(v) => {
                        object.insert(key, v);
                    }
                    Err(_) => return Err(ParseError::InvalidValue),
                }
                // parse ws [comma | right-curly-brace] ws }
                Value::parse_whitespace(context).unwrap();
                match context.step() {
                    Some(b',') => {
                        Value::parse_whitespace(context).unwrap();
                    }
                    Some(b'}') => return Ok(Value::Object(object)),
                    _ => return Err(ParseError::MissCommaOrCurlyBracket),
                }
            } else {
                return Err(ParseError::MissKey);
            }
        }
    }

    fn stringify_value(value: &Value) -> String {
        match value {
            Value::Null => String::from("null"),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => Value::stringify_string(s),
            Value::Array(arr) => Value::stringify_array(arr),
            Value::Object(object) => Value::stringify_object(object),
        }
    }

    fn stringify_string(s: &String) -> String {
        let mut stack = Vec::new();
        stack.push(b'"');
        for &byte in s.as_bytes().iter() {
            match byte {
                b'"' => {
                    stack.push(b'\\');
                    stack.push(b'\"');
                }
                b'\\' => {
                    stack.push(b'\\');
                    stack.push(b'\\');
                }
                b'\x62' => {
                    stack.push(b'\\');
                    stack.push(b'b');
                }
                b'\x66' => {
                    stack.push(b'\\');
                    stack.push(b'f');
                }
                b'\n' => {
                    stack.push(b'\\');
                    stack.push(b'n');
                }
                b'\r' => {
                    stack.push(b'\\');
                    stack.push(b'r');
                }
                b'\t' => {
                    stack.push(b'\\');
                    stack.push(b't');
                }
                _ => {
                    if byte < 0x20 {
                        for &c in format!("\\u{:04X}", byte).as_bytes() {
                            stack.push(c);
                        }
                    } else {
                        stack.push(byte);
                    }
                }
            }
        }
        stack.push(b'"');

        std::str::from_utf8(&stack).unwrap().to_string()
    }

    fn stringify_array(arr: &Vec<Value>) -> String {
        let mut result = String::from("[");
        match arr.len() {
            0 => (),
            1 => result.push_str(&arr.first().unwrap().to_string()),
            _ => {
                result.push_str(&arr.first().unwrap().to_string());
                for v in arr.iter().skip(1) {
                    result.push(',');
                    result.push_str(&v.to_string());
                }
            }
        }
        result.push(']');
        result
    }

    fn stringify_object(object: &BTreeMap<String, Value>) -> String {
        let mut result = String::from("{");
        match object.len() {
            0 => (),
            1 => {
                let (key, value) = object.first_key_value().unwrap();
                result.push('"');
                result.push_str(key);
                result.push('"');
                result.push(':');
                result.push_str(&value.to_string());
            }
            _ => {
                let (key, value) = object.first_key_value().unwrap();
                result.push('"');
                result.push_str(key);
                result.push('"');
                result.push(':');
                result.push_str(&value.to_string());
                for (key, value) in object.iter().skip(1) {
                    result.push(',');
                    result.push('"');
                    result.push_str(key);
                    result.push('"');
                    result.push(':');
                    result.push_str(&value.to_string());
                }
            }
        }
        result.push('}');
        result
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
    fn parse_array() {
        let result = Value::parse(r#"[ ]"#);
        assert!(result.is_ok());
        match result.ok().unwrap() {
            Value::Array(arr) => {
                assert!(true);
                assert_eq!(arr.len(), 0);
            }
            _ => assert!(false),
        };

        assert_eq!(
            Value::parse(r#"[ null , false , true , 123 , "abc" ]"#).ok().unwrap(),
            Value::Array(vec![
                Value::Null,
                Value::Bool(false),
                Value::Bool(true),
                Value::Number(Number::Int(123)),
                Value::String("abc".to_string())
            ])
        );

        assert_eq!(
            Value::parse(r#"[ [ ] , [ 0 ] , [ 0 , 1 ] , [ 0 , 1 , 2 ] ]"#)
                .ok()
                .unwrap(),
            Value::Array(vec![
                Value::Array(Vec::new()),
                Value::Array(vec![Value::Number(Number::Int(0))]),
                Value::Array(vec![Value::Number(Number::Int(0)), Value::Number(Number::Int(1))]),
                Value::Array(vec![
                    Value::Number(Number::Int(0)),
                    Value::Number(Number::Int(1)),
                    Value::Number(Number::Int(2))
                ]),
            ])
        );
    }

    #[test]
    fn parse_object() {
        let mut map = BTreeMap::new();
        map.insert("n".to_string(), Value::Null);
        map.insert("f".to_string(), Value::Bool(false));
        map.insert("t".to_string(), Value::Bool(true));
        map.insert("i".to_string(), Value::Number(Number::Int(123)));
        map.insert("s".to_string(), Value::String("abc".to_string()));
        map.insert(
            "a".to_string(),
            Value::Array(vec![
                Value::Number(Number::Int(1)),
                Value::Number(Number::Int(2)),
                Value::Number(Number::Int(3)),
            ]),
        );
        let mut submap = BTreeMap::new();
        submap.insert("1".to_string(), Value::Number(Number::Int(1)));
        submap.insert("2".to_string(), Value::Number(Number::Int(2)));
        submap.insert("3".to_string(), Value::Number(Number::Int(3)));
        map.insert("o".to_string(), Value::Object(submap));

        let object = Value::Object(map);
        assert_eq!(
            Value::parse(
                r##"
        {
        "n" : null ,
        "f" : false ,
        "t" : true ,
        "i" : 123 , 
        "s" : "abc", 
        "a" : [ 1, 2, 3 ],
        "o" : { "1" : 1, "2" : 2, "3" : 3 }
        }
            "##
            )
            .ok()
            .unwrap(),
            object
        );
    }

    #[test]
    fn parse_expect_value() {
        assert_eq!(Value::parse("").err().unwrap(), ParseError::ExpectValue);
        assert_eq!(Value::parse(" \t\r\n\n").err().unwrap(), ParseError::ExpectValue);
    }

    #[test]
    fn parse_invalid_value() {
        assert_eq!(Value::parse("nul").err().unwrap(), ParseError::InvalidValue);
        assert_eq!(Value::parse("?").err().unwrap(), ParseError::InvalidValue);

        assert_eq!(Value::parse("+0").err().unwrap(), ParseError::InvalidValue);
        assert_eq!(Value::parse("+1").err().unwrap(), ParseError::InvalidValue);
        assert_eq!(Value::parse(".123").err().unwrap(), ParseError::InvalidValue);
        assert_eq!(Value::parse("1.").err().unwrap(), ParseError::InvalidValue);
        assert_eq!(Value::parse("INF").err().unwrap(), ParseError::InvalidValue);
        assert_eq!(Value::parse("inf").err().unwrap(), ParseError::InvalidValue);
        assert_eq!(Value::parse("NAN").err().unwrap(), ParseError::InvalidValue);
        assert_eq!(Value::parse("NaN").err().unwrap(), ParseError::InvalidValue);
        assert_eq!(Value::parse("nan").err().unwrap(), ParseError::InvalidValue);

        assert_eq!(Value::parse("[1,]").err().unwrap(), ParseError::InvalidValue);
        assert_eq!(Value::parse(r#"["a", nul]"#).err().unwrap(), ParseError::InvalidValue);
    }

    #[test]
    fn parse_root_not_singular() {
        assert_eq!(Value::parse("null x").err().unwrap(), ParseError::RootNotSingular);
        assert_eq!(
            Value::parse(" \t\r\nnull\ntrue").err().unwrap(),
            ParseError::RootNotSingular
        );
        assert_eq!(
            Value::parse("null\n\r \ttrue\r \t\r").err().unwrap(),
            ParseError::RootNotSingular
        );

        assert_eq!(Value::parse("0123").err().unwrap(), ParseError::RootNotSingular);
        assert_eq!(Value::parse("0x0").err().unwrap(), ParseError::RootNotSingular);
        assert_eq!(Value::parse("0x123").err().unwrap(), ParseError::RootNotSingular);
    }

    #[test]
    fn parse_number_too_big() {
        assert_eq!(Value::parse("1e309").err().unwrap(), ParseError::NumberTooBig);
        assert_eq!(Value::parse("-1e309").err().unwrap(), ParseError::NumberTooBig);
    }

    #[test]
    fn parse_miss_quotation_mark() {
        assert_eq!(Value::parse(r#"""#).err().unwrap(), ParseError::MissQuotationMark);
        assert_eq!(Value::parse(r#""abc"#).err().unwrap(), ParseError::MissQuotationMark);
    }

    #[test]
    fn parse_invalid_string_escape() {
        assert_eq!(Value::parse(r#""\v""#).err().unwrap(), ParseError::InvalidStringEscape);
        assert_eq!(Value::parse(r#""\'""#).err().unwrap(), ParseError::InvalidStringEscape);
        assert_eq!(Value::parse(r#""\0""#).err().unwrap(), ParseError::InvalidStringEscape);
        assert_eq!(
            Value::parse(r#""\x12""#).err().unwrap(),
            ParseError::InvalidStringEscape
        );
    }

    #[test]
    fn parse_invalid_string_char() {
        assert_eq!(Value::parse("\"\x01\"").err().unwrap(), ParseError::InvalidStringChar);
        assert_eq!(Value::parse("\"\x1F\"").err().unwrap(), ParseError::InvalidStringChar);
    }

    #[test]
    fn parse_invalid_unicode_hex() {
        assert_eq!(Value::parse(r#""\u""#).err().unwrap(), ParseError::InvalidUnicodeHex);
        assert_eq!(Value::parse(r#""\u0""#).err().unwrap(), ParseError::InvalidUnicodeHex);
        assert_eq!(Value::parse(r#""\u01""#).err().unwrap(), ParseError::InvalidUnicodeHex);
        assert_eq!(Value::parse(r#""\u012""#).err().unwrap(), ParseError::InvalidUnicodeHex);
        assert_eq!(
            Value::parse(r#""\u/000""#).err().unwrap(),
            ParseError::InvalidUnicodeHex
        );
        assert_eq!(
            Value::parse(r#""\uG000""#).err().unwrap(),
            ParseError::InvalidUnicodeHex
        );
        assert_eq!(
            Value::parse(r#""\u0/00""#).err().unwrap(),
            ParseError::InvalidUnicodeHex
        );
        assert_eq!(
            Value::parse(r#""\u0G00""#).err().unwrap(),
            ParseError::InvalidUnicodeHex
        );
        assert_eq!(
            Value::parse(r#""\u00/0""#).err().unwrap(),
            ParseError::InvalidUnicodeHex
        );
        assert_eq!(
            Value::parse(r#""\u00G0""#).err().unwrap(),
            ParseError::InvalidUnicodeHex
        );
        assert_eq!(
            Value::parse(r#""\u000/""#).err().unwrap(),
            ParseError::InvalidUnicodeHex
        );
        assert_eq!(
            Value::parse(r#""\u000G""#).err().unwrap(),
            ParseError::InvalidUnicodeHex
        );
        assert_eq!(
            Value::parse(r#""\u 123""#).err().unwrap(),
            ParseError::InvalidUnicodeHex
        );
    }

    #[test]
    fn parse_invalid_unicode_surrogate() {
        assert_eq!(
            Value::parse(r#""\uD800""#).err().unwrap(),
            ParseError::InvalidUnicodeSurrogate
        );
        assert_eq!(
            Value::parse(r#""\uDBFF""#).err().unwrap(),
            ParseError::InvalidUnicodeSurrogate
        );
        assert_eq!(
            Value::parse(r#""\uD800\\""#).err().unwrap(),
            ParseError::InvalidUnicodeSurrogate
        );
        assert_eq!(
            Value::parse(r#""\uD800\uDBFF""#).err().unwrap(),
            ParseError::InvalidUnicodeSurrogate
        );
        assert_eq!(
            Value::parse(r#""\uD800""#).err().unwrap(),
            ParseError::InvalidUnicodeSurrogate
        );
        assert_eq!(
            Value::parse(r#""\uD800\uE000""#).err().unwrap(),
            ParseError::InvalidUnicodeSurrogate
        );
    }

    #[test]
    fn parse_miss_comma_or_square_bracket() {
        assert_eq!(Value::parse("[1").err().unwrap(), ParseError::MissCommaOrSquareBracket);
        assert_eq!(Value::parse("[1}").err().unwrap(), ParseError::MissCommaOrSquareBracket);
        assert_eq!(
            Value::parse("[1 2").err().unwrap(),
            ParseError::MissCommaOrSquareBracket
        );
        assert_eq!(Value::parse("[[]").err().unwrap(), ParseError::MissCommaOrSquareBracket);
    }

    #[test]
    fn parse_miss_key() {
        assert_eq!(Value::parse("{:1,").err().unwrap(), ParseError::MissKey);
        assert_eq!(Value::parse("{1:1,").err().unwrap(), ParseError::MissKey);
        assert_eq!(Value::parse("{true:1,").err().unwrap(), ParseError::MissKey);
        assert_eq!(Value::parse("{false:1,").err().unwrap(), ParseError::MissKey);
        assert_eq!(Value::parse("{null:1,").err().unwrap(), ParseError::MissKey);
        assert_eq!(Value::parse("{[]:1,").err().unwrap(), ParseError::MissKey);
        assert_eq!(Value::parse("{{}:1,").err().unwrap(), ParseError::MissKey);
        assert_eq!(Value::parse(r#"{"a":1,"#).err().unwrap(), ParseError::MissKey);
    }

    #[test]
    fn parse_miss_colon() {
        assert_eq!(Value::parse(r#"{"a""#).err().unwrap(), ParseError::MissColon);
        assert_eq!(Value::parse(r#"{"a","b"}"#).err().unwrap(), ParseError::MissColon);
    }

    #[test]
    fn parse_miss_comma_or_curly_bracket() {
        assert_eq!(
            Value::parse(r#"{"a":1"#).err().unwrap(),
            ParseError::MissCommaOrCurlyBracket
        );
        assert_eq!(
            Value::parse(r#"{"a":1]"#).err().unwrap(),
            ParseError::MissCommaOrCurlyBracket
        );
        assert_eq!(
            Value::parse(r#"{"a":1 "b"}"#).err().unwrap(),
            ParseError::MissCommaOrCurlyBracket
        );
        assert_eq!(
            Value::parse(r#"{"a":{}"#).err().unwrap(),
            ParseError::MissCommaOrCurlyBracket
        );
    }

    fn test_roundtrip(json: &str) {
        let v1 = Value::parse(json).unwrap();
        match Value::parse(&v1.to_string()) {
            Ok(v2) => assert_eq!(v1, v2),
            Err(e) => {
                println!("json:\n{}", json);
                Err(e).unwrap()
            }
        }
    }

    #[test]
    fn stringify_literal() {
        test_roundtrip("null");
        test_roundtrip("false");
        test_roundtrip("true");
    }

    #[test]
    fn stringify_number() {
        test_roundtrip("0");
        test_roundtrip("-0");
        test_roundtrip("1");
        test_roundtrip("-1");
        test_roundtrip("1.5");
        test_roundtrip("-1.5");
        test_roundtrip("3.25");
        test_roundtrip("1e+20");
        test_roundtrip("1.234e+20");
        test_roundtrip("1.234e-20");

        test_roundtrip("1.0000000000000002"); /* the smallest number > 1 */
        test_roundtrip("4.9406564584124654e-324"); /* minimum denormal */
        test_roundtrip("-4.9406564584124654e-324");
        test_roundtrip("2.2250738585072009e-308"); /* Max subnormal double */
        test_roundtrip("-2.2250738585072009e-308");
        test_roundtrip("2.2250738585072014e-308"); /* Min normal positive double */
        test_roundtrip("-2.2250738585072014e-308");
        test_roundtrip("1.7976931348623157e+308"); /* Max double */
        test_roundtrip("-1.7976931348623157e+308");
    }

    #[test]
    fn stringify_string() {
        test_roundtrip(r#""""#);
        test_roundtrip(r#""Hello""#);
        test_roundtrip(r#""Hello\nWorld""#);
        test_roundtrip(r#""\" \\ / \b \f \n \r \t""#);
        test_roundtrip(r#""Hello\u0000World""#);
    }

    #[test]
    fn stringify_array() {
        test_roundtrip("[]");
        test_roundtrip("[null,false,true,123,\"abc\",[1,2,3]]");
    }

    #[test]
    fn stringify_object() {
        test_roundtrip("{}");
        test_roundtrip(
            "{\"n\":null,\"f\":false,\"t\":true,\"i\":123,\"s\":\"abc\",\"a\":[1,2,3],\"o\":{\"1\":1,\"2\":2,\"3\":3}}",
        );
    }
}
