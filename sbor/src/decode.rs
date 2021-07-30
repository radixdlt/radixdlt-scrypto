use crate::*;

pub trait Decode<'de>: Sized {
    fn decode(decoder: &mut Decoder<'de>) -> Result<Self, String>;
}

macro_rules! decode_basic_type {
    ($type:ident, $method:ident) => {
        impl<'de> Decode<'de> for $type {
            fn decode(decoder: &mut Decoder<'de>) -> Result<Self, String> {
                decoder.$method()
            }
        }
    };
}

decode_basic_type!(bool, decode_bool);
decode_basic_type!(i8, decode_i8);
decode_basic_type!(i16, decode_i16);
decode_basic_type!(i32, decode_i32);
decode_basic_type!(i64, decode_i64);
decode_basic_type!(i128, decode_i128);
decode_basic_type!(u8, decode_u8);
decode_basic_type!(u16, decode_u16);
decode_basic_type!(u32, decode_u32);
decode_basic_type!(u64, decode_u64);
decode_basic_type!(u128, decode_u128);

impl<'de> Decode<'de> for String {
    fn decode(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        decoder.decode_string()
    }
}

impl<'de, T: Decode<'de>> Decode<'de> for Option<T> {
    fn decode(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        decoder.decode_option()
    }
}

impl<'de, T: Decode<'de>, const N: usize> Decode<'de> for [T; N] {
    fn decode(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        decoder.decode_array()
    }
}

impl<'de, T: Decode<'de>> Decode<'de> for Vec<T> {
    fn decode(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        decoder.decode_vec()
    }
}

pub struct Decoder<'de> {
    input: &'de [u8],
    offset: usize,
}

macro_rules! decode_int {
    ($method:ident, $sbor_type:expr, $native_type:ty, $n:expr) => {
        pub fn $method(&mut self) -> Result<$native_type, String> {
            let slice = self.read_type($sbor_type, $n)?;
            let mut value = [0u8; $n];
            value.copy_from_slice(&slice[0..$n]);
            Ok(<$native_type>::from_be_bytes(value))
        }
    };
}

impl<'de> Decoder<'de> {
    pub fn new(input: &'de [u8]) -> Self {
        Self { input, offset: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.input.len() - self.offset
    }

    pub fn decode_unit(&mut self) -> Result<(), String> {
        self.read_type(TYPE_UNIT, 0)?;
        Ok(())
    }

    pub fn decode_bool(&mut self) -> Result<bool, String> {
        let t = self.read_type(TYPE_BOOL, 1)?;
        if t[0] != 0 && t[0] != 1 {
            Err(format!("Invalid boolean value: {}", t[0]))
        } else {
            Ok(t[0] == 1)
        }
    }

    decode_int!(decode_i8, TYPE_I8, i8, 1);
    decode_int!(decode_i16, TYPE_I16, i16, 2);
    decode_int!(decode_i32, TYPE_I32, i32, 4);
    decode_int!(decode_i64, TYPE_I64, i64, 8);
    decode_int!(decode_i128, TYPE_I128, i128, 16);
    decode_int!(decode_u8, TYPE_U8, u8, 1);
    decode_int!(decode_u16, TYPE_U16, u16, 2);
    decode_int!(decode_u32, TYPE_U32, u32, 4);
    decode_int!(decode_u64, TYPE_U64, u64, 8);
    decode_int!(decode_u128, TYPE_U128, u128, 16);

    pub fn decode_string(&mut self) -> Result<String, String> {
        let n = self.read_type(TYPE_STRING, 2)?;
        let slice = self.read(Self::as_u16(n) as usize)?;
        let s = String::from_utf8(slice.to_vec()).map_err(|_| "Invalid utf-8");
        Ok(s?)
    }

    pub fn decode_option<T: Decode<'de>>(&mut self) -> Result<Option<T>, String> {
        let n = self.read_type(TYPE_OPTION, 1)?;

        match n[0] {
            1 => Ok(Some(T::decode(self)?)),
            0 => Ok(None),
            _ => Err(format!("Invalid option value: {}", n[0])),
        }
    }

