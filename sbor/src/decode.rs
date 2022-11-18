use crate::rust::marker::PhantomData;
use crate::rust::string::String;
use crate::type_id::*;
use crate::*;

/// Represents an error ocurred during decoding.
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub enum DecodeError {
    ExtraTrailingBytes(usize),

    BufferUnderflow { required: usize, remaining: usize },

    UnexpectedTypeId { expected: u8, actual: u8 },

    UnexpectedSize { expected: usize, actual: usize },

    UnknownTypeId(u8),

    UnknownDiscriminator(String),

    InvalidUnit(u8),

    InvalidBool(u8),

    InvalidUtf8,

    SizeTooLarge,

    InvalidCustomValue, // TODO: generify custom error codes
}

/// A data structure that can be decoded from a byte array using SBOR.
pub trait Decode<X: CustomTypeId>: Sized {
    /// Decodes from the byte array encapsulated by the given decoder.
    fn decode(decoder: &mut Decoder<X>) -> Result<Self, DecodeError> {
        let type_id = decoder.read_type_id()?;
        Self::decode_body_with_type_id(decoder, type_id)
    }

    /// Decodes from the byte array encapsulated by the given decoder, with a preloaded type id.
    fn decode_body_with_type_id(
        decoder: &mut Decoder<X>,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError>;
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
        // LEB128 and 4 bytes max
        let mut size = 0usize;
        let mut shift = 0;
        loop {
            let byte = self.read_byte()?;
            size |= ((byte & 0x7F) as usize) << shift;
            if byte < 0x80 {
                break;
            }
            shift += 7;
            if shift >= 28 {
                return Err(DecodeError::SizeTooLarge);
            }
        }
        Ok(size)
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

    pub fn check_preloaded_type_id(
        &self,
        type_id: SborTypeId<X>,
        expected: SborTypeId<X>,
    ) -> Result<SborTypeId<X>, DecodeError> {
        if type_id == expected {
            Ok(type_id)
        } else {
            Err(DecodeError::UnexpectedTypeId {
                actual: type_id.as_u8(),
                expected: expected.as_u8(),
            })
        }
    }

    pub fn read_and_check_type_id(
        &mut self,
        expected: SborTypeId<X>,
    ) -> Result<SborTypeId<X>, DecodeError> {
        let type_id = self.read_type_id()?;
        self.check_preloaded_type_id(type_id, expected)
    }

    pub fn read_and_check_size(&mut self, expected: usize) -> Result<(), DecodeError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::Encode;
    use crate::encode::Encoder;
    use crate::rust::borrow::ToOwned;
    use crate::rust::boxed::Box;
    use crate::rust::cell::RefCell;
    use crate::rust::collections::*;
    use crate::rust::rc::Rc;
    use crate::rust::string::String;
    use crate::rust::vec;
    use crate::rust::vec::Vec;

    fn encode_decode_size(size: usize) -> Result<(), DecodeError> {
        // Encode
        let mut bytes = Vec::with_capacity(512);
        let mut enc = Encoder::<NoCustomTypeId>::new(&mut bytes);
        enc.write_size(size);

        let mut dec = Decoder::<NoCustomTypeId>::new(&bytes);
        dec.read_and_check_size(size)?;
        dec.check_end()?;
        Ok(())
    }

    #[test]
    pub fn test_vlq() {
        encode_decode_size(0x00000000).unwrap();
        encode_decode_size(0x0000007F).unwrap();
        encode_decode_size(0x00000080).unwrap();
        encode_decode_size(0x00002000).unwrap();
        encode_decode_size(0x00003FFF).unwrap();
        encode_decode_size(0x00004000).unwrap();
        encode_decode_size(0x001FFFFF).unwrap();
        encode_decode_size(0x00200000).unwrap();
        encode_decode_size(0x08000000).unwrap();
        encode_decode_size(0x0FFFFFFF).unwrap();
    }

    #[test]
    pub fn test_vlq_too_large() {
        let mut dec = Decoder::<NoCustomTypeId>::new(&[0xff, 0xff, 0xff, 0xff, 0x00]);
        assert_eq!(dec.read_size(), Err(DecodeError::SizeTooLarge));
    }

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
            12, 5, 104, 101, 108, 108, 111, // string
            32, 9, 3, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, // array
            33, 2, 9, 1, 0, 0, 0, 9, 2, 0, 0, 0, // tuple
            32, 9, 3, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, // vec
            32, 7, 2, 1, 2, // set
            32, 33, 2, 2, 7, 1, 7, 2, 2, 7, 3, 7, 4, // map
            17, 4, 83, 111, 109, 101, 1, 9, 1, 0, 0, 0, // Some<T>
            17, 4, 78, 111, 110, 101, 0, // None
            17, 2, 79, 107, 1, 9, 1, 0, 0, 0, // Ok<T>
            17, 3, 69, 114, 114, 1, 12, 5, 104, 101, 108, 108, 111, // Err<T>
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
