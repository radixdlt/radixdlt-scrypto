use sbor::rust::vec::Vec;
use sbor::{any::encode_any_with_buffer, *};

/// Encodes a data structure into byte array.
pub fn scrypto_encode<T: Encode + ?Sized>(v: &T) -> Vec<u8> {
    encode_with_type(v)
}

/// Encodes a data structure into byte array and stores the size at the front.
pub fn scrypto_encode_with_size_prefix<T: Encode + ?Sized>(v: &T) -> Vec<u8> {
    // create a buffer and pre-append with length (0).
    let mut buf = Vec::with_capacity(512);
    buf.extend(&[0u8; 4]);

    // encode the data structure
    let mut enc = Encoder::with_type(&mut buf);
    enc.encode(v);

    // update the length field
    let len = (buf.len() - 4) as u32;
    (&mut buf[0..4]).copy_from_slice(&len.to_le_bytes());

    buf
}

/// Encodes any value into byte array and stores the size at the front.
pub fn scrypto_encode_any_with_size_prefix(v: &Value) -> Vec<u8> {
    // create a buffer and pre-append with length (0).
    let mut buf = Vec::with_capacity(512);
    buf.extend(&[0u8; 4]);

    // encode the data structure
    encode_any_with_buffer(v, &mut buf);

    // update the length field
    let len = (buf.len() - 4) as u32;
    (&mut buf[0..4]).copy_from_slice(&len.to_le_bytes());

    buf
}

/// Decodes an instance of `T` from a slice.
pub fn scrypto_decode<T: Decode>(buf: &[u8]) -> Result<T, DecodeError> {
    decode_with_type(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::api::*;
    use sbor::rust::vec;

    #[test]
    fn test_serialization() {
        let obj = GenerateUuidOutput { uuid: 123 };
        let encoded = scrypto_encode(&obj);
        let decoded = scrypto_decode::<GenerateUuidOutput>(&encoded).unwrap();
        assert_eq!(decoded.uuid, 123u128);
    }

    #[test]
    fn test_encode_for_radix_engine() {
        let encoded = scrypto_encode_with_size_prefix("abc");
        assert_eq!(vec![8, 0, 0, 0, 12, 3, 0, 0, 0, 97, 98, 99], encoded);
    }
}
