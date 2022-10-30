use crate::rust::vec::Vec;
use crate::{Decode, DecodeError, Decoder, Encode, Encoder};

/// Encode a `T` into byte array, with type info included.
pub fn encode<T: Encode + ?Sized>(v: &T) -> Vec<u8> {
    let mut buf = Vec::with_capacity(512);
    let mut enc = Encoder::new(&mut buf);
    v.encode(&mut enc);
    buf
}

/// Decode an instance of `T` from a slice, with type info included.
pub fn decode<T: Decode>(buf: &[u8]) -> Result<T, DecodeError> {
    let mut dec = Decoder::new(buf);
    let v = T::decode(&mut dec)?;
    dec.check_end()?;
    Ok(v)
}
