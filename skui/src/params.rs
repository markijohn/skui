use std::collections::HashMap;
use tinyvec::ArrayVec;
use crate::{Value, ValueKey};

#[derive(Debug, Clone)]
pub enum Parameters<'a> {
    Map(HashMap<&'a str,Value<'a>>),
    //Args(ArrayVec<[Value<'a>;7]>),
    Args(Vec<Value<'a>>),
}

impl <'a> Parameters<'a> {
    pub fn get_as_rk(&self, key: &'a [ValueKey]) -> Option<&Value> {
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