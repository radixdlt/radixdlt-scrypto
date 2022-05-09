use sbor::rust::ptr;
use sbor::rust::slice;
use sbor::rust::vec::Vec;
use sbor::*;

/// Encodes a data structure into byte array.
pub fn scrypto_encode<T: Encode + ?Sized>(v: &T) -> Vec<u8> {
    encode_with_type(v)
}

/// Decodes an instance of `T` from a slice.
pub fn scrypto_decode<T: Decode>(buf: &[u8]) -> Result<T, DecodeError> {
    decode_with_type(buf)
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

/// Decode a data structure from byte array with size at the front.
///
/// # Panics
///
/// If the input ptr is invalid
pub fn scrypto_decode_with_size_prefix<T: Decode + ?Sized>(
    input: *const u8,
) -> Result<T, DecodeError> {
    let mut temp = [0u8; 4];
    unsafe {
        ptr::copy(input, temp.as_mut_ptr(), 4);
    }
    let n = u32::from_le_bytes(temp) as usize;

    let slice = unsafe { slice::from_raw_parts(input.add(4), n) };
    scrypto_decode(slice)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbor::rust::vec;

    #[test]
    fn test_encode_for_radix_engine() {
        let encoded = scrypto_encode_with_size_prefix("abc");
        assert_eq!(vec![8, 0, 0, 0, 12, 3, 0, 0, 0, 97, 98, 99], encoded);
    }
}
