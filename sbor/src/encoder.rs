use crate::rust::marker::PhantomData;
use crate::rust::vec::Vec;
use crate::*;

/// Represents an error occurred during encoding.
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum EncodeError {
    MaxDepthExceeded(usize),
    SizeTooLarge { actual: usize, max_allowed: usize },
}

pub trait Encoder<X: CustomValueKind>: Sized {
    /// Consumes the Encoder and encodes the value as a full payload
    ///
    /// It starts by writing the payload prefix: It's the intention that each version of SBOR
    /// or change to the custom codecs should be given its own prefix
    #[inline]
    fn encode_payload<T: Encode<X, Self> + ?Sized>(
        mut self,
        value: &T,
        payload_prefix: u8,
    ) -> Result<(), EncodeError> {
        self.write_payload_prefix(payload_prefix)?;
        self.encode(value)
    }

    /// Encodes the value as part of a larger payload
    ///
    /// This method encodes the SBOR value's kind and then its body.
    fn encode<T: Encode<X, Self> + ?Sized>(&mut self, value: &T) -> Result<(), EncodeError> {
        value.encode_value_kind(self)?;
        self.encode_deeper_body(value)
    }

    /// Encodes the SBOR body of a child value as part of a larger payload.
    ///
    /// In many cases, you may wish to directly call `value.encode_body` instead of this method. See
    /// the below section for details.
    ///
    /// ## Direct calls and SBOR Depth
    ///
    /// In order to avoid SBOR depth differentials and disagreement about whether a payload
    /// is valid, typed codec implementations should ensure that the SBOR depth as measured
    /// during the encoding/decoding process agrees with the Value codec.
    ///
    /// Each layer of the Value counts as one depth.
    ///
    /// If the encoder you're writing is embedding a child type (and is represented as such
    /// in the Value type), then you should call `encoder.encode_body` to increment
    /// the SBOR depth tracker.
    ///
    /// You should call `value.encode_body` directly when the encoding of that type
    /// into an Value doesn't increase the SBOR depth in the encoder, that is:
    /// * When the wrapping type is invisible to the Value, ie:
    ///   * Smart pointers
    ///   * Transparent wrappers
    /// * Where the use of the inner type is invisible to Value, ie:
    ///   * Where the use of `value.encode_body` is coincidental / code re-use
    fn encode_deeper_body<T: Encode<X, Self> + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), EncodeError>;

    #[inline]
    fn write_payload_prefix(&mut self, payload_prefix: u8) -> Result<(), EncodeError> {
        self.write_byte(payload_prefix)
    }

    #[inline]
    fn write_value_kind(&mut self, ty: ValueKind<X>) -> Result<(), EncodeError> {
        self.write_byte(ty.as_u8())
    }

    fn write_discriminator(&mut self, discriminator: u8) -> Result<(), EncodeError> {
        self.write_byte(discriminator)
    }

    fn write_size(&mut self, mut size: usize) -> Result<(), EncodeError> {
        // LEB128 and 4 bytes max
        // This means the max size is 0x0FFFFFFF = 268,435,455
        if size > 0x0FFFFFFF {
            return Err(EncodeError::SizeTooLarge {
                actual: size,
                max_allowed: 0x0FFFFFFF,
            });
        }
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
}

/// An `Encoder` abstracts the logic for writing core types into a byte buffer.
pub struct VecEncoder<'a, X: CustomValueKind> {
    buf: &'a mut Vec<u8>,
    max_depth: usize,
    stack_depth: usize,
    phantom: PhantomData<X>,
}

impl<'a, X: CustomValueKind> VecEncoder<'a, X> {
    pub fn new(buf: &'a mut Vec<u8>, max_depth: usize) -> Self {
        Self {
            buf,
            stack_depth: 0,
            max_depth,
            phantom: PhantomData,
        }
    }

    #[inline]
    fn track_stack_depth_increase(&mut self) -> Result<(), EncodeError> {
        self.stack_depth += 1;
        if self.stack_depth > self.max_depth {
            return Err(EncodeError::MaxDepthExceeded(self.max_depth));
        }
        Ok(())
    }

    #[inline]
    fn track_stack_depth_decrease(&mut self) -> Result<(), EncodeError> {
        self.stack_depth -= 1;
        Ok(())
    }
}

impl<'a, X: CustomValueKind> Encoder<X> for VecEncoder<'a, X> {
    fn encode_deeper_body<T: Encode<X, Self> + ?Sized>(
        &mut self,
        value: &T,
    ) -> Result<(), EncodeError> {
        self.track_stack_depth_increase()?;
        value.encode_body(self)?;
        self.track_stack_depth_decrease()
    }

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

