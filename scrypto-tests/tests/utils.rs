use serde::Serialize;
use serde_json::Value;

pub fn json_eq<T: Serialize>(expected: Value, actual: T) {
    let actual_json = serde_json::to_value(&actual).unwrap();
    assert_eq!(expected, actual_json);
}
