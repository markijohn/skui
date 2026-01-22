use std::collections::HashMap;
use crate::Value;

#[derive(Debug, Clone)]
pub enum Parameters {
    Map(HashMap<String,Value>),
    Args(Vec<Value>),
}

pub trait ValueKey {
    fn from_value<'a>(&self, v: &'a Parameters) -> Option<&'a Value>;
}

impl ValueKey for str {
    fn from_value<'a>(&self, v: &'a Parameters) -> Option<&'a Value> {
        if let Parameters::Map(map) = v {
            map.get(self)
        } else {
            None
        }
    }
}

impl ValueKey for usize {
    fn from_value<'a>(&self, v: &'a Parameters) -> Option<&'a Value> {
        if let Parameters::Args( arr ) = v {
            arr.get(*self)
        } else {
            None
        }
    }
}

impl Parameters {
    pub fn get<K:ValueKey>(&self, key: K) -> Option<&Value> {
        key.from_value(self)
    }
}