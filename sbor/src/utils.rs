use crate::rust::vec::Vec;
use crate::{Decode, DecodeError, Decoder, Encode, Encoder};

/// Encode a `T` into byte array, with type info included.
pub fn encode_with_static_info<T: Encode + ?Sized>(v: &T) -> Vec<u8> {
    let mut buf = Vec::with_capacity(512);
    let mut enc = Encoder::with_static_info(&mut buf);
    v.encode(&mut enc);
    buf
}

/// Encode a `T` into byte array, with no type info.
pub fn encode_no_static_info<T: Encode + ?Sized>(v: &T) -> Vec<u8> {
    let mut buf = Vec::with_capacity(512);
    let mut enc = Encoder::no_static_info(&mut buf);
    v.encode(&mut enc);
    buf
}

/// Decode an instance of `T` from a slice, with type info included.
pub fn decode_with_static_info<T: Decode>(buf: &[u8]) -> Result<T, DecodeError> {
    let mut dec = Decoder::with_static_info(buf);
    let v = T::decode(&mut dec)?;
    dec.check_end()?;
    Ok(v)
}

/// Decode an instance of `T` from a slice, with no type info.
pub fn decode_no_static_info<T: Decode>(buf: &[u8]) -> Result<T, DecodeError> {
    let mut dec = Decoder::no_static_info(buf);
    let v = T::decode(&mut dec)?;
    dec.check_end()?;
    Ok(v)
}
