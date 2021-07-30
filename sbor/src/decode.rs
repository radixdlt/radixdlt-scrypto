use crate::*;

pub trait Decode<'de>: Sized {
    fn decode(decoder: &Decoder<'de>) -> Result<Self, String>;
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
            1 => Ok(Some(T::decode(&self)?)),
            0 => Ok(None),
            _ => Err(format!("Invalid option value: {}", n[0])),
        }
    }

    pub fn decode_vec<T: Decode<'de>>(&mut self) -> Result<Vec<T>, String> {
        let n = self.read_type(TYPE_VEC, 2)?;
        let len = Self::as_u16(n);

        let mut result = Vec::<T>::new();
        for _ in 0..len {
            result.push(T::decode(&self)?);
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

        let result = (A::decode(&self)?, B::decode(&self)?);
        Ok(result)
    }

    pub fn decode_struct<T: Decode<'de>>(&mut self) -> Result<T, String> {
        T::decode(&self)
    }

    pub fn decode_enum<T: Decode<'de>>(&mut self) -> Result<T, String> {
        T::decode(&self)
    }

    fn as_u16(slice: &[u8]) -> u16 {
        assert!(slice.len() >= 2);
        ((slice[0] as u16) << 8) + (slice[1] as u16) // big endian
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
