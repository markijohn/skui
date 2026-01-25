use std::collections::HashMap;
use std::str::FromStr;
use crate::{Component, Parameters};

#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    I64(i64),
    F64(f64),
}

impl Number {
    pub fn is_i64(&self) -> bool {
        matches!(self, Number::I64(_))
    }

    pub fn as_i64(&self) -> Option<i64> {
        if let Number::I64(i) = *self {
            Some(i)
        } else { None }
    }

    pub fn is_f64(&self) -> bool {
        matches!(self, Number::F64(_))
    }

    pub fn as_f64(&self) -> Option<f64> {
        if let Number::F64(i) = *self {
            Some(i)
        } else { None }
    }
}

pub enum InvalidValueKey {
    Empty,
    Invalid(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueKey<'a> {
    Index(usize),
    Name(&'a str),
}
//
// impl <'a> FromStr for ValueKey<'a> {
//     type Err = InvalidValueKey;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         if let Ok(i) = usize::from_str(s) {
//             Ok(Self::Index(i))
//         } else {
//             if s.len() <= 0 {
//                 Err(InvalidValueKey::Empty)
//             } else {
//                 let mut bytes = s.bytes();
//                 let first = bytes.next().unwrap();
//                 if !first.is_ascii_alphabetic() && first == b'_' {
//                     Err(InvalidValueKey::Invalid(s.to_string()))
//                 } else {
//                     if bytes.all( |c| c.is_ascii_alphanumeric() || c == b'_' ) {
//                         Ok(Self::Name(s))
//                     } else {
//                         Err(InvalidValueKey::Invalid(s.to_string()))
//                     }
//                 }
//             }
//         }
//     }
// }

impl <'a> ValueKey <'a> {
    fn from_str(s: &'a str) -> Result<Self, InvalidValueKey> {
        if let Ok(i) = usize::from_str(s) {
            Ok(Self::Index(i))
        } else {
            if s.len() <= 0 {
                Err(InvalidValueKey::Empty)
            } else {
                let mut bytes = s.bytes();
                let first = bytes.next().unwrap();
                if !first.is_ascii_alphabetic() && first == b'_' {
                    Err(InvalidValueKey::Invalid(s.to_string()))
                } else {
                    if bytes.all( |c| c.is_ascii_alphanumeric() || c == b'_' ) {
                        Ok(Self::Name(s))
                    } else {
                        Err(InvalidValueKey::Invalid(s.to_string()))
                    }
                }
            }
        }
    }
}


#[derive(Debug, Clone)]
pub enum Value<'a> {
    Ident(&'a str),
    Bool(bool),
    Number(Number),
    String(&'a str),
    Array(Vec<Value<'a>>),
    Map(HashMap<&'a str, Value<'a>>),
    Closure(&'a str),
    Component(Component<'a>),
    Relative(Vec<ValueKey<'a>>)
}

impl <'a> Default for Value<'a> {
    fn default() -> Self {
        Self::Bool(false)
    }
}

impl <'a> Value<'a> {
    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(b) = *self {
            Some(b)
        } else { None }
    }
    pub fn is_map(&self) -> bool {
        self.as_map().is_some()
    }

    pub fn as_map(&self) -> Option<&HashMap<&'a str, Value>> {
        match self {
            Value::Map(map) => Some(map),
            _ => None,
        }
    }

    pub fn as_object_mut(&mut self) -> Option<&'a mut HashMap<&'a str, Value>> {
        match self {
            Value::Map(map) => Some(map),
            _ => None,
        }
    }

    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(array) => Some(array),
            _ => None,
        }
    }

    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value<'a>>> {
        match self {
            Value::Array(list) => Some(list),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        self.as_str().is_some()
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Ident(s) => Some(s),
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn is_number(&self) -> bool {
        match *self {
            Value::Number(_) => true,
            _ => false,
        }
    }

    pub fn as_number(&self) -> Option<&Number> {
        match self {
            Value::Number(number) => Some(number),
            _ => None,
        }
    }

    pub fn is_i64(&self) -> bool {
        match self {
            Value::Number(n) => n.is_i64(),
            _ => false,
        }
    }

    pub fn is_f64(&self) -> bool {
        match self {
            Value::Number(n) => n.is_f64(),
            _ => false,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Number(n) => n.as_i64(),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(n) => n.as_f64(),
            _ => None,
        }
    }

    pub fn get_as_rk(&self, key: &'a [ValueKey]) -> Option<&'a Value> {
        if key.len() == 0 { return None }
        let first = &key[0];
        let find = match first {
            ValueKey::Index(idx) => {
                if let Value::Array(list) = self {
                    list.get(*idx)
                } else { None }
            }
            ValueKey::Name(name) => {
                if let Value::Map(map) = self {
                    map.get(name)
                } else { None }
            }
            _ => None
        };
        if key.len() == 1 { find } else { find.and_then(|v| v.get_as_rk(&key[1..])) }
    }
}


pub enum ValueError {
    NotNumber,
    NotString
}

macro_rules! impl_num {
    ( $($typ:ty),* ) => {
        $(
        impl <'a> TryFrom<&Value<'a>> for $typ {
            type Error = ValueError;
            fn try_from(value: &Value) -> Result<Self, Self::Error> {
                value.as_i64().map(|v| v as $typ).ok_or(ValueError::NotNumber)
            }
        }
        )*
    }
}

impl_num!( u8, u16, u32, u64, usize, i8, i16, i32, i64, isize );

macro_rules! impl_float {
    ( $($typ:ty),* ) => {
        $(
        impl <'a> TryFrom<&Value<'a>> for $typ {
            type Error = ValueError;
            fn try_from(value: &Value) -> Result<Self, Self::Error> {
                value.as_f64().map(|v| v as $typ).ok_or(ValueError::NotNumber)
            }
        }
        )*
    }
}

impl_float!( f32, f64 );

impl <'a> TryFrom<&'a Value<'a>> for &'a str {
    type Error = ValueError;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        value.as_str().ok_or(ValueError::NotString)
    }
}