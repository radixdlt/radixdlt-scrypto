use sbor::rust::vec::Vec;
use sbor::*;

use crate::buffer::*;
use radix_engine_lib::data::*;

/// Encodes a data structure into byte array.
pub fn scrypto_encode<T: Encode<ScryptoCustomTypeId> + ?Sized>(v: &T) -> Vec<u8> {
    encode(v)
}

/// Decodes an instance of `T` from a slice.
pub fn scrypto_decode<T: Decode<ScryptoCustomTypeId>>(buf: &[u8]) -> Result<T, DecodeError> {
    decode(buf)
}

/// Encodes a data structure into a Scrypto buffer.
pub fn scrypto_encode_to_buffer<T: Encode<ScryptoCustomTypeId> + ?Sized>(v: &T) -> *mut u8 {
    let bytes = scrypto_encode(v);
    scrypto_alloc_initialized(bytes)
}

/// Decode a data structure from a Scrypto buffer.
pub fn scrypto_decode_from_buffer<T: Decode<ScryptoCustomTypeId> + ?Sized>(
    ptr: *mut u8,
) -> Result<T, DecodeError> {
    scrypto_consume(ptr, |slice| scrypto_decode(slice))
}

/// Decode a data structure from a Scrypto buffer.
pub fn scrypto_raw_from_buffer(ptr: *mut u8) -> Vec<u8> {
    // TODO: Rather than to_vec(), just take ownership
    scrypto_consume(ptr, |slice| slice.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbor::rust::string::String;

    #[test]
    fn test_encode_for_radix_engine() {
        let encoded = scrypto_encode_to_buffer("abc");
        let decoded: String = scrypto_decode_from_buffer(encoded).unwrap();
        assert_eq!(decoded, "abc");
    }
}
