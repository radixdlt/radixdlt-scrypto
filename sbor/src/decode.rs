use crate::*;

pub trait Decode: Sized {
    fn decode<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        decoder.check_type(Self::sbor_type())?;
        Self::decode_value(decoder)
    }

    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String>;

    fn sbor_type() -> u8;
}

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

    #[inline(always)]
    pub fn remaining(&self) -> usize {
        self.input.len() - self.offset
    }

    #[inline(always)]
    pub fn require(&self, n: usize) -> Result<(), String> {
        if self.remaining() < n {
            Err(format!(
                "Buffer underflow: required = {}, remaining = {}",
                n,
                self.remaining()
            ))
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    pub fn read_u8(&mut self) -> Result<u8, String> {
        self.require(1)?;
        let result = self.input[self.offset];
        self.offset += 1;
        Ok(result)
    }

    #[inline(always)]
    pub fn read_bytes(&mut self, n: usize) -> Result<&'de [u8], String> {
        self.require(n)?;
        let slice = &self.input[self.offset..self.offset + n];
        self.offset += n;
        Ok(slice)
    }

    #[inline(always)]
    pub fn read_type(&mut self) -> Result<u8, String> {
        self.read_u8()
    }

    #[inline(always)]
    pub fn read_len(&mut self) -> Result<usize, String> {
        let mut bytes = [0u8; 2];
        bytes.copy_from_slice(&self.read_bytes(2)?[..]);
        Ok(u16::from_le_bytes(bytes) as usize)
    }

    #[inline(always)]
    pub fn read_index(&mut self) -> Result<usize, String> {
        Ok(self.read_u8()? as usize)
    }

    #[inline(always)]
    pub fn check_type(&mut self, expected: u8) -> Result<(), String> {
        if self.with_metadata {
            let ty = self.read_type()?;
            if ty != expected {
                return Err(format!(
                    "Unexpected type: expected = {}, actual = {}",
                    expected, ty
                ));
            }
        }

        Ok(())
    }

    #[inline(always)]
    pub fn check_name(&mut self, expected: &str) -> Result<(), String> {
        if self.with_metadata {
            self.check_type(TYPE_STRING)?;
            self.check_len(expected.len())?;

            let slice = self.read_bytes(expected.len())?;
            if slice != expected.as_bytes() {
                return Err(format!(
                    "Unexpected name: expected = {}, actual = {}",
                    expected,
                    String::from_utf8(slice.to_vec()).unwrap_or("<unknown>".to_string())
                ));
            }
        }

        Ok(())
    }

    #[inline(always)]
    pub fn check_len(&mut self, expected: usize) -> Result<(), String> {
        let len = self.read_len()?;
        if len != expected {
            return Err(format!(
                "Unexpected length: expected = {}, actual = {}",
                expected, len
            ));
        }

        Ok(())
    }
}

// Implementation for basic types:
// - We keep one flat implementation per type, i.e., the `decode()` function;
// - Everything else is inlined.

impl Decode for () {
    #[inline]
    fn decode_value<'de>(_decoder: &mut Decoder<'de>) -> Result<Self, String> {
        Ok(())
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_UNIT
    }
}

impl Decode for bool {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        let value = decoder.read_u8()?;
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(format!("Invalid boolean value: {}", value)),
        }
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_BOOL
    }
}

impl Decode for i8 {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
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
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        let value = decoder.read_u8()?;
        Ok(value)
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_U8
    }
}

macro_rules! decode_basic_type {
    ($type:ident, $sbor_type:ident, $n:expr) => {
        impl Decode for $type {
            #[inline]
            fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
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

decode_basic_type!(i16, TYPE_I16, 2);
decode_basic_type!(i32, TYPE_I32, 4);
decode_basic_type!(i64, TYPE_I64, 8);
decode_basic_type!(i128, TYPE_I128, 16);
decode_basic_type!(u16, TYPE_U16, 2);
decode_basic_type!(u32, TYPE_U32, 4);
decode_basic_type!(u64, TYPE_U64, 8);
decode_basic_type!(u128, TYPE_U128, 16);

impl Decode for String {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        let len = decoder.read_len()?;
        let slice = decoder.read_bytes(len)?;
        let s = String::from_utf8(slice.to_vec()).map_err(|_| "Invalid utf-8");
        Ok(s?)
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_STRING
    }
}

impl<T: Decode> Decode for Option<T> {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        let index = decoder.read_index()?;

        match index {
            0 => Ok(None),
            1 => Ok(Some(T::decode(decoder)?)),
            _ => Err(format!("Invalid option index: {}", index)),
        }
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_OPTION
    }
}

impl<T: Decode, const N: usize> Decode for [T; N] {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        decoder.check_len(N)?;
        decoder.check_type(T::sbor_type())?;

        let mut x = core::mem::MaybeUninit::<[T; N]>::uninit();
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

impl<T: Decode> Decode for Vec<T> {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        let len = decoder.read_len()?;
        decoder.check_type(T::sbor_type())?;

        let mut result = Vec::<T>::with_capacity(len); // Lengths are u16, so it's safe to pre-allocate.
        for _ in 0..len {
            result.push(T::decode_value(decoder)?);
        }
        Ok(result)
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_VEC
    }
}

// TODO expand to different lengths
impl<A: Decode, B: Decode> Decode for (A, B) {
    #[inline]
    fn decode_value<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        let len = decoder.read_len()?;

        if len != 2 {
            return Err(format!(
                "Invalid tuple length: expected = {}, actual = {}",
                2, len
            ));
        }

        let result = (A::decode(decoder)?, B::decode(decoder)?);
        Ok(result)
    }

    #[inline]
    fn sbor_type() -> u8 {
        TYPE_TUPLE
    }
}

