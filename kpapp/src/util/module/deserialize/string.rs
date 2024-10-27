use std::collections::BTreeMap;
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Value};
use std::result::Result as StdResult;

pub fn string_or_number<'de, D>(deserializer: D) -> StdResult<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(s) => Ok(s),
        Value::Number(num) => Ok(num.to_string()),
        _ => Err(serde::de::Error::custom("expected a string or an integer")),
    }
}

pub fn map_string_or_number<'de, D>(deserializer: D) -> StdResult<BTreeMap<String, String>, D::Error>
where
    D: Deserializer<'de>,
{
    let map = Map::<String, Value>::deserialize(deserializer)?;
    let mut result = BTreeMap::new();
    for (key, value) in map {
        let key = match key.parse::<i64>() {
            Ok(num) => num.to_string(),
            Err(_) => key,
        };
        let value = match value {
            Value::String(s) => s,
            Value::Number(num) => num.to_string(),
            _ => return Err(serde::de::Error::custom("expected a string or an integer for the value")),
        };
        result.insert(key, value);
    }
    Ok(result)
}