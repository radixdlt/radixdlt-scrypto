mod decode;
mod describe;
mod encode;
mod model;

pub use decode::*;
pub use describe::*;
pub use encode::*;
pub use model::*;

pub fn sbor_encode<T: Encode>(v: &T) -> Vec<u8> {
    let mut enc = Encoder::with_metadata();
    v.encode(&mut enc);
    enc.into()
}

pub fn sbor_decode<'de, T: Decode>(buf: &'de [u8]) -> Result<T, DecodeError> {
    let mut dec = Decoder::with_metadata(buf);
    T::decode(&mut dec)
}

pub fn sbor_encode_no_metadata<T: Encode>(v: &T) -> Vec<u8> {
    let mut enc = Encoder::no_metadata();
    v.encode(&mut enc);
    enc.into()
}

pub fn sbor_decode_no_metadata<'de, T: Decode>(buf: &'de [u8]) -> Result<T, DecodeError> {
    let mut dec = Decoder::no_metadata(buf);
    T::decode(&mut dec)
}

// Re-export sbor derive.
#[cfg(feature = "derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate sbor_derive;
#[cfg(feature = "derive")]
pub use sbor_derive::*;
