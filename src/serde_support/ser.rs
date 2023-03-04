use std::fmt::Display;

use serde::ser::{
    Error, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};
use serde::{Serialize, Serializer};


use crate::{Expr, Symbol, WolframError};

/// Serialize a value into a Wolfram Language expression.
pub struct WolframSerializer {
    // pub readable: bool,
}

pub struct WolframListSerializer<'a> {
    config: &'a WolframSerializer,
    namespace: &'static str,
    variant: &'static str,
    terms: Vec<Expr>,
}

pub struct WolframDictSerializer<'a> {
    config: &'a WolframSerializer,
    namespace: &'static str,
    variant: &'static str,
    rules: Vec<Expr>,
}


impl Error for WolframError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        WolframError::runtime_error(msg)
    }
}

impl<'a> WolframListSerializer<'a> {
    #[inline]
    pub fn new(capacity: usize, config: &'a WolframSerializer) -> Self {
        Self {
            config,
            namespace: "",
            variant: "",
            terms: Vec::with_capacity(capacity),
        }
    }
    #[inline(always)]
    pub fn with_name(mut self, class: &'static str, variant: &'static str) -> Self {
        self.namespace = class;
        self.variant = variant;
        self
    }
}

impl<'a> WolframDictSerializer<'a> {
    pub fn new(capacity: usize, config: &'a WolframSerializer) -> Self {
        Self {
            config,
            namespace: "",
            variant: "",
            rules: Vec::with_capacity(capacity),
        }
    }
    pub fn with_name(mut self, namespace: &'static str, variant: &'static str) -> Self {
        self.namespace = namespace;
        self.variant = variant;
        self
    }
}


impl<'a> Serializer for &'a WolframSerializer {
    type Ok = Expr;
    type Error = WolframError;
    type SerializeSeq = WolframListSerializer<'a>;
    type SerializeTuple = WolframListSerializer<'a>;
    type SerializeTupleStruct = WolframListSerializer<'a>;
    type SerializeTupleVariant = WolframListSerializer<'a>;
    type SerializeMap = WolframDictSerializer<'a>;
    type SerializeStruct = WolframDictSerializer<'a>;
    type SerializeStructVariant = WolframDictSerializer<'a>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Expr::from(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Expr::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Expr::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Expr::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Expr::from(v))
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        unimplemented!("serialize i128 `{v}` is not supported yet")
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(Expr::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(Expr::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(Expr::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        unimplemented!("serialize u64 `{v}` is not supported yet")
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        unimplemented!("serialize u128 `{v}` is not supported yet")
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        if v.is_nan() {
            return Ok(Expr::symbol(Symbol::new("System`Indeterminate")));
        }
        if v.is_infinite() {
            return Ok(Expr::symbol(Symbol::new("System`Infinity")));
        }
        Ok(Expr::real(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Expr::from(v))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        unimplemented!("serialize bytes `{v:?}` is not supported yet")
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Expr::symbol(Symbol::new("System`None")))
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Expr::null())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(Expr::normal(Symbol::new(&format!("Global`{name}")), vec![]))
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        unimplemented!("serialize unit variant `{name}{variant_index}{variant}` is not supported yet")
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        let _ = value;
        unimplemented!("serialize newtype struct `{name}` is not supported yet")
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        let _ = (value, variant, variant_index);
        unimplemented!("serialize newtype struct `{name}` is not supported yet")
    }

    fn serialize_seq(
        self,
        len: Option<usize>,
    ) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(WolframListSerializer::new(len.unwrap_or(0), &self))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(WolframListSerializer::new(len, &self))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(WolframListSerializer::new(len, &self).with_name("", name))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(WolframListSerializer::new(len, &self).with_name(name, variant))
    }

    fn serialize_map(
        self,
        len: Option<usize>,
    ) -> Result<Self::SerializeMap, Self::Error> {
        Ok(WolframDictSerializer::new(len.unwrap_or(0), &self))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(WolframDictSerializer::new(len, &self).with_name("", name))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(WolframDictSerializer::new(len, &self).with_name(name, variant))
    }

    fn collect_seq<I>(self, iter: I) -> Result<Self::Ok, Self::Error>
    where
        I: IntoIterator,
        <I as IntoIterator>::Item: Serialize,
    {
        let iter = iter.into_iter();
        let mut serializer = WolframListSerializer::new(iter.size_hint().0, &self);
        for item in iter {
            SerializeSeq::serialize_element(&mut serializer, &item)?;
        }
        SerializeSeq::end(serializer)
    }

    fn collect_map<K, V, I>(self, iter: I) -> Result<Self::Ok, Self::Error>
    where
        K: Serialize,
        V: Serialize,
        I: IntoIterator<Item = (K, V)>,
    {
        let _ = iter.into_iter();
        unimplemented!("collect map is not supported yet")
    }

    fn collect_str<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Display,
    {
        unimplemented!("collect str `{value}` is not supported yet")
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}

impl<'a> SerializeSeq for WolframListSerializer<'a> {
    type Ok = Expr;
    type Error = WolframError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.terms.push(value.serialize(self.config)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let head = if self.namespace.is_empty() {
            Symbol::new("System`List")
        } else if self.namespace.is_empty() {
            Symbol::new(self.variant)
        } else {
            Symbol::new(&format!("{}`{}", self.namespace, self.variant))
        };
        Ok(Expr::normal(head, self.terms))
    }
}


impl<'a> SerializeTuple for WolframListSerializer<'a> {
    type Ok = Expr;
    type Error = WolframError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl<'a> SerializeTupleStruct for WolframListSerializer<'a> {
    type Ok = Expr;
    type Error = WolframError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl<'a> SerializeTupleVariant for WolframListSerializer<'a> {
    type Ok = Expr;
    type Error = WolframError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl<'a> SerializeMap for WolframDictSerializer<'a> {
    type Ok = Expr;
    type Error = WolframError;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let _ = key;
        unimplemented!("serialize key is not supported yet")
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let _ = value;
        unimplemented!("serialize value is not supported yet")
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!("end is not supported yet")
    }
}

impl<'a> SerializeStruct for WolframDictSerializer<'a> {
    type Ok = Expr;
    type Error = WolframError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let lhs = Expr::string(key);
        let rhs = value.serialize(self.config)?;
        self.rules.push(Expr::rule(lhs, rhs));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Expr::normal(Symbol::new("System`Association"), self.rules))
    }
}

impl<'a> SerializeStructVariant for WolframDictSerializer<'a> {
    type Ok = Expr;
    type Error = WolframError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeStruct::end(self)
    }
}
