use sbor::*;

use crate::types::rust::vec::Vec;

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
    use crate::types::rust::string::String;
    use crate::types::rust::string::ToString;
    use crate::types::rust::vec;
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

    #[test]
    fn test_dynamic_structure() {
        let _: ComponentTest = super::scrypto_decode(&vec![
            20, 22, 3, 0, 0, 0, 8, 0, 0, 0, 114, 101, 115, 111, 117, 114, 99, 101, 50, 27, 0, 0, 0,
            3, 72, 109, 231, 102, 101, 227, 204, 130, 38, 99, 98, 255, 17, 12, 155, 148, 159, 237,
            15, 148, 196, 38, 1, 126, 226, 223, 6, 0, 0, 0, 116, 111, 107, 101, 110, 115, 20, 22,
            1, 0, 0, 0, 3, 0, 0, 0, 98, 105, 100, 51, 37, 0, 0, 0, 1, 141, 104, 35, 89, 18, 207,
            204, 62, 103, 41, 191, 51, 197, 108, 219, 123, 77, 108, 139, 186, 56, 135, 189, 31,
            145, 238, 68, 18, 153, 98, 1, 134, 1, 0, 0, 0, 6, 0, 0, 0, 115, 101, 99, 114, 101, 116,
            12, 3, 0, 0, 0, 97, 98, 99,
        ])
        .unwrap();
    }
}
