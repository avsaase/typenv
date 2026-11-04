//! A serde deserializer for a single environment variable value.

use serde::de::value::{Error, StrDeserializer};
use serde::de::{self, Deserializer, IntoDeserializer, Visitor};

/// Deserializes one string value. Numbers, booleans and `char` are parsed with `FromStr` first,
/// because their `Deserialize` impls do not accept a string. Enums are matched by variant name and
/// everything else is given the string as is.
pub(super) struct ValueDeserializer<'de>(pub(super) &'de str);

/// Parses the value into the requested primitive before handing it to the visitor.
macro_rules! parse_primitive {
    ($($method:ident => $visit:ident($ty:ty),)*) => {
        $(
            fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
                let value: $ty = self.0.parse().map_err(de::Error::custom)?;
                visitor.$visit(value)
            }
        )*
    };
}

impl<'de> Deserializer<'de> for ValueDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_borrowed_str(self.0)
    }

    parse_primitive! {
        deserialize_bool => visit_bool(bool),
        deserialize_i8 => visit_i8(i8),
        deserialize_i16 => visit_i16(i16),
        deserialize_i32 => visit_i32(i32),
        deserialize_i64 => visit_i64(i64),
        deserialize_i128 => visit_i128(i128),
        deserialize_u8 => visit_u8(u8),
        deserialize_u16 => visit_u16(u16),
        deserialize_u32 => visit_u32(u32),
        deserialize_u64 => visit_u64(u64),
        deserialize_u128 => visit_u128(u128),
        deserialize_f32 => visit_f32(f32),
        deserialize_f64 => visit_f64(f64),
        deserialize_char => visit_char(char),
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        // The variable is set, otherwise the parser would not have been called.
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        let deserializer: StrDeserializer<'de, Error> = self.0.into_deserializer();
        deserializer.deserialize_enum(name, variants, visitor)
    }

    serde::forward_to_deserialize_any! {
        str string bytes byte_buf unit unit_struct seq tuple tuple_struct map struct identifier
        ignored_any
    }
}
