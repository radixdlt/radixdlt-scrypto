use crate::rust::boxed::Box;
use crate::rust::collections::*;
use crate::rust::string::String;
use crate::rust::vec::Vec;
use crate::type_id::*;

/// A data structure that can be serialized into a byte array using SBOR.
pub trait Encode: TypeId {
    #[inline]
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_type(Self::type_id());
        self.encode_value(encoder);
    }

    fn encode_value(&self, encoder: &mut Encoder);
}

/// An `Encoder` abstracts the logic for writing core types into a byte buffer.
pub struct Encoder {
    buf: Vec<u8>,
    with_type: bool,
}

impl Encoder {
    pub fn new(buf: Vec<u8>, with_type: bool) -> Self {
        Self { buf, with_type }
    }

    pub fn with_type(buf: Vec<u8>) -> Self {
        Self::new(buf, true)
    }

    pub fn no_type(buf: Vec<u8>) -> Self {
        Self::new(buf, false)
    }

    pub fn write_type(&mut self, ty: u8) {
        if self.with_type {
            self.buf.push(ty);
        }
    }

    pub fn write_len(&mut self, len: usize) {
        self.buf.extend(&(len as u32).to_le_bytes());
    }

    pub fn write_u8(&mut self, n: u8) {
        self.buf.push(n);
    }

    pub fn write_slice(&mut self, slice: &[u8]) {
        self.buf.extend(slice);
    }
}

impl From<Encoder> for Vec<u8> {
    fn from(a: Encoder) -> Vec<u8> {
        a.buf
    }
}

impl Encode for () {
    fn encode_value(&self, _encoder: &mut Encoder) {}
}

impl Encode for bool {
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_u8(if *self { 1u8 } else { 0u8 })
    }
}

impl Encode for i8 {
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_u8(*self as u8);
    }
}

impl Encode for u8 {
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_u8(*self);
    }
}

macro_rules! encode_int {
    ($type:ident, $type_id:ident) => {
        impl Encode for $type {
            fn encode_value(&self, encoder: &mut Encoder) {
                encoder.write_slice(&(*self).to_le_bytes());
            }
        }
    };
}

encode_int!(i16, TYPE_I16);
encode_int!(i32, TYPE_I32);
encode_int!(i64, TYPE_I64);
encode_int!(i128, TYPE_I128);
encode_int!(u16, TYPE_U16);
encode_int!(u32, TYPE_U32);
encode_int!(u64, TYPE_U64);
encode_int!(u128, TYPE_U128);

impl Encode for isize {
    fn encode_value(&self, encoder: &mut Encoder) {
        (*self as i32).encode_value(encoder);
    }
}

impl Encode for usize {
    fn encode_value(&self, encoder: &mut Encoder) {
        (*self as u32).encode_value(encoder);
    }
}

impl Encode for str {
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_len(self.len());
        encoder.write_slice(self.as_bytes());
    }
}

impl Encode for &str {
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_len(self.len());
        encoder.write_slice(self.as_bytes());
    }
}

impl Encode for String {
    fn encode_value(&self, encoder: &mut Encoder) {
        self.as_str().encode_value(encoder);
    }
}

impl<T: Encode> Encode for Option<T> {
    fn encode_value(&self, encoder: &mut Encoder) {
        match self {
            None => {
                encoder.write_u8(0);
            }
            Some(v) => {
                encoder.write_u8(1);
                v.encode(encoder);
            }
        }
    }
}

impl<T: Encode> Encode for Box<T> {
    fn encode_value(&self, encoder: &mut Encoder) {
        self.as_ref().encode(encoder);
    }
}

impl<T: Encode, const N: usize> Encode for [T; N] {
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_type(T::type_id());
        encoder.write_len(self.len());
        for v in self {
            v.encode_value(encoder);
        }
    }
}

macro_rules! encode_tuple {
    ($n:tt $($idx:tt $name:ident)+) => {
        impl<$($name: Encode),+> Encode for ($($name,)+) {
            fn encode_value(&self, encoder: &mut Encoder) {
                encoder.write_len($n);

                $(self.$idx.encode(encoder);)+
            }
        }
    };
}

