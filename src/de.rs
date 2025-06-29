use crate::error::ParseError;
use crate::value::Value;

pub fn from_str(json: &str) -> Result<Value, ParseError> {
    Value::parse(json)
}

pub fn from_slice(json: &[u8]) -> Result<Value, ParseError> {
    Value::parse_slice(json)
}
