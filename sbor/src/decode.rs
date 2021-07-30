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
            self.read_type_and_check($sbor_type)?;
            let slice = self.read($n)?;
            let mut bytes = [0u8; $n];
            bytes.copy_from_slice(&slice[..]);
            Ok(<$native_type>::from_be_bytes(bytes))
        }
    };
}

impl<'de> Decoder<'de> {
    pub fn new(input: &'de [u8]) -> Self {
        Self { input, offset: 0 }
    }

    pub fn remaining_bytes(&self) -> usize {
        self.input.len() - self.offset
    }

    pub fn read(&mut self, n: usize) -> Result<&'de [u8], String> {
        if self.remaining_bytes() < n {
            return Err(format!(
                "Buffer underflow: required = {}, remaining_bytes = {}",
                n,
                self.remaining_bytes()
            ));
        }
        let slice = &self.input[self.offset..self.offset + n];
        self.offset += n;
        Ok(slice)
    }

    pub fn read_type(&mut self) -> Result<u8, String> {
        Ok(self.read(1)?[0])
    }

    pub fn read_type_and_check(&mut self, expected: u8) -> Result<(), String> {
        let ty = self.read_type()?;
        if ty != expected {
            return Err(format!(
                "Unexpected type: expected = {}, actual = {}",
                expected, ty
            ));
        }
        Ok(())
    }

    pub fn read_len(&mut self) -> Result<usize, String> {
        let mut bytes = [0u8; 2];
        bytes.copy_from_slice(&self.read(2)?[..]);
        Ok(u16::from_be_bytes(bytes) as usize)
    }

    pub fn decode_unit(&mut self) -> Result<(), String> {
        self.read_type_and_check(TYPE_UNIT)?;
        Ok(())
    }

    pub fn decode_bool(&mut self) -> Result<bool, String> {
        self.read_type_and_check(TYPE_BOOL)?;
        let slice = self.read(1)?;
        if slice[0] != 0 && slice[0] != 1 {
            Err(format!("Invalid boolean value: {}", slice[0]))
        } else {
            Ok(slice[0] == 1)
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
        self.read_type_and_check(TYPE_STRING)?;
        let len = self.read_len()?;
        let slice = self.read(len)?;
        let s = String::from_utf8(slice.to_vec()).map_err(|_| "Invalid utf-8");
        Ok(s?)
    }

    pub fn decode_option<T: Decode<'de>>(&mut self) -> Result<Option<T>, String> {
        self.read_type_and_check(TYPE_OPTION)?;
        let slice = self.read(1)?;

        match slice[0] {
            1 => Ok(Some(T::decode(self)?)),
            0 => Ok(None),
            _ => Err(format!("Invalid option value: {}", slice[0])),
        }
    }

    pub fn decode_array<T: Decode<'de>, const N: usize>(&mut self) -> Result<[T; N], String> {
        self.read_type_and_check(TYPE_ARRAY)?;
        let len = self.read_len()?;
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
        self.read_type_and_check(TYPE_VEC)?;
        let len = self.read_len()?;

        let mut result = Vec::<T>::new();
        for _ in 0..len {
            result.push(T::decode(self)?);
        }
        Ok(result)
    }

    // TODO expand to different lengths
    pub fn decode_tuple<A: Decode<'de>, B: Decode<'de>>(&mut self) -> Result<(A, B), String> {
        self.read_type_and_check(TYPE_TUPLE)?;
        let len = self.read_len()?;

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
