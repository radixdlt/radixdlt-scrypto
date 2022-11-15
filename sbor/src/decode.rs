use crate::constants::*;
use crate::rust::borrow::Cow;
use crate::rust::borrow::ToOwned;
use crate::rust::boxed::Box;
use crate::rust::cell::RefCell;
use crate::rust::collections::*;
use crate::rust::hash::Hash;
use crate::rust::marker::PhantomData;
use crate::rust::mem::MaybeUninit;
use crate::rust::ptr::copy;
use crate::rust::rc::Rc;
use crate::rust::string::String;
use crate::rust::vec::Vec;
use crate::type_id::*;
use crate::*;

/// Represents an error ocurred during decoding.
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub enum DecodeError {
    ExtraTrailingBytes(usize),

    BufferUnderflow { required: usize, remaining: usize },

    UnexpectedTypeId(u8),

    UnexpectedSize { expected: usize, actual: usize },

    UnknownTypeId(u8),

    UnknownDiscriminator(String),

    InvalidUnit(u8),

    InvalidBool(u8),

    InvalidUtf8,

    InvalidCustomValue, // TODO: generify custom error codes
}

/// A data structure that can be decoded from a byte array using SBOR.
pub trait Decode<X: CustomTypeId>: Sized {
    fn decode(decoder: &mut Decoder<X>) -> Result<Self, DecodeError> {
        let type_id = Self::decode_type_id(decoder)?;
        Self::decode_value(decoder, type_id)
    }

    fn decode_type_id(decoder: &mut Decoder<X>) -> Result<SborTypeId<X>, DecodeError>;

    fn decode_value(decoder: &mut Decoder<X>, type_id: SborTypeId<X>) -> Result<Self, DecodeError>;
}

/// A `Decoder` abstracts the logic for decoding basic types.
pub struct Decoder<'de, X: CustomTypeId> {
    input: &'de [u8],
    offset: usize,
    phantom: PhantomData<X>,
}

impl<'de, X: CustomTypeId> Decoder<'de, X> {
    pub fn new(input: &'de [u8]) -> Self {
        Self {
            input,
            offset: 0,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.input.len() - self.offset
    }

    pub fn require(&self, n: usize) -> Result<(), DecodeError> {
        if self.remaining() < n {
            Err(DecodeError::BufferUnderflow {
                required: n,
                remaining: self.remaining(),
            })
        } else {
            Ok(())
        }
    }

    pub fn read_type_id(&mut self) -> Result<SborTypeId<X>, DecodeError> {
        let id = self.read_byte()?;
        SborTypeId::from_u8(id).ok_or(DecodeError::UnknownTypeId(id))
    }

    pub fn read_discriminator(&mut self) -> Result<String, DecodeError> {
        let n = self.read_size()?;
        let slice = self.read_slice(n)?;
        String::from_utf8(slice.to_vec()).map_err(|_| DecodeError::InvalidUtf8)
    }

    pub fn read_size(&mut self) -> Result<usize, DecodeError> {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(self.read_slice(4)?);
        Ok(u32::from_le_bytes(bytes) as usize)
    }

    pub fn read_byte(&mut self) -> Result<u8, DecodeError> {
        self.require(1)?;
        let result = self.input[self.offset];
        self.offset += 1;
        Ok(result)
    }

    pub fn read_slice(&mut self, n: usize) -> Result<&'de [u8], DecodeError> {
        self.require(n)?;
        let slice = &self.input[self.offset..self.offset + n];
        self.offset += n;
        Ok(slice)
    }

    pub fn check_type_id(&mut self, expected: SborTypeId<X>) -> Result<SborTypeId<X>, DecodeError> {
        let ty = self.read_type_id()?;
        if ty != expected {
            return Err(DecodeError::UnexpectedTypeId(ty.as_u8()));
        }

        Ok(ty)
    }

    pub fn check_size(&mut self, expected: usize) -> Result<(), DecodeError> {
        let len = self.read_size()?;
        if len != expected {
            return Err(DecodeError::UnexpectedSize {
                expected,
                actual: len,
            });
        }

        Ok(())
    }

    pub fn check_end(&self) -> Result<(), DecodeError> {
        let n = self.remaining();
        if n != 0 {
            Err(DecodeError::ExtraTrailingBytes(n))
        } else {
            Ok(())
        }
    }
}

impl<X: CustomTypeId> Decode<X> for () {
    #[inline]
    fn decode_type_id(decoder: &mut Decoder<X>) -> Result<SborTypeId<X>, DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(
        decoder: &mut Decoder<X>,
        _type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        let value = decoder.read_byte()?;
        match value {
            0 => Ok(()),
            _ => Err(DecodeError::InvalidUnit(value)),
        }
    }
}

