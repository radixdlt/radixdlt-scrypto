mod address;
mod bid;
mod hash;
mod u256;

pub use address::*;
pub use bid::*;
pub use hash::*;
pub use u256::*;

extern crate alloc;
use alloc::string::String;

use sbor::*;

#[derive(Debug, Clone, Encode, Decode)]
pub struct ComponentInfo {
    pub blueprint: Address,
    pub name: String,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ResourceInfo {
    pub symbol: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub icon_url: String,
    pub minter: Option<Address>,
    pub supply: Option<U256>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum Level {
    Error = 0,
    Warn,
    Info,
    Debug,
    Trace,
}

#[cfg(test)]
mod tests {
    use sbor::{Decode, Encode};

    use crate::buffer::*;
    use crate::types::*;

    #[derive(Debug, Encode, Decode)]
    struct Test {
        address: Address,
        hash: Hash,
        bid: BID,
        value: U256,
    }

    #[test]
    fn test_from_to_string() {
        let obj = Test {
            address: "040377bac8066e51cd0d6b320c338d5abbcdbcca25572b6b3eee9443eafc92106bba".into(),
            hash: "374c00efbe61f645a8b35d7746e106afa7422877e5d607975b6018e0a1aa6bf0".into(),
            bid: BID::Transient(5),
            value: 1000.into(),
        };
        let bytes = scrypto_encode(&obj);
        let obj2: Test = scrypto_decode(&bytes).unwrap();
        let bytes2 = scrypto_encode(&obj2);
        assert_eq!(bytes, bytes2);
    }
}
