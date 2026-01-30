use std::collections::HashMap;
use tinyvec::ArrayVec;
use crate::{Value, ValueKey};

#[derive(Debug, Clone)]
pub enum Parameters<'a> {
    Map(HashMap<&'a str,Value<'a>>),
    Args(Vec<Value<'a>>),
}

impl <'a> Parameters<'a> {
    pub fn empty() -> Self { Parameters::Args( Vec::new() ) }
    pub fn get(&self, idx:usize, key:&'a str) -> Option<&Value> {
        match self {
            Parameters::Map(map) => map.get(key),
            Parameters::Args(list) => list.get(idx),
        }
    }
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

    pub fn consume_flat(&'a self, new:&'a Parameters<'a>) -> Parameters<'a> {
        match new {
            Parameters::Map(map) => {
                let mut new_map = HashMap::new();
                for (key,value) in map.iter() {
                    if let Value::Relative(vkey) = value {
                        if let Some(v) = self.get_as_rk(vkey.as_slice()) {
                            new_map.insert(key.clone(), v.clone());
                        } else {
                            eprintln!("Can't find relative value : {:?}. From : {:?}", vkey, self);
                        }
                    } else {
                        new_map.insert(key.clone(), value.clone());
                    }
                }
                Parameters::Map(new_map)
            },
            Parameters::Args(list) => {
                let mut new_list = Vec::new();
                for value in list.iter() {
                    if let Value::Relative(vkey) = value {
                        if let Some(v) = self.get_as_rk(vkey.as_slice()) {
                            new_list.push(v.clone());
                        } else {
                            eprintln!("Can't find relative value : {:?}. From : {:?}", vkey, self);
                        }
                        new.get_as_rk(vkey.as_slice()).map(|v| new_list.push(v.clone()));
                    } else {
                        new_list.push(value.clone());
                    }
                }
                Parameters::Args(new_list)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{Parameters, Value, ValueKey};


    #[test]
    fn test() {
        let map = Value::Map(
            [("key", Value::String("Hello World!"))].into()
        );
        let params = Parameters::Args( vec![map] );


        let vkey = ValueKey::vec_from_str("0").unwrap();
        println!("0 : {:?}", params.get_as_rk(vkey.as_slice()).unwrap());

        let vkey = ValueKey::vec_from_str("0.key").unwrap();
        println!("0.key : {:?}", params.get_as_rk(vkey.as_slice()).unwrap());
    }
}