        encoder.encode(&Option::<u32>::None)?;
        encoder.encode(&Some(1u32))?;
        encoder.encode(&Result::<u32, String>::Ok(1u32))?;
        encoder.encode(&Result::<u32, String>::Err("hello".to_owned()))?;

        Ok(())
    }

    #[test]
    pub fn test_encoding() {
        let mut bytes = Vec::with_capacity(512);
        let mut enc = BasicEncoder::new(&mut bytes, 256);
        do_encoding(&mut enc).unwrap();

        assert_eq!(
            vec![
                33, 0, // unit (encoded as empty tuple)
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
                35, 7, 7, 2, 1, 2, 3, 4, // map
                34, 0, 0, // None
                34, 1, 1, 9, 1, 0, 0, 0, // Some<T>
                34, 0, 1, 9, 1, 0, 0, 0, // Ok<T>
                34, 1, 1, 12, 5, 104, 101, 108, 108, 111, // Err<T>
            ],
            bytes
        );
    }

    #[test]
    pub fn test_size_too_large_error() {
        const MAX_SIZE: usize = 0x0FFFFFFF; // 268,435,455, so this many bytes is about 268MB
        const TOO_LARGE_SIZE: usize = MAX_SIZE + 1;

        assert!(basic_encode(&vec![0u8; MAX_SIZE]).is_ok());
        assert!(matches!(
            basic_encode(&vec![0u8; MAX_SIZE + 1]),
            Err(EncodeError::SizeTooLarge {
                actual: TOO_LARGE_SIZE,
                max_allowed: MAX_SIZE
            })
        ));
    }

    #[test]
    pub fn test_encode_index_map_and_set() {
        let mut bytes = Vec::with_capacity(512);
        let mut encoder = BasicEncoder::new(&mut bytes, 256);
        let mut set = index_set_new::<u8>();
        set.insert(1);
        set.insert(2);
        encoder.encode(&set).unwrap();
        let mut map = index_map_new::<u8, u8>();
        map.insert(1, 2);
        map.insert(3, 4);
        encoder.encode(&map).unwrap();

        assert_eq!(
            vec![
                32, 7, 2, 1, 2, // set
                35, 7, 7, 2, 1, 2, 3, 4, // map
            ],
            bytes
        );

        let mut decoder = BasicDecoder::new(&bytes, 256);
        let set_out = decoder.decode::<IndexSet<u8>>().unwrap();
        let map_out = decoder.decode::<IndexMap<u8, u8>>().unwrap();
        decoder.check_end().unwrap();

        assert_eq!(set, set_out);
        assert_eq!(map, map_out);
    }

    #[test]
    pub fn test_encode_cow_borrowed() {
        let mut set = BTreeSet::<u8>::new();
        set.insert(1);
        set.insert(2);
        let x = crate::rust::borrow::Cow::Borrowed(&set);
        let mut bytes = Vec::with_capacity(512);
        let mut encoder = BasicEncoder::new(&mut bytes, 256);
        encoder.encode(&x).unwrap();
        assert_eq!(bytes, vec![32, 7, 2, 1, 2]) // Same as set above
    }

    #[test]
    pub fn test_encode_cow_owned() {
        use crate::rust::borrow::Cow;
        let x: Cow<u8> = Cow::Owned(5u8);
        let mut bytes = Vec::with_capacity(512);
        let mut encoder = BasicEncoder::new(&mut bytes, 256);
        encoder.encode(&x).unwrap();
        assert_eq!(bytes, vec![7, 5])
    }

    #[test]
    pub fn test_encode_box() {
        let x = Box::new(5u8);
        let mut bytes = Vec::with_capacity(512);
        let mut encoder = BasicEncoder::new(&mut bytes, 256);
        encoder.encode(&x).unwrap();
        assert_eq!(bytes, vec![7, 5])
    }

    #[test]
    pub fn test_encode_rc() {
        let x = crate::rust::rc::Rc::new(5u8);
        let mut bytes = Vec::with_capacity(512);
        let mut encoder = BasicEncoder::new(&mut bytes, 256);
        encoder.encode(&x).unwrap();
        assert_eq!(bytes, vec![7, 5])
    }

    #[test]
    pub fn test_encode_ref_cell() {
        let x = crate::rust::cell::RefCell::new(5u8);
        let mut bytes = Vec::with_capacity(512);
        let mut encoder = BasicEncoder::new(&mut bytes, 256);
        encoder.encode(&x).unwrap();
        assert_eq!(bytes, vec![7, 5])
    }
}
