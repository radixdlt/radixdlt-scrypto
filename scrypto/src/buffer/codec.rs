use sbor::*;

use crate::rust::vec::Vec;

/// Encodes a data structure into byte array.
pub fn scrypto_encode<T: Encode>(v: &T) -> Vec<u8> {
    sbor::encode_with_metadata(v)
}

/// Decodes an instance of `T` from a slice.
pub fn scrypto_decode<'de, T: Decode>(buf: &'de [u8]) -> Result<T, DecodeError> {
    sbor::decode_with_metadata(buf)
}

#[cfg(test)]
mod tests {
    use sbor::*;

    use crate::buffer::*;
    use crate::kernel::*;
    use crate::resource::*;
    use crate::rust::string::String;
    use crate::rust::string::ToString;
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

    #[derive(Encode, Decode)]
    struct ComponentTest {
        resource: Address,
        tokens: Tokens,
        secret: String,
    }
}
