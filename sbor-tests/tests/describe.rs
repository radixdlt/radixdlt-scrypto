use sbor::Describe;
use serde_json::json;

mod utils;
use utils::json_eq;

#[derive(Describe)]
pub struct TestStructNamed {
    pub state: u32,
}

#[derive(Describe)]
pub struct TestStructUnnamed(u32);

#[derive(Describe)]
pub struct TestStructUnit {}

#[derive(Describe)]
pub enum TestEnum {
    A,
    B(u32),
    C { x: u32, y: u32 },
}

#[test]
fn test_describe_struct() {
    json_eq(
        json!({
          "type": "Struct",
          "name": "TestStructNamed",
          "fields": {
            "type": "Named",
            "fields": {
              "state": {
                "type": "U32"
              }
            }
          }
        }),
        TestStructNamed::describe(),
    );

    json_eq(
        json!({
          "type": "Struct",
          "name": "TestStructUnnamed",
          "fields": {
            "type": "Unnamed",
            "fields": [
              {
                "type": "U32"
              }
            ]
          }
        }),
        TestStructUnnamed::describe(),
    );

    json_eq(
        json!({
          "type": "Struct",
          "name": "TestStructUnit",
          "fields": {
            "type": "Named",
            "fields": {}
          }
        }),
        TestStructUnit::describe(),
    );
}

#[test]
fn test_describe_enum() {
    json_eq(
        json!({
          "type": "Enum",
          "name": "TestEnum",
          "variants": {
            "A": {
              "type": "Unit"
            },
            "B": {
              "type": "Unnamed",
              "fields": [
                {
                  "type": "U32"
                }
              ]
            },
            "C": {
              "type": "Named",
              "fields": {
                "x": {
                  "type": "U32"
                },
                "y": {
                  "type": "U32"
                }
              }
            }
          }
        }),
        TestEnum::describe(),
    );
}
