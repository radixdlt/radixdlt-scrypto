use crate::rust::marker::PhantomData;
use crate::rust::vec::Vec;
use crate::*;

/// Represents an error occurred during encoding.
#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum EncodeError {
    MaxDepthExceeded(u8),
    SizeTooLarge { actual: usize, max_allowed: usize },
}

pub trait Encoder<X: CustomTypeId>: Sized {
    #[inline]
    fn encode_payload<T: Encode<X, Self> + ?Sized>(mut self, value: &T) -> Result<(), EncodeError> {
        self.encode(value)
    }

    fn encode<T: Encode<X, Self> + ?Sized>(&mut self, value: &T) -> Result<(), EncodeError> {
        value.encode_type_id(self)?;
        self.encode_body(value)
    }

    fn encode_body<T: Encode<X, Self> + ?Sized>(&mut self, value: &T) -> Result<(), EncodeError> {
        self.track_stack_depth_increase()?;
        value.encode_body(self)?;
        self.track_stack_depth_decrease()
    }

    #[inline]
    fn write_type_id(&mut self, ty: SborTypeId<X>) -> Result<(), EncodeError> {
        self.write_byte(ty.as_u8())
    }

    fn write_discriminator(&mut self, discriminator: &str) -> Result<(), EncodeError> {
        self.write_size(discriminator.len())?;
        self.write_slice(discriminator.as_bytes())
    }

    fn write_size(&mut self, mut size: usize) -> Result<(), EncodeError> {
        // LEB128 and 4 bytes max
        assert!(size <= 0x0FFFFFFF); // 268,435,455
        loop {
            let seven_bits = size & 0x7F;
            size = size >> 7;
            if size == 0 {
                self.write_byte(seven_bits as u8)?;
                break;
            } else {
                self.write_byte(seven_bits as u8 | 0x80)?;
            }
        }
        Ok(())
    }

    fn write_byte(&mut self, n: u8) -> Result<(), EncodeError>;

    fn write_slice(&mut self, slice: &[u8]) -> Result<(), EncodeError>;

    fn track_stack_depth_increase(&mut self) -> Result<(), EncodeError>;

    fn track_stack_depth_decrease(&mut self) -> Result<(), EncodeError>;
}

/// An `Encoder` abstracts the logic for writing core types into a byte buffer.
pub struct VecEncoder<'a, X: CustomTypeId, const MAX_DEPTH: u8> {
    buf: &'a mut Vec<u8>,
    stack_depth: u8,
    phantom: PhantomData<X>,
}

impl<'a, X: CustomTypeId, const MAX_DEPTH: u8> VecEncoder<'a, X, MAX_DEPTH> {
    pub fn new(buf: &'a mut Vec<u8>) -> Self {
        Self {
            buf,
            stack_depth: 0,
            phantom: PhantomData,
        }
    }
}

impl<'a, X: CustomTypeId, const MAX_DEPTH: u8> Encoder<X> for VecEncoder<'a, X, MAX_DEPTH> {
    #[inline]
    fn write_byte(&mut self, n: u8) -> Result<(), EncodeError> {
        self.buf.push(n);
        Ok(())
    }

    #[inline]
    fn write_slice(&mut self, slice: &[u8]) -> Result<(), EncodeError> {
        self.buf.extend(slice);
        Ok(())
    }

    #[inline]
    fn track_stack_depth_increase(&mut self) -> Result<(), EncodeError> {
        self.stack_depth += 1;
        if self.stack_depth > MAX_DEPTH {
            return Err(EncodeError::MaxDepthExceeded(MAX_DEPTH));
        }
        Ok(())
    }

    #[inline]
    fn track_stack_depth_decrease(&mut self) -> Result<(), EncodeError> {
        self.stack_depth -= 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::borrow::ToOwned;
    use crate::rust::boxed::Box;
    use crate::rust::collections::*;
    use crate::rust::string::String;
    use crate::rust::vec;
    use crate::BasicEncoder;

    fn do_encoding(encoder: &mut BasicEncoder) -> Result<(), EncodeError> {
        encoder.encode(&())?;
        encoder.encode(&true)?;
        encoder.encode(&1i8)?;
        encoder.encode(&1i16)?;
        encoder.encode(&1i32)?;
        encoder.encode(&1i64)?;
        encoder.encode(&1i128)?;
        encoder.encode(&1u8)?;
        encoder.encode(&1u16)?;
        encoder.encode(&1u32)?;
        encoder.encode(&1u64)?;
        encoder.encode(&1u128)?;
        encoder.encode("hello")?;

        encoder.encode(&[1u32, 2u32, 3u32])?;
        encoder.encode(&(1u32, 2u32))?;

        encoder.encode(&vec![1u32, 2u32, 3u32])?;
        let mut set = BTreeSet::<u8>::new();
        set.insert(1);
        set.insert(2);
        encoder.encode(&set)?;
        let mut map = BTreeMap::<u8, u8>::new();
        map.insert(1, 2);
        map.insert(3, 4);
        encoder.encode(&map)?;

        encoder.encode(&Some(1u32))?;
        encoder.encode(&Option::<u32>::None)?;
        encoder.encode(&Result::<u32, String>::Ok(1u32))?;
        encoder.encode(&Result::<u32, String>::Err("hello".to_owned()))?;

        Ok(())
    }

    #[test]
    pub fn test_encoding() {
        let mut bytes = Vec::with_capacity(512);
        let mut enc = BasicEncoder::new(&mut bytes);
        do_encoding(&mut enc).unwrap();

        assert_eq!(
            vec![
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
            ],
            bytes
        );
    }

    #[test]
    pub fn test_encode_cow_borrowed() {
        let mut set = BTreeSet::<u8>::new();
        set.insert(1);
        set.insert(2);
        let x = crate::rust::borrow::Cow::Borrowed(&set);
        let mut bytes = Vec::with_capacity(512);
        let mut encoder = BasicEncoder::new(&mut bytes);
        encoder.encode(&x).unwrap();
        assert_eq!(bytes, vec![32, 7, 2, 1, 2]) // Same as set above
    }

    #[test]
    pub fn test_encode_cow_owned() {
        use crate::rust::borrow::Cow;
        let x: Cow<u8> = Cow::Owned(5u8);
        let mut bytes = Vec::with_capacity(512);
        let mut encoder = BasicEncoder::new(&mut bytes);
        encoder.encode(&x).unwrap();
        assert_eq!(bytes, vec![7, 5])
    }

    #[test]
    pub fn test_encode_box() {
        let x = Box::new(5u8);
        let mut bytes = Vec::with_capacity(512);
        let mut encoder = BasicEncoder::new(&mut bytes);
        encoder.encode(&x).unwrap();
        assert_eq!(bytes, vec![7, 5])
    }

    #[test]
    pub fn test_encode_rc() {
        let x = crate::rust::rc::Rc::new(5u8);
        let mut bytes = Vec::with_capacity(512);
        let mut encoder = BasicEncoder::new(&mut bytes);
        encoder.encode(&x).unwrap();
        assert_eq!(bytes, vec![7, 5])
    }

    #[test]
    pub fn test_encode_ref_cell() {
        let x = crate::rust::cell::RefCell::new(5u8);
        let mut bytes = Vec::with_capacity(512);
        let mut encoder = BasicEncoder::new(&mut bytes);
        encoder.encode(&x).unwrap();
        assert_eq!(bytes, vec![7, 5])
    }
}
