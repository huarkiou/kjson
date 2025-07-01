use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    Int(i64),
    UInt(u64),
    Float(f64),
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Int(n) => n.fmt(f),
            Number::UInt(n) => n.fmt(f),
            Number::Float(n) => n.fmt(f),
        }
    }
}

impl Into<Number> for i64 {
    fn into(self) -> Number {
        Number::Int(self)
    }
}

impl Into<Number> for u64 {
    fn into(self) -> Number {
        Number::UInt(self)
    }
}

impl Into<Number> for f64 {
    fn into(self) -> Number {
        Number::Float(self)
    }
}
