mod decode;
mod describe;
mod encode;
mod model;

pub use decode::*;
pub use describe::*;
pub use encode::*;
pub use model::*;

pub fn sbor_encode<T: Encode>(v: &T) -> Vec<u8> {
    let mut enc = Encoder::new();
    v.encode(&mut enc);
    enc.into()
}

pub fn sbor_decode<'de, T: Decode>(buf: &'de [u8]) -> Result<T, String> {
    let mut dec = Decoder::new(buf);
    T::decode(&mut dec)
}

// Re-export sbor derive.
#[cfg(feature = "derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate sbor_derive;
#[cfg(feature = "derive")]
pub use sbor_derive::*;
