#![cfg_attr(not(feature = "std"), no_std)]

use serde::Serialize;
use serde_json::{to_string, to_value, Value};

pub fn assert_json_eq<T: Serialize>(actual: T, expected: Value) {
    let actual = to_value(&actual).unwrap();
    if actual != expected {
        panic!(
            "Mismatching JSONs:\nActual: {}\nExpected: {}\n",
            to_string(&actual).unwrap(),
            to_string(&expected).unwrap()
        );
    }
}
