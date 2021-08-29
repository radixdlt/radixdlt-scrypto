use crate::constants::*;
use crate::rust::boxed::Box;
use crate::rust::collections::*;
use crate::rust::hash::Hash;
use crate::rust::mem::MaybeUninit;
use crate::rust::ptr::copy;
use crate::rust::string::String;
use crate::rust::string::ToString;
use crate::rust::vec::Vec;

/// Represents an error ocurred during decoding.
#[derive(Debug, Clone)]
pub enum DecodeError {
    Underflow { required: usize, remaining: usize },

    InvalidType { expected: u8, actual: u8 },

    InvalidName { expected: String, actual: String },

    InvalidLength { expected: usize, actual: usize },

    InvalidIndex(u8),

    InvalidBool(u8),

    InvalidUtf8,

    NotAllBytesUsed(usize),

    InvalidCustomData(u8),
}

/// A data structure that can be decoded from a byte array using SBOR.
pub trait Decode: Sized {
    fn decode<'de>(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
        decoder.check_type(Self::sbor_type())?;
        Self::decode_value(decoder)
    }

    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError>;

    fn sbor_type() -> u8;
}

/// An `Decoder` abstracts the logic for reading and checking core types from a slice.
pub struct Decoder<'de> {
    input: &'de [u8],
    offset: usize,
    with_metadata: bool,
}

impl<'de> Decoder<'de> {
    pub fn new(input: &'de [u8], with_metadata: bool) -> Self {
        Self {
            input,
            offset: 0,
            with_metadata,
        }
    }

    pub fn with_metadata(input: &'de [u8]) -> Self {
        Self::new(input, true)
    }

    pub fn no_metadata(input: &'de [u8]) -> Self {
        Self::new(input, false)
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.input.len() - self.offset
    }

    #[inline]
    pub fn require(&self, n: usize) -> Result<(), DecodeError> {
        if self.remaining() < n {
            Err(DecodeError::Underflow {
                required: n,
                remaining: self.remaining(),
            })
        } else {
            Ok(())
        }
    }

    #[inline]
    pub fn read_u8(&mut self) -> Result<u8, DecodeError> {
        self.require(1)?;
        let result = self.input[self.offset];
        self.offset += 1;
        Ok(result)
    }

    #[inline]
    pub fn read_bytes(&mut self, n: usize) -> Result<&'de [u8], DecodeError> {
        self.require(n)?;
        let slice = &self.input[self.offset..self.offset + n];
        self.offset += n;
        Ok(slice)
    }

    #[inline]
    pub fn read_type(&mut self) -> Result<u8, DecodeError> {
        self.read_u8()
    }

    #[inline]
    pub fn read_len(&mut self) -> Result<usize, DecodeError> {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.read_bytes(4)?[..]);
        Ok(u32::from_le_bytes(bytes) as usize)
    }

    #[inline]
    pub fn read_name(&mut self) -> Result<String, DecodeError> {
        let len = self.read_len()?;
        let slice = self.read_bytes(len)?;
        let name = String::from_utf8(slice.to_vec()).map_err(|_| DecodeError::InvalidUtf8)?;
        Ok(name)
    }

    #[inline]
    pub fn read_index(&mut self) -> Result<u8, DecodeError> {
        self.read_u8()
    }

    #[inline]
    pub fn check_type(&mut self, expected: u8) -> Result<(), DecodeError> {
        if self.with_metadata {
            let ty = self.read_type()?;
            if ty != expected {
                return Err(DecodeError::InvalidType {
                    expected,
                    actual: ty,
                });
            }
        }

        Ok(())
    }

    #[inline]
    pub fn check_name(&mut self, expected: &str) -> Result<(), DecodeError> {
        if self.with_metadata {
            let name = self.read_name()?;
            if name.as_str() != expected {
                return Err(DecodeError::InvalidName {
                    expected: expected.to_string(),
                    actual: name,
                });
            }
        }

        Ok(())
    }

    #[inline]
    pub fn check_len(&mut self, expected: usize) -> Result<(), DecodeError> {
        let len = self.read_len()?;
        if len != expected {
            return Err(DecodeError::InvalidLength {
                expected,
                actual: len,
            });
        }

        Ok(())
    }

    #[inline]
    pub fn check_end(&self) -> Result<(), DecodeError> {
        let n = self.remaining();
        if n != 0 {
            Err(DecodeError::NotAllBytesUsed(n))
        } else {
            Ok(())
        }
    }
}

// Implementation for basic types:
// - We keep one flat implementation per type, i.e., the `decode()` function;
// - Everything else is inlined.

