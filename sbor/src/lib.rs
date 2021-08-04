#![cfg_attr(not(feature = "std"), no_std)]

mod decode;
mod describe;
mod encode;
mod model;

extern crate alloc;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

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
    let v = T::decode(&mut dec);
    dec.check_end()?;
    v
}

pub fn sbor_encode_no_metadata<T: Encode>(v: &T) -> Vec<u8> {
    let mut enc = Encoder::no_metadata();
    v.encode(&mut enc);
    enc.into()
}

pub fn sbor_decode_no_metadata<'de, T: Decode>(buf: &'de [u8]) -> Result<T, DecodeError> {
    let mut dec = Decoder::no_metadata(buf);
    let v = T::decode(&mut dec);
    dec.check_end()?;
    v
}

extern crate sbor_derive;
pub use sbor_derive::*;

// This is to make derives work within this crate.
// See: https://users.rust-lang.org/t/how-can-i-use-my-derive-macro-from-the-crate-that-declares-the-trait/60502
extern crate self as sbor;
