use crate::*;

pub trait Decode: Sized {
    fn decode<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String>;
}

macro_rules! decode_basic_type {
    ($type:ident, $method:ident) => {
        impl Decode for $type {
            fn decode<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
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

impl Decode for String {
    fn decode<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        decoder.decode_string()
    }
}

impl<T: Decode> Decode for Option<T> {
    fn decode<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        decoder.decode_option()
    }
}

impl<T: Decode, const N: usize> Decode for [T; N] {
    fn decode<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        decoder.decode_array()
    }
}

impl<T: Decode> Decode for Vec<T> {
    fn decode<'de>(decoder: &mut Decoder<'de>) -> Result<Self, String> {
        decoder.decode_vec()
    }
}

pub struct Decoder<'de> {
    input: &'de [u8],
    offset: usize,
    with_schema: bool,
}

macro_rules! decode_int {
    ($method:ident, $sbor_type:expr, $native_type:ty, $n:expr) => {
        pub fn $method(&mut self) -> Result<$native_type, String> {
            self.check_type($sbor_type)?;
            let slice = self.decode($n)?;
            let mut bytes = [0u8; $n];
            bytes.copy_from_slice(&slice[..]);
            Ok(<$native_type>::from_le_bytes(bytes))
        }
    };
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

    pub fn remaining_bytes(&self) -> usize {
        self.input.len() - self.offset
    }

    pub fn check_type(&mut self, expected: u8) -> Result<(), String> {
        if self.with_schema {
            let ty = self.decode_type()?;
            if ty != expected {
                return Err(format!(
                    "Unexpected type: expected = {}, actual = {}",
                    expected, ty
                ));
            }
        }

        Ok(())
    }

    pub fn check_name(&mut self, expected: &str) -> Result<(), String> {
        if self.with_schema {
            self.check_type(TYPE_STRING)?;
            self.check_len(expected.len())?;

            let slice = self.decode(expected.len())?;
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

    pub fn check_len(&mut self, expected: usize) -> Result<(), String> {
        let len = self.decode_len()?;
        if len != expected {
            return Err(format!(
                "Unexpected length: expected = {}, actual = {}",
                expected, len
            ));
        }

        Ok(())
    }

    pub fn decode(&mut self, n: usize) -> Result<&'de [u8], String> {
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

    pub fn decode_type(&mut self) -> Result<u8, String> {
        Ok(self.decode(1)?[0])
    }

    pub fn decode_len(&mut self) -> Result<usize, String> {
        let mut bytes = [0u8; 2];
        bytes.copy_from_slice(&self.decode(2)?[..]);
        Ok(u16::from_le_bytes(bytes) as usize)
    }

    pub fn decode_index(&mut self) -> Result<usize, String> {
        Ok(self.decode(1)?[0] as usize)
    }

    pub fn decode_unit(&mut self) -> Result<(), String> {
        self.check_type(TYPE_UNIT)?;
        Ok(())
    }

    pub fn decode_bool(&mut self) -> Result<bool, String> {
        self.check_type(TYPE_BOOL)?;
        let slice = self.decode(1)?;
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
        self.check_type(TYPE_STRING)?;
        let len = self.decode_len()?;
        let slice = self.decode(len)?;
        let s = String::from_utf8(slice.to_vec()).map_err(|_| "Invalid utf-8");
        Ok(s?)
    }

    pub fn decode_option<T: Decode>(&mut self) -> Result<Option<T>, String> {
        self.check_type(TYPE_OPTION)?;
        let index = self.decode_index()?;

        match index {
            0 => Ok(None),
            1 => Ok(Some(T::decode(self)?)),
            _ => Err(format!("Invalid option index: {}", index)),
        }
    }

    pub fn decode_array<T: Decode, const N: usize>(&mut self) -> Result<[T; N], String> {
        self.check_type(TYPE_ARRAY)?;
        let len = self.decode_len()?;
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

    pub fn decode_vec<T: Decode>(&mut self) -> Result<Vec<T>, String> {
        self.check_type(TYPE_VEC)?;
        let len = self.decode_len()?;

        let mut result = Vec::<T>::new();
        for _ in 0..len {
            result.push(T::decode(self)?);
        }
        Ok(result)
    }

    // TODO expand to different lengths
    pub fn decode_tuple<A: Decode, B: Decode>(&mut self) -> Result<(A, B), String> {
        self.check_type(TYPE_TUPLE)?;
        let len = self.decode_len()?;

        if len != 2 {
            return Err(format!(
                "Invalid tuple length: expected = {}, actual = {}",
                2, len
            ));
        }

        let result = (A::decode(self)?, B::decode(self)?);
        Ok(result)
    }

    pub fn decode_struct<T: Decode>(&mut self) -> Result<T, String> {
        T::decode(self)
    }

    pub fn decode_enum<T: Decode>(&mut self) -> Result<T, String> {
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