impl<X: CustomTypeId> Decode<X> for bool {
    #[inline]
    fn decode_type_id(decoder: &mut Decoder<X>) -> Result<SborTypeId<X>, DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(
        decoder: &mut Decoder<X>,
        _type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        let value = decoder.read_byte()?;
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DecodeError::InvalidBool(value)),
        }
    }
}

impl<X: CustomTypeId> Decode<X> for i8 {
    #[inline]
    fn decode_type_id(decoder: &mut Decoder<X>) -> Result<SborTypeId<X>, DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(
        decoder: &mut Decoder<X>,
        _type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        let value = decoder.read_byte()?;
        Ok(value as i8)
    }
}

impl<X: CustomTypeId> Decode<X> for u8 {
    #[inline]
    fn decode_type_id(decoder: &mut Decoder<X>) -> Result<SborTypeId<X>, DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(
        decoder: &mut Decoder<X>,
        _type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        let value = decoder.read_byte()?;
        Ok(value)
    }
}

macro_rules! decode_int {
    ($type:ident, $type_id:ident, $n:expr) => {
        impl<X: CustomTypeId> Decode<X> for $type {
            #[inline]
            fn decode_type_id(decoder: &mut Decoder<X>) -> Result<SborTypeId<X>, DecodeError> {
                decoder.check_type_id(Self::type_id())
            }
            fn decode_value(
                decoder: &mut Decoder<X>,
                _type_id: SborTypeId<X>,
            ) -> Result<Self, DecodeError> {
                let slice = decoder.read_slice($n)?;
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

impl<X: CustomTypeId> Decode<X> for isize {
    #[inline]
    fn decode_type_id(decoder: &mut Decoder<X>) -> Result<SborTypeId<X>, DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(decoder: &mut Decoder<X>, type_id: SborTypeId<X>) -> Result<Self, DecodeError> {
        i64::decode_value(decoder, type_id).map(|i| i as isize)
    }
}

impl<X: CustomTypeId> Decode<X> for usize {
    #[inline]
    fn decode_type_id(decoder: &mut Decoder<X>) -> Result<SborTypeId<X>, DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(decoder: &mut Decoder<X>, type_id: SborTypeId<X>) -> Result<Self, DecodeError> {
        u64::decode_value(decoder, type_id).map(|i| i as usize)
    }
}

impl<X: CustomTypeId> Decode<X> for String {
    #[inline]
    fn decode_type_id(decoder: &mut Decoder<X>) -> Result<SborTypeId<X>, DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(
        decoder: &mut Decoder<X>,
        _type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        let len = decoder.read_size()?;
        let slice = decoder.read_slice(len)?;
        String::from_utf8(slice.to_vec()).map_err(|_| DecodeError::InvalidUtf8)
    }
}

impl<'a, X: CustomTypeId, B: ?Sized + 'a + ToOwned<Owned = O>, O: Decode<X> + TypeId<X>> Decode<X>
    for Cow<'a, B>
{
    #[inline]
    fn decode_type_id(decoder: &mut Decoder<X>) -> Result<SborTypeId<X>, DecodeError> {
        decoder.check_type_id(O::type_id())
    }
    fn decode_value(decoder: &mut Decoder<X>, type_id: SborTypeId<X>) -> Result<Self, DecodeError> {
        let v = O::decode_value(decoder, type_id)?;
        Ok(Cow::Owned(v))
    }
}

impl<X: CustomTypeId, T: Decode<X> + TypeId<X>> Decode<X> for Box<T> {
    #[inline]
    fn decode_type_id(decoder: &mut Decoder<X>) -> Result<SborTypeId<X>, DecodeError> {
        decoder.check_type_id(T::type_id())
    }
    fn decode_value(decoder: &mut Decoder<X>, type_id: SborTypeId<X>) -> Result<Self, DecodeError> {
        let v = T::decode_value(decoder, type_id)?;
        Ok(Box::new(v))
    }
}

impl<X: CustomTypeId, T: Decode<X> + TypeId<X>> Decode<X> for Rc<T> {
    #[inline]
    fn decode_type_id(decoder: &mut Decoder<X>) -> Result<SborTypeId<X>, DecodeError> {
        decoder.check_type_id(T::type_id())
    }
    fn decode_value(decoder: &mut Decoder<X>, type_id: SborTypeId<X>) -> Result<Self, DecodeError> {
        let v = T::decode_value(decoder, type_id)?;
        Ok(Rc::new(v))
    }
}

impl<X: CustomTypeId, T: Decode<X> + TypeId<X>> Decode<X> for RefCell<T> {
    #[inline]
    fn decode_type_id(decoder: &mut Decoder<X>) -> Result<SborTypeId<X>, DecodeError> {
        decoder.check_type_id(T::type_id())
    }
    fn decode_value(decoder: &mut Decoder<X>, type_id: SborTypeId<X>) -> Result<Self, DecodeError> {
        let v = T::decode_value(decoder, type_id)?;
        Ok(RefCell::new(v))
    }
}

impl<X: CustomTypeId, T: Decode<X> + TypeId<X>, const N: usize> Decode<X> for [T; N] {
    #[inline]
    fn decode_type_id(decoder: &mut Decoder<X>) -> Result<SborTypeId<X>, DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(
        decoder: &mut Decoder<X>,
        _type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        let element_type_id = decoder.check_type_id(T::type_id())?;
        decoder.check_size(N)?;

        // Please read:
        // * https://doc.rust-lang.org/stable/std/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
        // * https://github.com/rust-lang/rust/issues/61956
        //
        // TODO: replace with `uninit_array` and `assume_array_init` once they're stable

        // Create an uninitialized array
        let mut data: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };

        // Decode element by element
        for elem in &mut data[..] {
            elem.write(T::decode_value(decoder, element_type_id)?);
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
        impl<X: CustomTypeId, $($name: Decode<X>),+> Decode<X> for ($($name,)+) {
            #[inline]
            fn decode_type_id(decoder: &mut Decoder<X>) -> Result<SborTypeId<X>, DecodeError> {
                decoder.check_type_id(Self::type_id())
            }
            fn decode_value(decoder: &mut Decoder<X>, _type_id: SborTypeId<X>) -> Result<Self, DecodeError> {
                decoder.check_size($n)?;

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

impl<X: CustomTypeId, T: Decode<X>> Decode<X> for Option<T> {
    #[inline]
    fn decode_type_id(decoder: &mut Decoder<X>) -> Result<SborTypeId<X>, DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(
        decoder: &mut Decoder<X>,
        _type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        let discriminator = decoder.read_discriminator()?;

        match discriminator.as_ref() {
            OPTION_VARIANT_SOME => {
                decoder.check_size(1)?;
                Ok(Some(T::decode(decoder)?))
            }
            OPTION_VARIANT_NONE => {
                decoder.check_size(0)?;
                Ok(None)
            }
            _ => Err(DecodeError::UnknownDiscriminator(discriminator)),
        }
    }
}

impl<X: CustomTypeId, T: Decode<X> + TypeId<X>, E: Decode<X> + TypeId<X>> Decode<X>
    for Result<T, E>
{
    #[inline]
    fn decode_type_id(decoder: &mut Decoder<X>) -> Result<SborTypeId<X>, DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(
        decoder: &mut Decoder<X>,
        _type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        let discriminator = decoder.read_discriminator()?;
        match discriminator.as_ref() {
            RESULT_VARIANT_OK => {
                decoder.check_size(1)?;
                Ok(Ok(T::decode(decoder)?))
            }
            RESULT_VARIANT_ERR => {
                decoder.check_size(1)?;
                Ok(Err(E::decode(decoder)?))
            }
            _ => Err(DecodeError::UnknownDiscriminator(discriminator)),
        }
    }
}

impl<X: CustomTypeId, T: Decode<X> + TypeId<X>> Decode<X> for Vec<T> {
    #[inline]
    fn decode_type_id(decoder: &mut Decoder<X>) -> Result<SborTypeId<X>, DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(
        decoder: &mut Decoder<X>,
        _type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        let element_type_id = decoder.check_type_id(T::type_id())?;
        let len = decoder.read_size()?;

        if T::type_id() == SborTypeId::U8 || T::type_id() == SborTypeId::I8 {
            let slice = decoder.read_slice(len)?; // length is checked here
            let mut result = Vec::<T>::with_capacity(len);
            unsafe {
                copy(slice.as_ptr(), result.as_mut_ptr() as *mut u8, slice.len());
                result.set_len(slice.len());
            }
            Ok(result)
        } else {
            let mut result = Vec::<T>::with_capacity(if len <= 1024 { len } else { 1024 });
            for _ in 0..len {
                result.push(T::decode_value(decoder, element_type_id)?);
            }
            Ok(result)
        }
    }
}

impl<X: CustomTypeId, T: Decode<X> + TypeId<X> + Ord> Decode<X> for BTreeSet<T> {
    #[inline]
    fn decode_type_id(decoder: &mut Decoder<X>) -> Result<SborTypeId<X>, DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(decoder: &mut Decoder<X>, type_id: SborTypeId<X>) -> Result<Self, DecodeError> {
        let elements: Vec<T> = Vec::<T>::decode_value(decoder, type_id)?;
        Ok(elements.into_iter().collect())
    }
}

impl<X: CustomTypeId, T: Decode<X> + TypeId<X> + Hash + Eq> Decode<X> for HashSet<T> {
    #[inline]
    fn decode_type_id(decoder: &mut Decoder<X>) -> Result<SborTypeId<X>, DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(decoder: &mut Decoder<X>, type_id: SborTypeId<X>) -> Result<Self, DecodeError> {
        let elements: Vec<T> = Vec::<T>::decode_value(decoder, type_id)?;
        Ok(elements.into_iter().collect())
    }
}

impl<X: CustomTypeId, K: Decode<X> + TypeId<X> + Ord, V: Decode<X> + TypeId<X>> Decode<X>
    for BTreeMap<K, V>
{
    #[inline]
    fn decode_type_id(decoder: &mut Decoder<X>) -> Result<SborTypeId<X>, DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(decoder: &mut Decoder<X>, type_id: SborTypeId<X>) -> Result<Self, DecodeError> {
        let elements = Vec::<(K, V)>::decode_value(decoder, type_id)?;
        Ok(elements.into_iter().collect())
    }
}

impl<X: CustomTypeId, K: Decode<X> + TypeId<X> + Hash + Eq, V: Decode<X> + TypeId<X>> Decode<X>
    for HashMap<K, V>
{
    #[inline]
    fn decode_type_id(decoder: &mut Decoder<X>) -> Result<SborTypeId<X>, DecodeError> {
        decoder.check_type_id(Self::type_id())
    }
    fn decode_value(decoder: &mut Decoder<X>, type_id: SborTypeId<X>) -> Result<Self, DecodeError> {
        let elements: Vec<(K, V)> = Vec::<(K, V)>::decode_value(decoder, type_id)?;
        Ok(elements.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::Encode;
    use crate::encode::Encoder;
    use crate::rust::borrow::ToOwned;
    use crate::rust::vec;

    fn assert_decoding<X: CustomTypeId>(dec: &mut Decoder<X>) {
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

        assert_eq!(Some(1u32), <Option<u32>>::decode(dec).unwrap());
        assert_eq!(None, <Option<u32>>::decode(dec).unwrap());
        assert_eq!(Ok(1u32), <Result<u32, String>>::decode(dec).unwrap());
        assert_eq!(
            Err("hello".to_owned()),
            <Result<u32, String>>::decode(dec).unwrap()
        );
    }

    #[test]
    pub fn test_decoding() {
        let bytes = vec![
            0, 0, // unit
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
            32, 9, 3, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, // array
            33, 2, 0, 0, 0, 9, 1, 0, 0, 0, 9, 2, 0, 0, 0, // tuple
            32, 9, 3, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, // vec
            32, 7, 2, 0, 0, 0, 1, 2, // set
            32, 33, 2, 0, 0, 0, 2, 0, 0, 0, 7, 1, 7, 2, 2, 0, 0, 0, 7, 3, 7, 4, // map
            17, 4, 0, 0, 0, 83, 111, 109, 101, 1, 0, 0, 0, 9, 1, 0, 0, 0, // Some<T>
            17, 4, 0, 0, 0, 78, 111, 110, 101, 0, 0, 0, 0, // None
            17, 2, 0, 0, 0, 79, 107, 1, 0, 0, 0, 9, 1, 0, 0, 0, // Ok<T>
            17, 3, 0, 0, 0, 69, 114, 114, 1, 0, 0, 0, 12, 5, 0, 0, 0, 104, 101, 108, 108,
            111, // Err<T>
        ];
        let mut dec = Decoder::<NoCustomTypeId>::new(&bytes);
        assert_decoding(&mut dec);
    }

    #[test]
    pub fn test_decode_box() {
        let bytes = vec![7u8, 5u8];
        let mut dec = Decoder::<NoCustomTypeId>::new(&bytes);
        let x = <Box<u8>>::decode(&mut dec).unwrap();
        assert_eq!(Box::new(5u8), x);
    }

    #[test]
    pub fn test_decode_rc() {
        let bytes = vec![7u8, 5u8];
        let mut dec = Decoder::<NoCustomTypeId>::new(&bytes);
        let x = <Rc<u8>>::decode(&mut dec).unwrap();
        assert_eq!(Rc::new(5u8), x);
    }

    #[test]
    pub fn test_decode_ref_cell() {
        let bytes = vec![7u8, 5u8];
        let mut dec = Decoder::<NoCustomTypeId>::new(&bytes);
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
        let mut enc = Encoder::<NoCustomTypeId>::new(&mut bytes);
        value1.encode(&mut enc);

        let mut dec = Decoder::<NoCustomTypeId>::new(&bytes);
        let value2 = <[NFA; 2]>::decode(&mut dec).unwrap();
        assert_eq!(value1, value2);
    }
}
