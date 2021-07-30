extern crate alloc;
use alloc::vec::Vec;

use crate::*;

pub trait Encode {
    fn encode(&self, encoder: &mut Encoder);
}

macro_rules! encode_basic_type {
    ($type:ident, $method:ident) => {
        impl Encode for $type {
            fn encode(&self, encoder: &mut Encoder) {
                encoder.$method(*self);
            }
        }
    };
}

encode_basic_type!(bool, encode_bool);
encode_basic_type!(i8, encode_i8);
encode_basic_type!(i16, encode_i16);
encode_basic_type!(i32, encode_i32);
encode_basic_type!(i64, encode_i64);
encode_basic_type!(i128, encode_i128);
encode_basic_type!(u8, encode_u8);
encode_basic_type!(u16, encode_u16);
encode_basic_type!(u32, encode_u32);
encode_basic_type!(u64, encode_u64);
encode_basic_type!(u128, encode_u128);

impl Encode for String {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.encode_string(self);
    }
}

impl<T: Encode> Encode for Option<T> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.encode_option(self);
    }
}

impl<T: Encode, const N: usize> Encode for [T; N] {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.encode_array(self);
    }
}

impl<T: Encode> Encode for Vec<T> {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.encode_vec(self);
    }
}

// TODO impl for tuples

pub struct Encoder {
    buf: Vec<u8>,
}

macro_rules! encode_int {
    ($method:ident, $sbor_type:expr, $native_type:ty) => {
        pub fn $method(&mut self, value: $native_type) {
            self.buf.push($sbor_type);
            self.buf.extend(&value.to_be_bytes());
        }
    };
}

impl Encoder {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn encode_unit(&mut self) {
        self.buf.push(TYPE_UNIT);
    }

    pub fn encode_bool(&mut self, value: bool) {
        self.buf.push(TYPE_BOOL);
        self.buf.push(if value { 1u8 } else { 0u8 });
    }

    encode_int!(encode_i8, TYPE_I8, i8);
    encode_int!(encode_i16, TYPE_I16, i16);
    encode_int!(encode_i32, TYPE_I32, i32);
    encode_int!(encode_i64, TYPE_I64, i64);
    encode_int!(encode_i128, TYPE_I128, i128);
    encode_int!(encode_u8, TYPE_U8, u8);
    encode_int!(encode_u16, TYPE_U16, u16);
    encode_int!(encode_u32, TYPE_U32, u32);
    encode_int!(encode_u64, TYPE_U64, u64);
    encode_int!(encode_u128, TYPE_U128, u128);

    pub fn encode_string(&mut self, value: &String) {
        self.buf.push(TYPE_STRING);
        self.buf.extend(&(value.len() as u16).to_be_bytes());
        self.buf.extend(value.as_bytes());
    }

    pub fn encode_option<T: Encode>(&mut self, value: &Option<T>) {
        self.buf.push(TYPE_OPTION);
        match value {
            Some(v) => {
                self.buf.push(1);
                v.encode(self);
            }
            None => {
                self.buf.push(0);
            }
        }
    }

    pub fn encode_array<T: Encode>(&mut self, value: &[T]) {
        self.buf.push(TYPE_ARRAY);
        self.buf.extend(&(value.len() as u16).to_be_bytes());
        for v in value {
            v.encode(self);
        }
    }

    pub fn encode_vec<T: Encode>(&mut self, value: &Vec<T>) {
        self.buf.push(TYPE_VEC);
        self.buf.extend(&(value.len() as u16).to_be_bytes());
        for v in value {
            v.encode(self);
        }
    }

    // TODO expand to different lengths
    pub fn encode_tuple<A: Encode, B: Encode>(&mut self, value: &(A, B)) {
        self.buf.push(TYPE_TUPLE);
        self.buf.extend(&2u16.to_be_bytes());

        value.0.encode(self);
        value.1.encode(self);
    }

    pub fn encode_struct<T: Encode>(&mut self, value: &T) {
        value.encode(self);
    }

    pub fn encode_enum<T: Encode>(&mut self, value: &T) {
        value.encode(self);
    }
}

impl Into<Vec<u8>> for Encoder {
    fn into(self) -> Vec<u8> {
        self.buf
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::string::ToString;

    use super::Encoder;

    #[test]
    pub fn test_encoding() {
        let mut enc = Encoder::new();
        enc.encode_unit();
        enc.encode_bool(true);
        enc.encode_i8(1);
        enc.encode_i16(1);
        enc.encode_i32(1);
        enc.encode_i64(1);
        enc.encode_i128(1);
        enc.encode_u8(1);
        enc.encode_u16(1);
        enc.encode_u32(1);
        enc.encode_u64(1);
        enc.encode_u128(1);
        enc.encode_string(&"hello".to_string());
        enc.encode_option(&Some(1u32));
        enc.encode_array(&[1u32, 2u32, 3u32]);
        enc.encode_vec(&vec![1u32, 2u32, 3u32]);
        enc.encode_tuple(&(1u32, 2u32));

        let bytes: Vec<u8> = enc.into();
        assert_eq!(
            vec![
                0, // unit
                1, 1, // bool
                2, 1, // i8
                3, 0, 1, // i16
                4, 0, 0, 0, 1, // i32
                5, 0, 0, 0, 0, 0, 0, 0, 1, // i64
                6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // i128
                7, 1, // u8
                8, 0, 1, // u16
                9, 0, 0, 0, 1, // u32
                10, 0, 0, 0, 0, 0, 0, 0, 1, // u64
                11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, // u128
                12, 0, 5, 104, 101, 108, 108, 111, // String
                13, 1, 9, 0, 0, 0, 1, // option
                14, 0, 3, 9, 0, 0, 0, 1, 9, 0, 0, 0, 2, 9, 0, 0, 0, 3, // array
                15, 0, 3, 9, 0, 0, 0, 1, 9, 0, 0, 0, 2, 9, 0, 0, 0, 3, // vector
                16, 0, 2, 9, 0, 0, 0, 1, 9, 0, 0, 0, 2 // tuple
            ],
            bytes
        );
    }
}