impl Decode for () {
    #[inline]
    fn decode_value<'de>(_decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
        Ok(())
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_UNIT
    }
}

impl Decode for bool {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
        let value = decoder.read_u8()?;
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DecodeError::InvalidBool(value)),
        }
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_BOOL
    }
}

impl Decode for i8 {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
        let value = decoder.read_u8()?;
        Ok(value as i8)
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_I8
    }
}

impl Decode for u8 {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
        let value = decoder.read_u8()?;
        Ok(value)
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_U8
    }
}

macro_rules! decode_int {
    ($type:ident, $sbor_type:ident, $n:expr) => {
        impl Decode for $type {
            #[inline]
            fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
                let slice = decoder.read_bytes($n)?;
                let mut bytes = [0u8; $n];
                bytes.copy_from_slice(&slice[..]);
                Ok(<$type>::from_le_bytes(bytes))
            }

            #[inline]
            fn sbor_type() -> u8 {
                $sbor_type
            }
        }
    };
}

decode_int!(i16, TYPE_I16, 2);
decode_int!(i32, TYPE_I32, 4);
decode_int!(i64, TYPE_I64, 8);
decode_int!(i128, TYPE_I128, 16);
decode_int!(u16, TYPE_U16, 2);
decode_int!(u32, TYPE_U32, 4);
decode_int!(u64, TYPE_U64, 8);
decode_int!(u128, TYPE_U128, 16);

impl Decode for isize {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
        i32::decode_value(decoder).map(|i| i as isize)
    }

    #[inline]
    fn sbor_type() -> u8 {
        i32::sbor_type()
    }
}

impl Decode for usize {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
        u32::decode_value(decoder).map(|i| i as usize)
    }

    #[inline]
    fn sbor_type() -> u8 {
        u32::sbor_type()
    }
}

impl Decode for String {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
        let len = decoder.read_len()?;
        let slice = decoder.read_bytes(len)?;
        let s = String::from_utf8(slice.to_vec()).map_err(|_| DecodeError::InvalidUtf8);
        Ok(s?)
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_STRING
    }
}

impl<T: Decode> Decode for Option<T> {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
        let index = decoder.read_index()?;

        match index {
            0 => Ok(None),
            1 => Ok(Some(T::decode(decoder)?)),
            _ => Err(DecodeError::InvalidIndex(index)),
        }
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_OPTION
    }
}

impl<T: Decode> Decode for Box<T> {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
        let v = T::decode(decoder)?;
        Ok(Box::new(v))
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_BOX
    }
}

impl<T: Decode, const N: usize> Decode for [T; N] {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
        decoder.check_type(T::sbor_type())?;
        decoder.check_len(N)?;

        let mut x = MaybeUninit::<[T; N]>::uninit();
        let arr = unsafe { &mut *x.as_mut_ptr() };
        for i in 0..N {
            arr[i] = T::decode_value(decoder)?;
        }
        Ok(unsafe { x.assume_init() })
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_ARRAY
    }
}

macro_rules! decode_tuple {
    ($n:tt $($idx:tt $name:ident)+) => {
        impl<$($name: Decode),+> Decode for ($($name,)+) {
            #[inline]
            fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
                let len = decoder.read_len()?;

                if len != $n {
                    return Err(DecodeError::InvalidLength{expected: $n, actual: len });
                }

                Ok(($($name::decode(decoder)?),+))
            }

            #[inline]
            fn sbor_type() -> u8 {
                TYPE_TUPLE
            }
        }
    };
}

decode_tuple! { 2 0 A 1 B }
decode_tuple! { 3 0 A 1 B 2 C }
decode_tuple! { 4 0 A 1 B 2 C 3 D }
decode_tuple! { 5 0 A 1 B 2 C 3 D 4 E }
decode_tuple! { 6 0 A 1 B 2 C 3 D 4 E 5 F }
decode_tuple! { 7 0 A 1 B 2 C 3 D 4 E 5 F 6 G }
decode_tuple! { 8 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H }
decode_tuple! { 9 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I }
decode_tuple! { 10 0 A 1 B 2 C 3 D 4 E 5 F 6 G 7 H 8 I 9 J }

impl<T: Decode> Decode for Vec<T> {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
        decoder.check_type(T::sbor_type())?;
        let len = decoder.read_len()?;

