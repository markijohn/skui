use std::collections::HashMap;
use crate::Component;

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


#[derive(Debug, Clone)]
pub enum Value {
    Ident(String),
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Map(HashMap<String, Value>),
    Closure(String),
    Component(Component),
}


impl Value {
    pub fn is_map(&self) -> bool {
        self.as_map().is_some()
    }

    pub fn as_map(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Map(map) => Some(map),
            _ => None,
        }
    }

    pub fn as_object_mut(&mut self) -> Option<&mut HashMap<String, Value>> {
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

    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
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
}
