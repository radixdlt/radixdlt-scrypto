use crate::rust::boxed::Box;
use crate::rust::cell::RefCell;
use crate::rust::collections::*;
use crate::rust::hash::Hash;
use crate::rust::mem::MaybeUninit;
use crate::rust::ptr::copy;
use crate::rust::rc::Rc;
use crate::rust::string::String;
use crate::rust::string::ToString;
use crate::rust::vec::Vec;
use crate::type_id::*;
use sbor::*;

/// Represents an error ocurred during decoding.
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub enum DecodeError {
    Underflow { required: usize, remaining: usize },

    InvalidType { expected: Option<u8>, actual: u8 },

    InvalidName { expected: String, actual: String },

    InvalidLength { expected: usize, actual: usize },

    InvalidIndex(u8),

    InvalidEnumVariant(String),

    InvalidBool(u8),

    InvalidUtf8,

    NotAllBytesUsed(usize),

    CustomError(String),
}

/// A data structure that can be decoded from a byte array using SBOR.
pub trait Decode: Sized {
    fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Self::check_type_id(decoder)?;
        Self::decode_value(decoder)
    }

    fn check_type_id(decoder: &mut Decoder) -> Result<(), DecodeError>;

    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError>;
}

/// A `Decoder` abstracts the logic for decoding basic types.
pub struct Decoder<'de> {
    input: &'de [u8],
    offset: usize,
    with_static_info: bool,
}

impl<'de> Decoder<'de> {
    pub fn new(input: &'de [u8], with_static_info: bool) -> Self {
        Self {
            input,
            offset: 0,
            with_static_info,
        }
    }

    pub fn with_static_info(input: &'de [u8]) -> Self {
        Self::new(input, true)
    }

    pub fn no_static_info(input: &'de [u8]) -> Self {
        Self::new(input, false)
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.input.len() - self.offset
    }

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

    pub fn read_type(&mut self) -> Result<u8, DecodeError> {
        self.read_byte()
    }

    pub fn read_variant_index(&mut self) -> Result<u8, DecodeError> {
        self.read_byte()
    }

    pub fn read_variant_label(&mut self) -> Result<String, DecodeError> {
        let n = self.read_dynamic_size()?;
        let slice = self.read_bytes(n)?;
        String::from_utf8(slice.to_vec()).map_err(|_| DecodeError::InvalidUtf8)
    }

    pub fn read_static_size(&mut self) -> Result<usize, DecodeError> {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(self.read_bytes(4)?);
        Ok(u32::from_le_bytes(bytes) as usize)
    }

    pub fn read_dynamic_size(&mut self) -> Result<usize, DecodeError> {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(self.read_bytes(4)?);
        Ok(u32::from_le_bytes(bytes) as usize)
    }

    pub fn read_byte(&mut self) -> Result<u8, DecodeError> {
        self.require(1)?;
        let result = self.input[self.offset];
        self.offset += 1;
        Ok(result)
    }

    pub fn read_bytes(&mut self, n: usize) -> Result<&'de [u8], DecodeError> {
        self.require(n)?;
        let slice = &self.input[self.offset..self.offset + n];
        self.offset += n;
        Ok(slice)
    }

    pub fn check_type_id(&mut self, expected: u8) -> Result<(), DecodeError> {
        if self.with_static_info {
            let ty = self.read_type()?;
            if ty != expected {
                return Err(DecodeError::InvalidType {
                    expected: Some(expected),
                    actual: ty,
                });
            }
        }

        Ok(())
    }

    pub fn check_static_size(&mut self, expected: usize) -> Result<(), DecodeError> {
        if self.with_static_info {
            let len = self.read_dynamic_size()?;
            if len != expected {
                return Err(DecodeError::InvalidLength {
                    expected,
                    actual: len,
                });
            }
        }

        Ok(())
    }

    pub fn check_end(&self) -> Result<(), DecodeError> {
        let n = self.remaining();
        if n != 0 {
            Err(DecodeError::NotAllBytesUsed(n))
        } else {
            Ok(())
        }
    }
}

