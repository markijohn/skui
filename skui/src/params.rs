use std::collections::HashMap;
use crate::{Value, ValueKey};

#[derive(Debug, Clone)]
pub enum Parameters {
    Map(HashMap<String,Value>),
    Args(Vec<Value>),
}

impl Parameters {
    pub fn get_as_rk(&self, key: &[ValueKey]) -> Option<&Value> {
        if key.len() == 0 { return None }
        let first = &key[0];
        let find = match first {
            ValueKey::Index(idx) => {
                if let Parameters::Args(list) = self {
                    list.get(*idx)
                } else { None }
            }
            ValueKey::Name(name) => {
                if let Parameters::Map(map) = self {
                    map.get(name)
                } else { None }
            }
        };
        if key.len() == 1 { find } else { find.and_then(|v| v.get_as_rk(&key[1..])) }
    }
}