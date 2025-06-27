use crate::value::Value;

mod context;
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

impl ToString for Json {
    fn to_string(&self) -> String {
        self.value.to_string()
    }
}
