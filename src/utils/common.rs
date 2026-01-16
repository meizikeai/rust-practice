use serde_json::{Map, Value};
use std::collections::HashMap;

pub fn hashmap_to_serde_map<T>(input: HashMap<String, T>) -> Map<String, Value>
where
    T: Into<Value>,
{
    input.into_iter().map(|(k, v)| (k, v.into())).collect()
}
