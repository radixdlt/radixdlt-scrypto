use crate::*;

pub trait Decode: Sized {
    fn decode<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String>;
}

pub struct Decoder<'de> {
    input: &'de [u8],
    offset: usize,
    with_schema: bool,
}

impl<'de> Decoder<'de> {
    pub fn new(input: &'de [u8]) -> Self {
        Self {
            input,
            offset: 0,
            with_schema: true,
        }
    }

    pub fn new_no_schema(input: &'de [u8]) -> Self {
        Self {
            input,
            offset: 0,
            with_schema: false,
        }
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
        if self.with_schema {
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
        if self.with_schema {
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

// implementation for basic types

impl Decode for () {
    fn decode<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        decoder.check_type(TYPE_UNIT)?;
        Ok(())
    }
}

impl Decode for bool {
    fn decode<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        decoder.check_type(TYPE_BOOL)?;
        let value = decoder.read_u8()?;
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(format!("Invalid boolean value: {}", value)),
        }
    }
}

impl Decode for i8 {
    fn decode<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        decoder.check_type(TYPE_I8)?;
        let value = decoder.read_u8()?;
        Ok(value as i8)
    }
}

impl Decode for u8 {
    fn decode<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        decoder.check_type(TYPE_U8)?;
        let value = decoder.read_u8()?;
        Ok(value)
    }
}

macro_rules! decode_basic_type {
    ($type:ident, $sbor_type:ident, $n:expr) => {
        impl Decode for $type {
            fn decode<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
                decoder.check_type($sbor_type)?;
                let slice = decoder.read_bytes($n)?;
                let mut bytes = [0u8; $n];
                bytes.copy_from_slice(&slice[..]);
                Ok(<$type>::from_le_bytes(bytes))
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
    fn decode<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        decoder.check_type(TYPE_STRING)?;
        let len = decoder.read_len()?;
        let slice = decoder.read_bytes(len)?;
        let s = String::from_utf8(slice.to_vec()).map_err(|_| "Invalid utf-8");
        Ok(s?)
    }
}

impl<T: Decode> Decode for Option<T> {
    fn decode<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        decoder.check_type(TYPE_OPTION)?;
        let index = decoder.read_index()?;

        match index {
            0 => Ok(None),
            1 => Ok(Some(T::decode(decoder)?)),
            _ => Err(format!("Invalid option index: {}", index)),
        }
    }
}

impl<T: Decode, const N: usize> Decode for [T; N] {
    fn decode<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        decoder.check_type(TYPE_ARRAY)?;
        let len = decoder.read_len()?;
        if len != N {
            return Err(format!(
                "Invalid array length: expected = {}, actual = {}",
                N, len
            ));
        }

        let mut x = core::mem::MaybeUninit::<[T; N]>::uninit();
        let arr = unsafe { &mut *x.as_mut_ptr() };
        for i in 0..len {
            arr[i] = T::decode(decoder)?;
        }
        Ok(unsafe { x.assume_init() })
    }
}

impl<T: Decode> Decode for Vec<T> {
    fn decode<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        decoder.check_type(TYPE_VEC)?;
        let len = decoder.read_len()?;

        let mut result = Vec::<T>::with_capacity(len); // Lengths are u16, so it's safe to pre-allocate.
        for _ in 0..len {
            result.push(T::decode(decoder)?);
        }
        Ok(result)
    }
}

// TODO expand to different lengths
impl<A: Decode, B: Decode> Decode for (A, B) {
    fn decode<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        decoder.check_type(TYPE_TUPLE)?;
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
            14, 3, 0, 9, 1, 0, 0, 0, 9, 2, 0, 0, 0, 9, 3, 0, 0, 0, // array
            15, 3, 0, 9, 1, 0, 0, 0, 9, 2, 0, 0, 0, 9, 3, 0, 0, 0, // vector
            16, 2, 0, 9, 1, 0, 0, 0, 9, 2, 0, 0, 0, // tuple
        ];
        let mut dec = Decoder::new(&bytes);
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
    pub fn test_decoding_no_schema() {
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
        let mut dec = Decoder::new_no_schema(&bytes);
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
