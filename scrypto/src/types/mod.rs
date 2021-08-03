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

/// A value that encloses data and resources, used for communication between components.
///
/// For now, it's a JSON value but will be replaced with Radix data format.
pub type Value = sbor::Value;

/// The serialized form of a `Value`.
pub type SerializedValue = Vec<u8>;

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::string::ToString;

    use sbor::{Decode, Encode};

    use crate::buffer::*;
    use crate::types::*;

    #[derive(Debug, Encode, Decode)]
    struct Test {
        address: Address,
        hash: Hash,
        rid: RID,
        value: U256,
    }

    #[test]
    fn test_from_to_string() {
        let obj = Test {
            address: "040377bac8066e51cd0d6b320c338d5abbcdbcca25572b6b3eee9443eafc92106bba".into(),
            hash: "374c00efbe61f645a8b35d7746e106afa7422877e5d607975b6018e0a1aa6bf0".into(),
            rid: RID::new(ResourceKind::Badges, "id".to_string()),
            value: 1000.into(),
        };
        let bytes = scrypto_encode(&obj);
        let obj2: Test = scrypto_decode(&bytes);
        let bytes2 = scrypto_encode(&obj2);
        assert_eq!(bytes, bytes2);
    }
}
