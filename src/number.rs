use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum Number {
    Int(i64),
    Float(f64),
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Int(n) => n.fmt(f),
            Number::Float(n) => n.fmt(f),
        }
    }
}