impl Decode for () {
    #[inline]
    fn check_type_id(decoder: &mut Decoder) -> Result<(), DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(_decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Ok(())
    }
}

impl Decode for bool {
    #[inline]
    fn check_type_id(decoder: &mut Decoder) -> Result<(), DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let value = decoder.read_byte()?;
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DecodeError::InvalidBool(value)),
        }
    }
}

impl Decode for i8 {
    #[inline]
    fn check_type_id(decoder: &mut Decoder) -> Result<(), DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let value = decoder.read_byte()?;
        Ok(value as i8)
    }
}

impl Decode for u8 {
    #[inline]
    fn check_type_id(decoder: &mut Decoder) -> Result<(), DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let value = decoder.read_byte()?;
        Ok(value)
    }
}

macro_rules! decode_int {
    ($type:ident, $type_id:ident, $n:expr) => {
        impl Decode for $type {
            #[inline]
            fn check_type_id(decoder: &mut Decoder) -> Result<(), DecodeError> {
                decoder.check_type_id(Self::type_id())
            }
            fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
                let slice = decoder.read_bytes($n)?;
                let mut bytes = [0u8; $n];
                bytes.copy_from_slice(&slice[..]);
                Ok(<$type>::from_le_bytes(bytes))
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
    fn check_type_id(decoder: &mut Decoder) -> Result<(), DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        i64::decode_value(decoder).map(|i| i as isize)
    }
}

impl Decode for usize {
    #[inline]
    fn check_type_id(decoder: &mut Decoder) -> Result<(), DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        u64::decode_value(decoder).map(|i| i as usize)
    }
}

impl Decode for String {
    #[inline]
    fn check_type_id(decoder: &mut Decoder) -> Result<(), DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let len = decoder.read_dynamic_size()?;
        let slice = decoder.read_bytes(len)?;
        String::from_utf8(slice.to_vec()).map_err(|_| DecodeError::InvalidUtf8)
    }
}

impl<T: Decode> Decode for Option<T> {
    #[inline]
    fn check_type_id(decoder: &mut Decoder) -> Result<(), DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let index = decoder.read_variant_index()?;

        match index {
            OPTION_VARIANT_SOME => Ok(Some(T::decode(decoder)?)),
            OPTION_VARIANT_NONE => Ok(None),
            _ => Err(DecodeError::InvalidIndex(index)),
        }
    }
}

impl<T: Decode + TypeId> Decode for Box<T> {
    #[inline]
    fn check_type_id(decoder: &mut Decoder) -> Result<(), DecodeError> {
        decoder.check_type_id(T::type_id())
    }
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let v = T::decode_value(decoder)?;
        Ok(Box::new(v))
    }
}

impl<T: Decode + TypeId> Decode for Rc<T> {
    #[inline]
    fn check_type_id(decoder: &mut Decoder) -> Result<(), DecodeError> {
        decoder.check_type_id(T::type_id())
    }
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let v = T::decode_value(decoder)?;
        Ok(Rc::new(v))
    }
}

impl<T: Decode + TypeId> Decode for RefCell<T> {
    #[inline]
    fn check_type_id(decoder: &mut Decoder) -> Result<(), DecodeError> {
        decoder.check_type_id(T::type_id())
    }
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let v = T::decode_value(decoder)?;
        Ok(RefCell::new(v))
    }
}

impl<T: Decode + TypeId, const N: usize> Decode for [T; N] {
    #[inline]
    fn check_type_id(decoder: &mut Decoder) -> Result<(), DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        decoder.check_type_id(T::type_id())?;
        decoder.check_static_size(N)?;

        // Please read:
        // * https://doc.rust-lang.org/stable/std/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
        // * https://github.com/rust-lang/rust/issues/61956
        //
        // TODO: replace with `uninit_array` and `assume_array_init` once they're stable

