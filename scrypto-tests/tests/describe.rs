use scrypto::abi::Describe;
use scrypto_derive::Describe;

#[derive(Describe)]
pub struct Simple {
    pub state: u32,
}

#[test]
fn test_describe_simple() {
    let abi = Simple::describe();
    println!("{:?}", abi);
}