    pub fn decode_array<T: Decode<'de>, const N: usize>(&mut self) -> Result<[T; N], String> {
        let n = self.read_type(TYPE_ARRAY, 2)?;
        let len = Self::as_u16(n) as usize;
        if len != N {
            return Err(format!(
                "Invalid array length: expected = {}, actual = {}",
                N, len
            ));
        }

        let mut x = core::mem::MaybeUninit::<[T; N]>::uninit();
        let x_arr = unsafe { &mut *x.as_mut_ptr() };
        for i in 0..len {
            x_arr[i] = T::decode(self)?;
        }
        let arr = unsafe { x.assume_init() };
        Ok(arr)
    }

    pub fn decode_vec<T: Decode<'de>>(&mut self) -> Result<Vec<T>, String> {
        let n = self.read_type(TYPE_VEC, 2)?;
        let len = Self::as_u16(n) as usize;

        let mut result = Vec::<T>::new();
        for _ in 0..len {
            result.push(T::decode(self)?);
        }
        Ok(result)
    }

    // TODO expand to different lengths
    pub fn decode_tuple<A: Decode<'de>, B: Decode<'de>>(&mut self) -> Result<(A, B), String> {
        let n = self.read_type(TYPE_TUPLE, 2)?;
        let len = Self::as_u16(n);

        if len != 2 {
            return Err(format!(
                "Invalid tuple length: expected = {}, actual = {}",
                2, len
            ));
        }

        let result = (A::decode(self)?, B::decode(self)?);
        Ok(result)
    }

    pub fn decode_struct<T: Decode<'de>>(&mut self) -> Result<T, String> {
        T::decode(self)
    }

    pub fn decode_enum<T: Decode<'de>>(&mut self) -> Result<T, String> {
        T::decode(self)
    }

    fn as_u16(slice: &[u8]) -> u16 {
        let mut bytes = [0u8; 2];
        bytes.copy_from_slice(&slice[0..2]);
        u16::from_be_bytes(bytes)
    }

    fn read_type(&mut self, ty: u8, n: usize) -> Result<&'de [u8], String> {
        let slice = self.read(n + 1)?;
        if slice[0] != ty {
            return Err(format!(
                "Unexpected type: expected = {}, actual = {}",
                ty, slice[0]
            ));
        }
        Ok(&slice[1..])
    }

    fn read(&mut self, n: usize) -> Result<&'de [u8], String> {
        if self.remaining() < n {
            return Err(format!(
                "Buffer underflow: required = {}, remaining = {}",
                n,
                self.remaining()
            ));
        }
        let slice = &self.input[self.offset..self.offset + n];
        self.offset += n;
        Ok(slice)
    }
}

#[cfg(test)]
mod tests {
    use super::Decoder;

    #[test]
    pub fn test_decoding() {
        let bytes = vec![
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
            16, 0, 2, 9, 0, 0, 0, 1, 9, 0, 0, 0, 2, // tuple
        ];
        let mut dec = Decoder::new(&bytes);
        dec.decode_unit().unwrap();
        assert_eq!(true, dec.decode_bool().unwrap());
        assert_eq!(1, dec.decode_i8().unwrap());
        assert_eq!(1, dec.decode_i16().unwrap());
        assert_eq!(1, dec.decode_i32().unwrap());
        assert_eq!(1, dec.decode_i64().unwrap());
        assert_eq!(1, dec.decode_i128().unwrap());
        assert_eq!(1, dec.decode_u8().unwrap());
        assert_eq!(1, dec.decode_u16().unwrap());
        assert_eq!(1, dec.decode_u32().unwrap());
        assert_eq!(1, dec.decode_u64().unwrap());
        assert_eq!(1, dec.decode_u128().unwrap());
        assert_eq!("hello", dec.decode_string().unwrap());
        assert_eq!(Some(1u32), dec.decode_option().unwrap());
        assert_eq!([1u32, 2u32, 3u32], dec.decode_array().unwrap());
        assert_eq!(vec![1u32, 2u32, 3u32], dec.decode_vec().unwrap());
        assert_eq!((1u32, 2u32), dec.decode_tuple().unwrap());
    }
}
