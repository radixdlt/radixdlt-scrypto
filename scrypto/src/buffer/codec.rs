use sbor::rust::vec::Vec;
use sbor::*;

use crate::buffer::*;

/// Encodes a data structure into byte array.
pub fn scrypto_encode<T: Encode + ?Sized>(v: &T) -> Vec<u8> {
    encode_with_static_info(v)
}

/// Decodes an instance of `T` from a slice.
pub fn scrypto_decode<T: Decode>(buf: &[u8]) -> Result<T, DecodeError> {
    decode_with_static_info(buf)
}

/// Encodes a data structure into a Scrypto buffer.
pub fn scrypto_encode_to_buffer<T: Encode + ?Sized>(v: &T) -> *mut u8 {
    let bytes = scrypto_encode(v);
    scrypto_alloc_initialized(bytes)
}

/// Decode a data structure from a Scrypto buffer.
pub fn scrypto_decode_from_buffer<T: Decode + ?Sized>(ptr: *mut u8) -> Result<T, DecodeError> {
    scrypto_consume(ptr, |slice| scrypto_decode(slice))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_for_radix_engine() {
        let encoded = scrypto_encode_to_buffer("abc");
        let decoded: String = scrypto_decode_from_buffer(encoded).unwrap();
        assert_eq!(decoded, "abc");
    }
}
