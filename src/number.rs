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