encode_tuple! { 2 0 A 1 B }
encode_tuple! { 3 0 A 1 B 2 C }
encode_tuple! { 4 0 A 1 B 2 C 3 D }
encode_tuple! { 5 0 A 1 B 2 C 3 D 4 E }
encode_tuple! { 6 0 A 1 B 2 C 3 D 4 E 5 F }
encode_tuple! { 7 0 A 1 B 2 C 3 D 4 E 5 F 6 G }
encode_tuple! { 8 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H }
encode_tuple! { 9 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I }
encode_tuple! { 10 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J }

impl<T: Encode> Encode for Vec<T> {
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_type(T::type_id());
        encoder.write_len(self.len());
        for v in self {
            v.encode_value(encoder);
        }
    }
}

impl<T: Encode> Encode for BTreeSet<T> {
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_type(T::type_id());
        encoder.write_len(self.len());
        for v in self {
            v.encode_value(encoder);
        }
    }
}

impl<K: Encode, V: Encode> Encode for BTreeMap<K, V> {
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_type(K::type_id());
        encoder.write_type(V::type_id());
        encoder.write_len(self.len());
        for (k, v) in self {
            k.encode_value(encoder);
            v.encode_value(encoder);
        }
    }
}

impl<T: Encode> Encode for HashSet<T> {
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_type(T::type_id());
        encoder.write_len(self.len());
        for v in self {
            v.encode_value(encoder);
        }
    }
}

impl<K: Encode, V: Encode> Encode for HashMap<K, V> {
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_type(K::type_id());
        encoder.write_type(V::type_id());
        encoder.write_len(self.len());
        for (k, v) in self {
            k.encode_value(encoder);
            v.encode_value(encoder);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::rust::boxed::Box;
    use crate::rust::collections::*;
    use crate::rust::vec;
    use crate::rust::vec::Vec;

    use super::{Encode, Encoder};

    fn do_encoding(enc: &mut Encoder) {
        ().encode(enc);
        true.encode(enc);
        1i8.encode(enc);
        1i16.encode(enc);
        1i32.encode(enc);
        1i64.encode(enc);
        1i128.encode(enc);
        1u8.encode(enc);
        1u16.encode(enc);
        1u32.encode(enc);
        1u64.encode(enc);
        1u128.encode(enc);
        "hello".encode(enc);

        Some(1u32).encode(enc);
        Box::new(1u32).encode(enc);
        [1u32, 2u32, 3u32].encode(enc);
        (1u32, 2u32).encode(enc);

        vec![1u32, 2u32, 3u32].encode(enc);
        let mut set = BTreeSet::<u8>::new();
        set.insert(1);
        set.insert(2);
        set.encode(enc);
        let mut map = BTreeMap::<u8, u8>::new();
        map.insert(1, 2);
        map.insert(3, 4);
        map.encode(enc);
    }

    #[test]
    pub fn test_encoding() {
        let mut enc = Encoder::with_type(Vec::with_capacity(512));
        do_encoding(&mut enc);

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
                12, 5, 0, 0, 0, 104, 101, 108, 108, 111, // string
                16, 1, 9, 1, 0, 0, 0, // option
                17, 9, 1, 0, 0, 0, // box
                18, 9, 3, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, // array
                19, 2, 0, 0, 0, 9, 1, 0, 0, 0, 9, 2, 0, 0, 0, // tuple
                32, 9, 3, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, // vec
                33, 7, 2, 0, 0, 0, 1, 2, // set
                34, 7, 7, 2, 0, 0, 0, 1, 2, 3, 4 // map
            ],
            bytes
        );
    }

    #[test]
    pub fn test_encoding_no_type() {
        let mut enc = Encoder::no_type(Vec::with_capacity(512));
        do_encoding(&mut enc);

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
                5, 0, 0, 0, 104, 101, 108, 108, 111, // string
                1, 1, 0, 0, 0, // option
                1, 0, 0, 0, // box
                3, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, // array
                2, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, // tuple
                3, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, // vec
                2, 0, 0, 0, 1, 2, // set
                2, 0, 0, 0, 1, 2, 3, 4 // map
            ],
            bytes
        );
    }
}