#[cfg(test)]
mod tests {
    use super::{Decode, Decoder};

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
            12, 5, 0, 104, 101, 108, 108, 111, // string
            13, 1, 9, 1, 0, 0, 0, // option
            14, 3, 0, 9, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, // array
            15, 3, 0, 9, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, // vector
            16, 2, 0, 9, 1, 0, 0, 0, 9, 2, 0, 0, 0, // tuple
        ];
        let mut dec = Decoder::with_metadata(&bytes);
        <()>::decode(&mut dec).unwrap();
        assert_eq!(true, <bool>::decode(&mut dec).unwrap());
        assert_eq!(1, <i8>::decode(&mut dec).unwrap());
        assert_eq!(1, <i16>::decode(&mut dec).unwrap());
        assert_eq!(1, <i32>::decode(&mut dec).unwrap());
        assert_eq!(1, <i64>::decode(&mut dec).unwrap());
        assert_eq!(1, <i128>::decode(&mut dec).unwrap());
        assert_eq!(1, <u8>::decode(&mut dec).unwrap());
        assert_eq!(1, <u16>::decode(&mut dec).unwrap());
        assert_eq!(1, <u32>::decode(&mut dec).unwrap());
        assert_eq!(1, <u64>::decode(&mut dec).unwrap());
        assert_eq!(1, <u128>::decode(&mut dec).unwrap());
        assert_eq!("hello", <String>::decode(&mut dec).unwrap());
        assert_eq!(Some(1u32), <Option<u32>>::decode(&mut dec).unwrap());
        assert_eq!([1u32, 2u32, 3u32], <[u32; 3]>::decode(&mut dec).unwrap());
        assert_eq!(
            vec![1u32, 2u32, 3u32],
            <Vec<u32>>::decode(&mut dec).unwrap()
        );
        assert_eq!((1u32, 2u32), <(u32, u32)>::decode(&mut dec).unwrap());
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
            5, 0, 104, 101, 108, 108, 111, // string
            1, 1, 0, 0, 0, // option
            3, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, // array
            3, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, // vector
            2, 0, 1, 0, 0, 0, 2, 0, 0, 0, // tuple
        ];
        let mut dec = Decoder::no_metadata(&bytes);
        <()>::decode(&mut dec).unwrap();
        assert_eq!(true, <bool>::decode(&mut dec).unwrap());
        assert_eq!(1, <i8>::decode(&mut dec).unwrap());
        assert_eq!(1, <i16>::decode(&mut dec).unwrap());
        assert_eq!(1, <i32>::decode(&mut dec).unwrap());
        assert_eq!(1, <i64>::decode(&mut dec).unwrap());
        assert_eq!(1, <i128>::decode(&mut dec).unwrap());
        assert_eq!(1, <u8>::decode(&mut dec).unwrap());
        assert_eq!(1, <u16>::decode(&mut dec).unwrap());
        assert_eq!(1, <u32>::decode(&mut dec).unwrap());
        assert_eq!(1, <u64>::decode(&mut dec).unwrap());
        assert_eq!(1, <u128>::decode(&mut dec).unwrap());
        assert_eq!("hello", <String>::decode(&mut dec).unwrap());
        assert_eq!(Some(1u32), <Option<u32>>::decode(&mut dec).unwrap());
        assert_eq!([1u32, 2u32, 3u32], <[u32; 3]>::decode(&mut dec).unwrap());
        assert_eq!(
            vec![1u32, 2u32, 3u32],
            <Vec<u32>>::decode(&mut dec).unwrap()
        );
        assert_eq!((1u32, 2u32), <(u32, u32)>::decode(&mut dec).unwrap());
    }
}