        if Self::sbor_type() == TYPE_U8 {
            let slice = decoder.read_bytes(len)?; // length is checked here
            let mut result = Vec::<T>::with_capacity(len);
            unsafe {
                copy(slice.as_ptr(), result.as_mut_ptr() as *mut u8, slice.len());
                result.set_len(slice.len());
            }
            Ok(result)
        } else {
            let mut result = Vec::<T>::with_capacity(if len <= 1024 { len } else { 1024 });
            for _ in 0..len {
                result.push(T::decode_value(decoder)?);
            }
            Ok(result)
        }
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_VEC
    }
}

impl<T: Decode + Ord> Decode for BTreeSet<T> {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
        decoder.check_type(T::sbor_type())?;
        let len = decoder.read_len()?;

        let mut result = BTreeSet::new();
        for _ in 0..len {
            result.insert(T::decode_value(decoder)?);
        }
        Ok(result)
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_TREE_SET
    }
}

impl<K: Decode + Ord, V: Decode> Decode for BTreeMap<K, V> {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
        let len = decoder.read_len()?;
        decoder.check_type(K::sbor_type())?;
        decoder.check_type(V::sbor_type())?;

        let mut map = BTreeMap::new();
        for _ in 0..len {
            map.insert(K::decode_value(decoder)?, V::decode_value(decoder)?);
        }
        Ok(map)
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_TREE_MAP
    }
}

impl<T: Decode + Hash + Eq> Decode for HashSet<T> {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
        decoder.check_type(T::sbor_type())?;
        let len = decoder.read_len()?;

        let mut result = HashSet::new();
        for _ in 0..len {
            result.insert(T::decode_value(decoder)?);
        }
        Ok(result)
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_HASH_SET
    }
}

impl<K: Decode + Hash + Eq, V: Decode> Decode for HashMap<K, V> {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, DecodeError> {
        let len = decoder.read_len()?;
        decoder.check_type(K::sbor_type())?;
        decoder.check_type(V::sbor_type())?;

        let mut map = HashMap::new();
        for _ in 0..len {
            map.insert(K::decode_value(decoder)?, V::decode_value(decoder)?);
        }
        Ok(map)
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_HASH_MAP
    }
}

#[cfg(test)]
mod tests {
    use crate::rust::boxed::Box;
    use crate::rust::collections::*;
    use crate::rust::string::String;
    use crate::rust::vec;
    use crate::rust::vec::Vec;

    use super::{Decode, Decoder};

    fn assert_decoding(dec: &mut Decoder) {
        <()>::decode(dec).unwrap();
        assert_eq!(true, <bool>::decode(dec).unwrap());
        assert_eq!(1, <i8>::decode(dec).unwrap());
        assert_eq!(1, <i16>::decode(dec).unwrap());
        assert_eq!(1, <i32>::decode(dec).unwrap());
        assert_eq!(1, <i64>::decode(dec).unwrap());
        assert_eq!(1, <i128>::decode(dec).unwrap());
        assert_eq!(1, <u8>::decode(dec).unwrap());
        assert_eq!(1, <u16>::decode(dec).unwrap());
        assert_eq!(1, <u32>::decode(dec).unwrap());
        assert_eq!(1, <u64>::decode(dec).unwrap());
        assert_eq!(1, <u128>::decode(dec).unwrap());
        assert_eq!("hello", <String>::decode(dec).unwrap());

        assert_eq!(Some(1u32), <Option<u32>>::decode(dec).unwrap());
        assert_eq!(Box::new(1u32), <Box<u32>>::decode(dec).unwrap());
        assert_eq!([1u32, 2u32, 3u32], <[u32; 3]>::decode(dec).unwrap());
        assert_eq!((1u32, 2u32), <(u32, u32)>::decode(dec).unwrap());

        assert_eq!(vec![1u32, 2u32, 3u32], <Vec<u32>>::decode(dec).unwrap());
        let mut set = BTreeSet::<u8>::new();
        set.insert(1);
        set.insert(2);
        assert_eq!(set, <BTreeSet<u8>>::decode(dec).unwrap());
        let mut map = BTreeMap::<u8, u8>::new();
        map.insert(1, 2);
        map.insert(3, 4);
        assert_eq!(map, <BTreeMap<u8, u8>>::decode(dec).unwrap());
    }

    #[test]
    pub fn test_decoding() {
        let bytes = vec![
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
            34, 2, 0, 0, 0, 7, 7, 1, 2, 3, 4, // map
        ];
        let mut dec = Decoder::with_metadata(&bytes);
        assert_decoding(&mut dec);
    }

    #[test]
    pub fn test_decoding_no_metadata() {
        let bytes = vec![
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
            2, 0, 0, 0, 1, 2, 3, 4, // map
        ];
        let mut dec = Decoder::no_metadata(&bytes);
        assert_decoding(&mut dec);
    }
}
