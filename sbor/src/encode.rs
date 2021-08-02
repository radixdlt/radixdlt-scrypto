extern crate alloc;
use alloc::vec::Vec;

use crate::*;

pub trait Encode {
    fn encode(&self, encoder: &mut Encoder);
}

pub struct Encoder {
    buf: Vec<u8>,
    with_schema: bool,
}

impl Encoder {
    pub fn new() -> Self {
        Self {
            buf: Vec::with_capacity(256),
            with_schema: true,
        }
    }

    pub fn new_no_schema() -> Self {
        Self {
            buf: Vec::with_capacity(256),
            with_schema: false,
        }
    }

    #[inline(always)]
    pub fn write_type(&mut self, ty: u8) {
        if self.with_schema {
            self.buf.push(ty);
        }
    }

    #[inline(always)]
    pub fn write_name(&mut self, value: &str) {
        if self.with_schema {
            self.write_type(TYPE_STRING);
            self.write_len(value.len());
            self.buf.extend(value.as_bytes());
        }
    }

    #[inline(always)]
    pub fn write_len(&mut self, len: usize) {
        self.buf.extend(&(len as u16).to_le_bytes());
    }

    #[inline(always)]
    pub fn write_index(&mut self, len: usize) {
        self.buf.push(len as u8);
    }

    #[inline(always)]
    pub fn write_u8(&mut self, len: u8) {
        self.buf.push(len as u8);
    }

    #[inline(always)]
    pub fn write_slice(&mut self, slice: &[u8]) {
        self.buf.extend(slice);
    }
}

impl Into<Vec<u8>> for Encoder {
    fn into(self) -> Vec<u8> {
        self.buf
    }
}

// implementation for basic types

impl Encode for () {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_type(TYPE_UNIT);
    }
}

impl Encode for bool {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_type(TYPE_BOOL);
        encoder.write_u8(if *self { 1u8 } else { 0u8 })
    }
}

impl Encode for i8 {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_type(TYPE_I8);
        encoder.write_u8(*self as u8);
    }
}

impl Encode for u8 {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_type(TYPE_U8);
        encoder.write_u8(*self);
    }
}

macro_rules! encode_basic_type {
    ($type:ident, $sbor_type:ident) => {
        impl Encode for $type {
            fn encode(&self, encoder: &mut Encoder) {
                encoder.write_type($sbor_type);
                encoder.write_slice(&(*self).to_le_bytes());
            }
        }
    };
}

encode_basic_type!(i16, TYPE_I16);
encode_basic_type!(i32, TYPE_I32);
encode_basic_type!(i64, TYPE_I64);
encode_basic_type!(i128, TYPE_I128);
encode_basic_type!(u16, TYPE_U16);
encode_basic_type!(u32, TYPE_U32);
encode_basic_type!(u64, TYPE_U64);
encode_basic_type!(u128, TYPE_U128);

impl Encode for str {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_type(TYPE_STRING);
        encoder.write_len(self.len());
        encoder.write_slice(self.as_bytes());
    }
}

impl Encode for String {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_type(TYPE_STRING);
        encoder.write_len(self.len());
        encoder.write_slice(self.as_bytes());
    }
}

impl<T: Encode> Encode for Option<T> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_type(TYPE_OPTION);
        match self {
            None => {
                encoder.write_index(0);
            }
            Some(v) => {
                encoder.write_index(1);
                v.encode(encoder);
            }
        }
    }
}

impl<T: Encode, const N: usize> Encode for [T; N] {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_type(TYPE_ARRAY);
        encoder.write_len(self.len());
        for v in self {
            v.encode(encoder);
        }
    }
}

impl<T: Encode> Encode for Vec<T> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_type(TYPE_VEC);
        encoder.write_len(self.len());
        for v in self {
            v.encode(encoder);
        }
    }
}

// TODO expand to different lengths
impl<A: Encode, B: Encode> Encode for (A, B) {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_type(TYPE_TUPLE);
        encoder.write_len(2);

        self.0.encode(encoder);
        self.1.encode(encoder);
    }
}

#[cfg(test)]
mod tests {
    use super::{Encode, Encoder};

    #[test]
    pub fn test_encoding() {
        let mut enc = Encoder::new();
        ().encode(&mut enc);
        true.encode(&mut enc);
        1i8.encode(&mut enc);
        1i16.encode(&mut enc);
        1i32.encode(&mut enc);
        1i64.encode(&mut enc);
        1i128.encode(&mut enc);
        1u8.encode(&mut enc);
        1u16.encode(&mut enc);
        1u32.encode(&mut enc);
        1u64.encode(&mut enc);
        1u128.encode(&mut enc);
        "hello".encode(&mut enc);
        Some(1u32).encode(&mut enc);
        [1u32, 2u32, 3u32].encode(&mut enc);
        vec![1u32, 2u32, 3u32].encode(&mut enc);
        (1u32, 2u32).encode(&mut enc);

        let bytes: Vec<u8> = enc.into();
        assert_eq!(
            vec![
                0, // unit
                1, 1, // bool
                2, 1, // i8
                3, 1, 0, // i16
                4, 1, 0, 0, 0, // i32
                5, 1, 0, 0, 0, 0, 0, 0, 0, // i64
                6, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // i128
                7, 1, // u8
                8, 1, 0, // u16
                9, 1, 0, 0, 0, // u32
                10, 1, 0, 0, 0, 0, 0, 0, 0, // u64
                11, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // u128
                12, 5, 0, 104, 101, 108, 108, 111, // string
                13, 1, 9, 1, 0, 0, 0, // option
                14, 3, 0, 9, 1, 0, 0, 0, 9, 2, 0, 0, 0, 9, 3, 0, 0, 0, // array
                15, 3, 0, 9, 1, 0, 0, 0, 9, 2, 0, 0, 0, 9, 3, 0, 0, 0, // vector
                16, 2, 0, 9, 1, 0, 0, 0, 9, 2, 0, 0, 0 // tuple
            ],
            bytes
        );
    }

    #[test]
    pub fn test_encoding_no_schema() {
        let mut enc = Encoder::new_no_schema();
        ().encode(&mut enc);
        true.encode(&mut enc);
        1i8.encode(&mut enc);
        1i16.encode(&mut enc);
        1i32.encode(&mut enc);
        1i64.encode(&mut enc);
        1i128.encode(&mut enc);
        1u8.encode(&mut enc);
        1u16.encode(&mut enc);
        1u32.encode(&mut enc);
        1u64.encode(&mut enc);
        1u128.encode(&mut enc);
        "hello".encode(&mut enc);
        Some(1u32).encode(&mut enc);
        [1u32, 2u32, 3u32].encode(&mut enc);
        vec![1u32, 2u32, 3u32].encode(&mut enc);
        (1u32, 2u32).encode(&mut enc);

        let bytes: Vec<u8> = enc.into();
        assert_eq!(
            vec![
                // unit
                1, // bool
                1, // i8
                1, 0, // i16
                1, 0, 0, 0, // i32
                1, 0, 0, 0, 0, 0, 0, 0, // i64
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // i128
                1, // u8
                1, 0, // u16
                1, 0, 0, 0, // u32
                1, 0, 0, 0, 0, 0, 0, 0, // u64
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // u128
                5, 0, 104, 101, 108, 108, 111, // string
                1, 1, 0, 0, 0, // option
                3, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, // array
                3, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, // vector
                2, 0, 1, 0, 0, 0, 2, 0, 0, 0 // tuple
            ],
            bytes
        );
    }
}
