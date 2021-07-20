extern crate alloc;
use alloc::vec::Vec;

mod address;
mod hash;
mod rid;
mod u256;

pub use address::*;
pub use hash::*;
pub use rid::*;
pub use u256::*;

/// A recursive, schemaless value used for exchange.
pub type Value = serde_json::Value;

/// The serialized form of a `Value`.
pub type SerializedValue = Vec<u8>;

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::string::ToString;

    use serde::{Deserialize, Serialize};

    use crate::types::*;

    #[derive(Debug, Serialize, Deserialize)]
    struct Test {
        address: Address,
        hash: Hash,
        rid: RID,
        value: U256,
    }

    #[test]
    fn test_from_to_string() {
        let t = Test {
            address: "040377bac8066e51cd0d6b320c338d5abbcdbcca25572b6b3eee9443eafc92106bba".into(),
            hash: "374c00efbe61f645a8b35d7746e106afa7422877e5d607975b6018e0a1aa6bf0".into(),
            rid: RID::new(ResourceType::Badges, "id".to_string()),
            value: 1000.into(),
        };
        let expected = "{\"address\":\"040377bac8066e51cd0d6b320c338d5abbcdbcca25572b6b3eee9443eafc92106bba\",\"hash\":\"374c00efbe61f645a8b35d7746e106afa7422877e5d607975b6018e0a1aa6bf0\",\"rid\":{\"kind\":\"Badges\",\"id\":\"id\"},\"value\":\"1000\"}";
        assert_eq!(serde_json::to_string(&t).unwrap(), expected);
    }
}
