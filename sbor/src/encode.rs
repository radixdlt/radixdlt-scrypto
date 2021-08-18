#[cfg(any(feature = "scrypto_std", feature = "scrypto_alloc"))]
use scrypto_types::{Address, BID, H256, RID, U256};

use crate::constants::*;
use crate::rust::boxed::Box;
use crate::rust::collections::*;
use crate::rust::string::String;
use crate::rust::vec::Vec;

/// A data structure that can be serialized into a byte array using SBOR.
pub trait Encode {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.write_type(Self::sbor_type());
        self.encode_value(encoder);
    }

    fn encode_value(&self, encoder: &mut Encoder);

    fn sbor_type() -> u8;
}

/// An `Encoder` abstracts the logic for writing core types into a byte buffer.
pub struct Encoder {
    buf: Vec<u8>,
    with_metadata: bool,
}

impl Encoder {
    pub fn new(capacity: usize, with_metadata: bool) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
            with_metadata,
        }
    }

    pub fn with_metadata() -> Self {
        Self::new(256, true)
    }

    pub fn no_metadata() -> Self {
        Self::new(256, false)
    }

    #[inline]
    pub fn write_type(&mut self, ty: u8) {
        if self.with_metadata {
            self.buf.push(ty);
        }
    }

    #[inline]
    pub fn write_name(&mut self, value: &str) {
        if self.with_metadata {
            self.write_len(value.len());
            self.buf.extend(value.as_bytes());
        }
    }

    #[inline]
    pub fn write_len(&mut self, len: usize) {
        self.buf.extend(&(len as u32).to_le_bytes());
    }

    #[inline]
    pub fn write_index(&mut self, len: usize) {
        self.buf.push(len as u8);
    }

    #[inline]
    pub fn write_u8(&mut self, len: u8) {
        self.buf.push(len as u8);
    }

    #[inline]
    pub fn write_slice(&mut self, slice: &[u8]) {
        self.buf.extend(slice);
    }
}

impl Into<Vec<u8>> for Encoder {
    fn into(self) -> Vec<u8> {
        self.buf
    }
}

// Implementation for basic types:
// - We keep one flat implementation per type, i.e., the `encode()` function;
// - Everything else is inlined.

impl Encode for () {
    #[inline]
    fn encode_value(&self, _encoder: &mut Encoder) {}

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_UNIT
    }
}

impl Encode for bool {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_u8(if *self { 1u8 } else { 0u8 })
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_BOOL
    }
}

impl Encode for i8 {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_u8(*self as u8);
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_I8
    }
}

impl Encode for u8 {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_u8(*self);
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_U8
    }
}

macro_rules! encode_int {
    ($type:ident, $sbor_type:ident) => {
        impl Encode for $type {
            #[inline]
            fn encode_value(&self, encoder: &mut Encoder) {
                encoder.write_slice(&(*self).to_le_bytes());
            }

            #[inline]
            fn sbor_type() -> u8 {
                $sbor_type
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
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        (*self as i32).encode_value(encoder);
    }

    #[inline]
    fn sbor_type() -> u8 {
        i32::sbor_type()
    }
}

impl Encode for usize {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        (*self as u32).encode_value(encoder);
    }

    #[inline]
    fn sbor_type() -> u8 {
        u32::sbor_type()
    }
}

impl Encode for str {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_len(self.len());
        encoder.write_slice(self.as_bytes());
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_STRING
    }
}

impl Encode for String {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        self.as_str().encode_value(encoder);
    }

    #[inline]
    fn sbor_type() -> u8 {
        str::sbor_type()
    }
}

impl<T: Encode> Encode for Option<T> {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
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

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_OPTION
    }
}

impl<T: Encode> Encode for Box<T> {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        self.as_ref().encode(encoder);
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_BOX
    }
}

impl<T: Encode, const N: usize> Encode for [T; N] {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_type(T::sbor_type());
        encoder.write_len(self.len());
        for v in self {
            v.encode_value(encoder);
        }
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_ARRAY
    }
}

macro_rules! encode_tuple {
    ($n:tt $($idx:tt $name:ident)+) => {
        impl<$($name: Encode),+> Encode for ($($name,)+) {
            #[inline]
            fn encode_value(&self, encoder: &mut Encoder) {
                encoder.write_len($n);

                $(self.$idx.encode(encoder);)+
            }

            #[inline]
            fn sbor_type() -> u8 {
                TYPE_TUPLE
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
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_type(T::sbor_type());
        encoder.write_len(self.len());
        for v in self {
            v.encode_value(encoder);
        }
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_VEC
    }
}

impl<T: Encode> Encode for BTreeSet<T> {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_type(T::sbor_type());
        encoder.write_len(self.len());
        for v in self {
            v.encode_value(encoder);
        }
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_TREE_SET
    }
}

impl<K: Encode, V: Encode> Encode for BTreeMap<K, V> {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_len(self.len());
        encoder.write_type(K::sbor_type());
        encoder.write_type(V::sbor_type());

        for (k, v) in self {
            k.encode_value(encoder);
            v.encode_value(encoder);
        }
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_TREE_MAP
    }
}

impl<T: Encode> Encode for HashSet<T> {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_type(T::sbor_type());
        encoder.write_len(self.len());
        for v in self {
            v.encode_value(encoder);
        }
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_HASH_SET
    }
}

impl<K: Encode, V: Encode> Encode for HashMap<K, V> {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_len(self.len());
        encoder.write_type(K::sbor_type());
        encoder.write_type(V::sbor_type());

        for (k, v) in self {
            k.encode_value(encoder);
            v.encode_value(encoder);
        }
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_HASH_MAP
    }
}

#[cfg(any(feature = "scrypto_std", feature = "scrypto_alloc"))]
impl Encode for H256 {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        let slice = self.as_ref();
        encoder.write_slice(slice);
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_H256
    }
}

#[cfg(any(feature = "scrypto_std", feature = "scrypto_alloc"))]
impl Encode for U256 {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        let mut bytes = [0u8; 32];
        self.to_little_endian(&mut bytes);
        encoder.write_slice(&bytes);
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_U256
    }
}
#[cfg(any(feature = "scrypto_std", feature = "scrypto_alloc"))]
impl Encode for Address {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        let bytes: Vec<u8> = self.clone().into();
        encoder.write_len(bytes.len());
        encoder.write_slice(&bytes);
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_ADDRESS
    }
}

#[cfg(any(feature = "scrypto_std", feature = "scrypto_alloc"))]
impl Encode for BID {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        let bytes: Vec<u8> = self.clone().into();
        encoder.write_len(bytes.len());
        encoder.write_slice(&bytes);
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_BID
    }
}

#[cfg(any(feature = "scrypto_std", feature = "scrypto_alloc"))]
impl Encode for RID {
    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        let bytes: Vec<u8> = self.clone().into();
        encoder.write_len(bytes.len());
        encoder.write_slice(&bytes);
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_RID
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
        let mut enc = Encoder::with_metadata();
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
                34, 2, 0, 0, 0, 7, 7, 1, 2, 3, 4 // map
            ],
            bytes
        );
    }

    #[test]
    pub fn test_encoding_no_metadata() {
        let mut enc = Encoder::no_metadata();
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
