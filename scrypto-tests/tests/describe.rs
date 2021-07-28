use scrypto::abi::Describe;
use scrypto_derive::Describe;

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
    let abi1 = TestStructNamed::describe();
    assert_eq!("{\"type\":\"Struct\",\"name\":\"TestStructNamed\",\"fields\":{\"type\":\"Named\",\"fields\":{\"state\":{\"type\":\"U32\"}}}}", serde_json::to_string(&abi1).unwrap());

    let abi2 = TestStructUnnamed::describe();
    assert_eq!("{\"type\":\"Struct\",\"name\":\"TestStructUnnamed\",\"fields\":{\"type\":\"Unnamed\",\"fields\":[{\"type\":\"U32\"}]}}", serde_json::to_string(&abi2).unwrap());

    let abi3 = TestStructUnit::describe();
    assert_eq!("{\"type\":\"Struct\",\"name\":\"TestStructUnit\",\"fields\":{\"type\":\"Named\",\"fields\":{}}}", serde_json::to_string(&abi3).unwrap());
}

#[test]
fn test_describe_enum() {
    let abi1 = TestEnum::describe();
    assert_eq!("{\"type\":\"Enum\",\"name\":\"TestEnum\",\"variants\":{\"A\":{\"type\":\"Unit\"},\"B\":{\"type\":\"Unnamed\",\"fields\":[{\"type\":\"U32\"}]},\"C\":{\"type\":\"Named\",\"fields\":{\"x\":{\"type\":\"U32\"},\"y\":{\"type\":\"U32\"}}}}}", serde_json::to_string(&abi1).unwrap());
}
