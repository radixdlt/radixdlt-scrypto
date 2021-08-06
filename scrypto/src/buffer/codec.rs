extern crate alloc;
use alloc::vec::Vec;

use sbor::*;

/// Encodes a data structure into byte array.
pub fn scrypto_encode<T: Encode>(v: &T) -> Vec<u8> {
    sbor::sbor_encode_with_metadata(v)
}

/// Decodes an instance of `T` from a slice.
pub fn scrypto_decode<'de, T: Decode>(buf: &'de [u8]) -> Result<T, DecodeError> {
    sbor::sbor_decode_with_metadata(buf)
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::string::String;
    use alloc::string::ToString;

    use crate::buffer::*;
    use crate::kernel::*;
    use crate::types::*;

    #[test]
    fn test_serialization() {
        let obj = PutComponentStateInput {
            component: Address::System,
            state: scrypto_encode(&"test".to_string()),
        };
        let encoded = crate::buffer::scrypto_encode(&obj);
        let decoded = crate::buffer::scrypto_decode::<PutComponentStateInput>(&encoded).unwrap();
        assert_eq!(decoded.component, Address::System);
        assert_eq!(scrypto_decode::<String>(&decoded.state).unwrap(), "test");
    }
}
