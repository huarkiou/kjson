mod de;
mod error;
mod ser;

#[allow(unused)]
pub use de::from_str;
#[allow(unused)]
pub use error::JsonError;
#[allow(unused)]
pub use ser::to_string;

#[cfg(test)]
mod tests {
    use crate::Value;

    use super::*;
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Person {
        name: String,
        age: u32,
        hobbies: Vec<String>,
    }

    #[test]
    fn test_serde_support() {
        let person = Person {
            name: "Alice".to_string(),
            age: 30,
            hobbies: vec!["reading".to_string(), "coding".to_string()],
        };

        let json_str = to_string(&person).unwrap();
        assert_eq!(json_str, r#"{"name":"Alice","age":30,"hobbies":["reading","coding"]}"#);

        let deserialized: Person = from_str(&json_str).unwrap();
        assert_eq!(deserialized, person);
    }

    #[test]
    fn test_serde_value() {
        let json_str = r#"{"name":"Alice","age":30,"hobbies":["reading","coding"]}"#;
        let v = Value::parse(json_str).unwrap();

        let person1: Person = from_str(&to_string(&v).unwrap()).unwrap();
        let person2: Person = from_str(&json_str).unwrap();
        assert_eq!(person1, person2);
    }
}