        // Create an uninitialized array
        let mut data: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };

        // Decode element by element
        for elem in &mut data[..] {
            elem.write(T::decode_value(decoder)?);
        }

        // Use &mut as an assertion of unique "ownership"
        let ptr = &mut data as *mut _ as *mut [T; N];
        let res = unsafe { ptr.read() };
        core::mem::forget(data);

        Ok(res)
    }
}

macro_rules! decode_tuple {
    ($n:tt $($idx:tt $name:ident)+) => {
        impl<$($name: Decode),+> Decode for ($($name,)+) {
            #[inline]
            fn check_type_id(decoder: &mut Decoder) -> Result<(), DecodeError> {
                decoder.check_type_id(Self::type_id())
            }
            fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
                decoder.check_static_size($n)?;

                Ok(($($name::decode(decoder)?),+))
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

impl<T: Decode + TypeId, E: Decode + TypeId> Decode for Result<T, E> {
    #[inline]
    fn check_type_id(decoder: &mut Decoder) -> Result<(), DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let index = decoder.read_variant_index()?;
        match index {
            RESULT_VARIANT_OK => Ok(Ok(T::decode(decoder)?)),
            RESULT_VARIANT_ERR => Ok(Err(E::decode(decoder)?)),
            _ => Err(DecodeError::InvalidIndex(index)),
        }
    }
}

impl<T: Decode + TypeId> Decode for Vec<T> {
    #[inline]
    fn check_type_id(decoder: &mut Decoder) -> Result<(), DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        decoder.check_type_id(T::type_id())?;
        let len = decoder.read_dynamic_size()?;

        if T::type_id() == TYPE_U8 || T::type_id() == TYPE_I8 {
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
}

impl<T: Decode + TypeId + Ord> Decode for BTreeSet<T> {
    #[inline]
    fn check_type_id(decoder: &mut Decoder) -> Result<(), DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        decoder.check_type_id(T::type_id())?;
        let len = decoder.read_dynamic_size()?;

        let mut result = BTreeSet::new();
        for _ in 0..len {
            if !result.insert(T::decode_value(decoder)?) {
                // This is a custom error because key duplication logic is defined by the application
                return Err(DecodeError::CustomError(
                    "Duplicate BTreeSet entries".to_string(),
                ));
            }
        }
        Ok(result)
    }
}

impl<K: Decode + TypeId + Ord, V: Decode + TypeId> Decode for BTreeMap<K, V> {
    #[inline]
    fn check_type_id(decoder: &mut Decoder) -> Result<(), DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        decoder.check_type_id(K::type_id())?;
        decoder.check_type_id(V::type_id())?;
        let len = decoder.read_dynamic_size()?;
        let mut map = BTreeMap::new();
        for _ in 0..len {
            if map
                .insert(K::decode_value(decoder)?, V::decode_value(decoder)?)
                .is_some()
            {
                // This is a custom error because key duplication logic is defined by the application
                return Err(DecodeError::CustomError(
                    "Duplicate BTreeMap entries".to_string(),
                ));
            }
        }
        Ok(map)
    }
}

impl<T: Decode + TypeId + Hash + Eq> Decode for HashSet<T> {
    #[inline]
    fn check_type_id(decoder: &mut Decoder) -> Result<(), DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        decoder.check_type_id(T::type_id())?;
        let len = decoder.read_dynamic_size()?;

        let mut result = HashSet::new();
        for _ in 0..len {
            if !result.insert(T::decode_value(decoder)?) {
                // This is a custom error because key duplication logic is defined by the application
                return Err(DecodeError::CustomError(
                    "Duplicate HashSet entries".to_string(),
                ));
            }
        }
        Ok(result)
    }
}

