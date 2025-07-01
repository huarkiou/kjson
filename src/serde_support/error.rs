use std::fmt::{self, Display};

#[derive(Debug)]
pub enum JsonError {
    // One or more variants that can be created by data structures through the
    // `ser::Error` and `de::Error` traits. For example the Serialize impl for
    // Mutex<T> might return an error because the mutex is poisoned, or the
    // Deserialize impl for a struct may return an error because a required
    // field is missing.
    Message(String),

    // Zero or more variants that can be created directly by the Serializer and
    // Deserializer without going through `ser::Error` and `de::Error`. These
    // are specific to the format, in this case JSON.
    Eof,
    Syntax,
    ExpectedBoolean,
    ExpectedInteger,
    ExpectedString,
    ExpectedNull,
    ExpectedArray,
    ExpectedArrayComma,
    ExpectedArrayEnd,
    ExpectedMap,
    ExpectedMapColon,
    ExpectedMapComma,
    ExpectedMapEnd,
    ExpectedEnum,
    TrailingCharacters,
}

impl Display for JsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JsonError::Message(msg) => formatter.write_str(msg),
            JsonError::Eof => formatter.write_str("unexpected end of input"),
            JsonError::Syntax => formatter.write_str("syntax error"),
            JsonError::ExpectedBoolean => formatter.write_str("expected boolean"),
            JsonError::ExpectedInteger => formatter.write_str("expectedInteger"),
            JsonError::ExpectedString => formatter.write_str("expected string"),
            JsonError::ExpectedNull => formatter.write_str("expected null"),
            JsonError::ExpectedArray => formatter.write_str("expected array"),
            JsonError::ExpectedArrayComma => formatter.write_str("expected array comma"),
            JsonError::ExpectedArrayEnd => formatter.write_str("expected array end"),
            JsonError::ExpectedMap => formatter.write_str("expected map"),
            JsonError::ExpectedMapColon => formatter.write_str("expected map colon"),
            JsonError::ExpectedMapComma => formatter.write_str("expected map comma"),
            JsonError::ExpectedMapEnd => formatter.write_str("expected map end"),
            JsonError::ExpectedEnum => formatter.write_str("expected enum"),
            JsonError::TrailingCharacters => formatter.write_str("trailing characters"),
        }
    }
}

impl std::error::Error for JsonError {}

impl serde::ser::Error for JsonError {
    fn custom<T: Display>(msg: T) -> Self {
        JsonError::Message(msg.to_string())
    }
}

impl serde::de::Error for JsonError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        JsonError::Message(msg.to_string())
    }
}
