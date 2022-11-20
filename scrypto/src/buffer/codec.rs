use sbor::rust::vec::Vec;
use sbor::*;

use crate::buffer::*;

use radix_engine_interface::data::*;

/// Encodes a data structure into a Scrypto buffer.
pub fn scrypto_encode_to_buffer<T: ScryptoEncode + ?Sized>(v: &T) -> Result<*mut u8, EncodeError> {
    let bytes = scrypto_encode(v)?;
    Ok(scrypto_alloc_initialized(bytes))
}

/// Decode a data structure from a Scrypto buffer.
pub fn scrypto_decode_from_buffer<T: ScryptoDecode>(ptr: *mut u8) -> Result<T, DecodeError> {
    scrypto_consume(ptr, |slice| scrypto_decode(slice))
}

/// Decode a data structure from a Scrypto buffer.
pub fn scrypto_buffer_to_vec(ptr: *mut u8) -> Vec<u8> {
    // TODO: Rather than to_vec(), just take ownership
    scrypto_consume(ptr, |slice| slice.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbor::rust::string::String;

    #[test]
    fn test_encode_for_radix_engine() {
        let encoded = scrypto_encode_to_buffer("abc").unwrap();
        let decoded: String = scrypto_decode_from_buffer(encoded).unwrap();
        assert_eq!(decoded, "abc");
    }
}