impl<K: Decode + TypeId + Hash + Eq, V: Decode + TypeId> Decode for HashMap<K, V> {
    #[inline]
    fn check_type_id(decoder: &mut Decoder) -> Result<(), DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        decoder.check_type_id(K::type_id())?;
        decoder.check_type_id(V::type_id())?;
        let len = decoder.read_dynamic_size()?;
        let mut map = HashMap::new();
        for _ in 0..len {
            if map
                .insert(K::decode_value(decoder)?, V::decode_value(decoder)?)
                .is_some()
            {
                // This is a custom error because key duplication logic is defined by the application
                return Err(DecodeError::CustomError(
                    "Duplicate HashMap entries".to_string(),
                ));
            }
        }
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::Encode;
    use crate::encode::Encoder;
    use crate::rust::borrow::ToOwned;
    use crate::rust::vec;

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
        assert_eq!(None, <Option<u32>>::decode(dec).unwrap());
        assert_eq!(Ok(1u32), <Result<u32, String>>::decode(dec).unwrap());
        assert_eq!(
            Err("hello".to_owned()),
            <Result<u32, String>>::decode(dec).unwrap()
        );

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
            18, 0, 9, 1, 0, 0, 0, // option
            18, 1, // option
            19, 0, 9, 1, 0, 0, 0, // result
            19, 1, 12, 5, 0, 0, 0, 104, 101, 108, 108, 111, // result
            32, 9, 3, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, // array
            33, 2, 0, 0, 0, 9, 1, 0, 0, 0, 9, 2, 0, 0, 0, // tuple
            48, 9, 3, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, // list
            49, 7, 2, 0, 0, 0, 1, 2, // set
            50, 7, 7, 2, 0, 0, 0, 1, 2, 3, 4, // map
        ];
        let mut dec = Decoder::with_static_info(&bytes);
        assert_decoding(&mut dec);
    }

    #[test]
    pub fn test_decoding_no_static_info() {
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
            0, 1, 0, 0, 0, // option
            1, // option
            0, 1, 0, 0, 0, // result
            1, 5, 0, 0, 0, 104, 101, 108, 108, 111, // result
            1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, // array
            1, 0, 0, 0, 2, 0, 0, 0, // tuple
            3, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, // list
            2, 0, 0, 0, 1, 2, // set
            2, 0, 0, 0, 1, 2, 3, 4, // map
        ];
        let mut dec = Decoder::no_static_info(&bytes);
        assert_decoding(&mut dec);
    }

    #[test]
    pub fn test_decode_box() {
        let bytes = vec![7u8, 5u8];
        let mut dec = Decoder::with_static_info(&bytes);
        let x = <Box<u8>>::decode(&mut dec).unwrap();
        assert_eq!(Box::new(5u8), x);
    }

    #[test]
    pub fn test_decode_rc() {
        let bytes = vec![7u8, 5u8];
        let mut dec = Decoder::with_static_info(&bytes);
        let x = <Rc<u8>>::decode(&mut dec).unwrap();
        assert_eq!(Rc::new(5u8), x);
    }

    #[test]
    pub fn test_decode_ref_cell() {
        let bytes = vec![7u8, 5u8];
        let mut dec = Decoder::with_static_info(&bytes);
        let x = <RefCell<u8>>::decode(&mut dec).unwrap();
        assert_eq!(RefCell::new(5u8), x);
    }

    #[derive(sbor::TypeId, sbor::Encode, sbor::Decode, PartialEq, Eq, Debug)]
    struct NFA {
        a: [u8; 32],
        b: Vec<u8>,
    }

    #[test]
    pub fn test_generic_array() {
        let value1 = [
            NFA {
                a: [1u8; 32],
                b: vec![1],
            },
            NFA {
                a: [2u8; 32],
                b: vec![2],
            },
        ];

        // Encode
        let mut bytes = Vec::with_capacity(512);
        let mut enc = Encoder::with_static_info(&mut bytes);
        value1.encode(&mut enc);

        let mut dec = Decoder::with_static_info(&bytes);
        let value2 = <[NFA; 2]>::decode(&mut dec).unwrap();
        assert_eq!(value1, value2);
    }
}
