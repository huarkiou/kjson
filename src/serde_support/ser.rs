use crate::serde_support::error::JsonError;
use crate::{number::Number, value::Value};
use serde::ser::{
    Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant, Serializer,
};

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::Number(Number::Int(n)) => serializer.serialize_i64(*n),
            Value::Number(Number::UInt(n)) => serializer.serialize_u64(*n),
            Value::Number(Number::Float(n)) => serializer.serialize_f64(*n),
            Value::String(s) => serializer.serialize_str(s),
            Value::Array(arr) => {
                let mut seq = serializer.serialize_seq(Some(arr.len()))?;
                for value in arr {
                    seq.serialize_element(value)?;
                }
                seq.end()
            }
            Value::Object(obj) => {
                let mut map = serializer.serialize_map(Some(obj.len()))?;
                for (key, value) in obj.iter() {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
        }
    }
}

pub fn to_string<T>(value: &T) -> Result<String, JsonError>
where
    T: Serialize,
{
    let mut serializer = JsonSerializer::new();
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl JsonSerializer {
    fn new() -> Self {
        JsonSerializer { output: String::new() }
    }
}

struct JsonSerializer {
    output: String,
}

impl<'a> Serializer for &'a mut JsonSerializer {
    type Ok = ();
    type Error = JsonError;

    // 处理序列化结构的入口
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.output += if v { "true" } else { "false" };
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        let _ = v;
        Err(serde::ser::Error::custom("i128 is not supported"))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        let _ = v;
        Err(serde::ser::Error::custom("u128 is not supported"))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.output += "\"";
        self.output += v;
        self.output += "\"";
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            SerializeSeq::serialize_element(&mut seq, byte)?;
        }
        // seq.end()
        SerializeSeq::end(seq)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.output += "null";
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.output += "{";
        variant.serialize(&mut *self)?;
        self.output += ":";
        value.serialize(&mut *self)?;
        self.output += "}";
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.output += "[";
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.output += "{";
        variant.serialize(&mut *self)?;
        self.output += ":[";
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.output += "{";
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.output += "{";
        variant.serialize(&mut *self)?;
        self.output += ":{";
        Ok(self)
    }
}

// The following 7 impls deal with the serialization of compound types like
// sequences and maps. Serialization of such types is begun by a Serializer
// method and followed by zero or more calls to serialize individual elements of
// the compound type and one call to end the compound type.
//
// This impl is SerializeSeq so these methods are called after `serialize_seq`
// is called on the Serializer.
impl<'a> SerializeSeq for &'a mut JsonSerializer {
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = JsonError;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok, JsonError>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('[') {
            self.output += ",";
        }
        value.serialize(&mut **self)
    }

    // Close the sequence.
    fn end(self) -> Result<Self::Ok, JsonError> {
        self.output += "]";
        Ok(())
    }
}

// Same thing but for tuples.
impl<'a> SerializeTuple for &'a mut JsonSerializer {
    type Ok = ();
    type Error = JsonError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok, JsonError>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('[') {
            self.output += ",";
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, JsonError> {
        self.output += "]";
        Ok(())
    }
}

// Same thing but for tuple structs.
impl<'a> SerializeTupleStruct for &'a mut JsonSerializer {
    type Ok = ();
    type Error = JsonError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok, JsonError>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('[') {
            self.output += ",";
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, JsonError> {
        self.output += "]";
        Ok(())
    }
}

// Tuple variants are a little different. Refer back to the
// `serialize_tuple_variant` method above:
//
//    self.output += "{";
//    variant.serialize(&mut *self)?;
//    self.output += ":[";
//
// So the `end` method in this impl is responsible for closing both the `]` and
// the `}`.
impl<'a> SerializeTupleVariant for &'a mut JsonSerializer {
    type Ok = ();
    type Error = JsonError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok, JsonError>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('[') {
            self.output += ",";
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, JsonError> {
        self.output += "]}";
        Ok(())
    }
}

// Some `Serialize` types are not able to hold a key and value in memory at the
// same time so `SerializeMap` implementations are required to support
// `serialize_key` and `serialize_value` individually.
//
// There is a third optional method on the `SerializeMap` trait. The
// `serialize_entry` method allows serializers to optimize for the case where
// key and value are both available simultaneously. In JSON it doesn't make a
// difference so the default behavior for `serialize_entry` is fine.
impl<'a> SerializeMap for &'a mut JsonSerializer {
    type Ok = ();
    type Error = JsonError;

    // The Serde data model allows map keys to be any serializable type. JSON
    // only allows string keys so the implementation below will produce invalid
    // JSON if the key serializes as something other than a string.
    //
    // A real JSON serializer would need to validate that map keys are strings.
    // This can be done by using a different Serializer to serialize the key
    // (instead of `&mut **self`) and having that other serializer only
    // implement `serialize_str` and return an error on any other data type.
    fn serialize_key<T>(&mut self, key: &T) -> Result<Self::Ok, JsonError>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('{') {
            self.output += ",";
        }
        key.serialize(&mut **self)
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> Result<Self::Ok, JsonError>
    where
        T: ?Sized + Serialize,
    {
        self.output += ":";
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, JsonError> {
        self.output += "}";
        Ok(())
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'a> SerializeStruct for &'a mut JsonSerializer {
    type Ok = ();
    type Error = JsonError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<Self::Ok, JsonError>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('{') {
            self.output += ",";
        }
        key.serialize(&mut **self)?;
        self.output += ":";
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, JsonError> {
        self.output += "}";
        Ok(())
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a> SerializeStructVariant for &'a mut JsonSerializer {
    type Ok = ();
    type Error = JsonError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<Self::Ok, JsonError>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('{') {
            self.output += ",";
        }
        key.serialize(&mut **self)?;
        self.output += ":";
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, JsonError> {
        self.output += "}}";
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    #[test]
    fn test_struct() {
        #[derive(Serialize)]
        struct Test {
            int: u32,
            seq: Vec<&'static str>,
        }

        let test = Test {
            int: 1,
            seq: vec!["a", "b"],
        };
        let expected = r#"{"int":1,"seq":["a","b"]}"#;
        assert_eq!(to_string(&test).unwrap(), expected);
    }

    #[test]
    fn test_enum() {
        #[derive(Serialize)]
        enum E {
            Unit,
            Newtype(u32),
            Tuple(u32, u32),
            Struct { a: u32 },
        }

        let u = E::Unit;
        let expected = r#""Unit""#;
        assert_eq!(to_string(&u).unwrap(), expected);

        let n = E::Newtype(1);
        let expected = r#"{"Newtype":1}"#;
        assert_eq!(to_string(&n).unwrap(), expected);

        let t = E::Tuple(1, 2);
        let expected = r#"{"Tuple":[1,2]}"#;
        assert_eq!(to_string(&t).unwrap(), expected);

        let s = E::Struct { a: 1 };
        let expected = r#"{"Struct":{"a":1}}"#;
        assert_eq!(to_string(&s).unwrap(), expected);
    }
}
