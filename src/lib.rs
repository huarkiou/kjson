use crate::value::Value;

mod context;
mod dict;
mod error;
mod number;
mod stack;
mod value;

pub struct Json {
    value: Value,
}

impl Json {
    pub fn parse(json: &str) -> Self {
        Self {
            value: Value::parse(json).unwrap(),
        }
    }
}

impl std::fmt::Display for Json {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
