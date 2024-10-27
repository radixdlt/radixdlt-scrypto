use crate::rust::prelude::*;
use crate::value_kind::*;
use crate::*;

/// Represents an error occurred during decoding.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Sbor)]
pub enum DecodeError {
    ExtraTrailingBytes(usize),

    BufferUnderflow { required: usize, remaining: usize },

    UnexpectedPayloadPrefix { expected: u8, actual: u8 },

    UnexpectedValueKind { expected: u8, actual: u8 },

    UnexpectedCustomValueKind { actual: u8 },

    UnexpectedSize { expected: usize, actual: usize },

    UnexpectedDiscriminator { expected: u8, actual: u8 },

    UnknownValueKind(u8),

    UnknownDiscriminator(u8),

    InvalidBool(u8),

    InvalidUtf8,

    InvalidSize,

    MaxDepthExceeded(usize),

    DuplicateKey,

    InvalidCustomValue, // TODO: generify custom error codes
}

pub trait Decoder<X: CustomValueKind>: Sized {
    /// Consumes the Decoder and decodes the value as a full payload
    ///
    /// This includes a check of the payload prefix byte: It's the intention that each version of SBOR
    /// or change to the custom codecs should be given its own prefix
    #[inline]
    fn decode_payload<T: Decode<X, Self>>(mut self, expected_prefix: u8) -> Result<T, DecodeError> {
        self.read_and_check_payload_prefix(expected_prefix)?;
        let value = self.decode()?;
        self.check_end()?;
        Ok(value)
    }

    /// Decodes the value as part of a larger payload
    ///
    /// This method decodes the SBOR value's kind, and then its body.
    fn decode<T: Decode<X, Self>>(&mut self) -> Result<T, DecodeError> {
        let value_kind = self.read_value_kind()?;
        self.decode_deeper_body_with_value_kind(value_kind)
    }

    /// Decodes the SBOR body of a child value as part of a larger payload.
    ///
    /// In many cases, you may wish to directly call `T::decode_body_with_value_kind` instead of this method.
    /// See the below section for details.
    ///
    /// ## Direct calls and SBOR Depth
    ///
    /// In order to avoid SBOR depth differentials and disagreement about whether a payload
    /// is valid, typed codec implementations should ensure that the SBOR depth as measured
    /// during the encoding/decoding process agrees with the SBOR [`Value`] codec.
    ///
    /// Each layer of the SBOR `Value` counts as one depth.
    ///
    /// If the decoder you're writing is embedding a child type (and is represented as such
    /// in the SBOR `Value` type), then you should call `decoder.decode_body_with_value_kind` to increment
    /// the SBOR depth tracker.
    ///
    /// You should call `T::decode_body_with_value_kind` directly when the decoding of that type
    /// into an SBOR `Value` doesn't increase the SBOR depth in the decoder, that is:
    /// * When the wrapping type is invisible to the SBOR `Value`, ie:
    ///   * Smart pointers
    ///   * Transparent wrappers
    /// * Where the use of the inner type is invisible to SBOR `Value`, ie:
    ///   * Where the use of `T::decode_body_with_value_kind` is coincidental / code re-use
    fn decode_deeper_body_with_value_kind<T: Decode<X, Self>>(
        &mut self,
        value_kind: ValueKind<X>,
    ) -> Result<T, DecodeError>;

    #[inline]
    fn read_value_kind(&mut self) -> Result<ValueKind<X>, DecodeError> {
        let id = self.read_byte()?;
        ValueKind::from_u8(id).ok_or(DecodeError::UnknownValueKind(id))
    }

    #[inline]
    fn read_discriminator(&mut self) -> Result<u8, DecodeError> {
        self.read_byte()
    }

    fn read_size(&mut self) -> Result<usize, DecodeError> {
        // LEB128 and 4 bytes max
        let mut size = 0usize;
        let mut shift = 0;
        let mut byte;
        loop {
            byte = self.read_byte()?;
            size |= ((byte & 0x7F) as usize) << shift;
            if byte < 0x80 {
                break;
            }
            shift += 7;
            if shift >= 28 {
                return Err(DecodeError::InvalidSize);
            }
        }

        // The last byte should not be zero, unless the size is zero
        if byte == 0 && shift != 0 {
            return Err(DecodeError::InvalidSize);
        }

        Ok(size)
    }

    #[inline]
    fn check_preloaded_value_kind(
        &self,
        value_kind: ValueKind<X>,
        expected: ValueKind<X>,
    ) -> Result<ValueKind<X>, DecodeError> {
        if value_kind == expected {
            Ok(value_kind)
        } else {
            Err(DecodeError::UnexpectedValueKind {
                actual: value_kind.as_u8(),
                expected: expected.as_u8(),
            })
        }
    }

    #[inline]
    fn read_expected_discriminator(
        &mut self,
        expected_discriminator: u8,
    ) -> Result<(), DecodeError> {
        let actual = self.read_discriminator()?;
        if actual == expected_discriminator {
            Ok(())
        } else {
            Err(DecodeError::UnexpectedDiscriminator {
                actual,
                expected: expected_discriminator,
            })
        }
    }

    #[inline]
    fn read_and_check_payload_prefix(&mut self, expected_prefix: u8) -> Result<(), DecodeError> {
        let actual_payload_prefix = self.read_byte()?;
        if actual_payload_prefix != expected_prefix {
            return Err(DecodeError::UnexpectedPayloadPrefix {
                actual: actual_payload_prefix,
                expected: expected_prefix,
            });
        }

        Ok(())
    }

    #[inline]
    fn read_and_check_value_kind(
        &mut self,
        expected: ValueKind<X>,
    ) -> Result<ValueKind<X>, DecodeError> {
        let value_kind = self.read_value_kind()?;
        self.check_preloaded_value_kind(value_kind, expected)
    }

    #[inline]
    fn read_and_check_size(&mut self, expected: usize) -> Result<(), DecodeError> {
        let len = self.read_size()?;
        if len != expected {
            return Err(DecodeError::UnexpectedSize {
                expected,
                actual: len,
            });
        }

        Ok(())
    }

    fn check_end(&self) -> Result<(), DecodeError>;

    fn read_byte(&mut self) -> Result<u8, DecodeError>;

    fn read_slice(&mut self, n: usize) -> Result<&[u8], DecodeError>;

    // Advanced methods - mostly for use by traversers

    fn peek_remaining(&self) -> &[u8];

    fn get_depth_limit(&self) -> usize;

    fn get_stack_depth(&self) -> usize;

    fn get_offset(&self) -> usize;

    fn peek_value_kind(&self) -> Result<ValueKind<X>, DecodeError> {
        let id = self.peek_byte()?;
        ValueKind::from_u8(id).ok_or(DecodeError::UnknownValueKind(id))
    }

    fn peek_byte(&self) -> Result<u8, DecodeError>;
}

pub trait BorrowingDecoder<'de, X: CustomValueKind>: Decoder<X> {
    fn read_slice_from_payload(&mut self, n: usize) -> Result<&'de [u8], DecodeError>;
}

/// A `Decoder` abstracts the logic for decoding basic types.
pub struct VecDecoder<'de, X: CustomValueKind> {
    input: &'de [u8],
    offset: usize,
    stack_depth: usize,
    max_depth: usize,
    phantom: PhantomData<X>,
}

impl<'de, X: CustomValueKind> VecDecoder<'de, X> {
    pub fn new(input: &'de [u8], max_depth: usize) -> Self {
        Self {
            input,
            offset: 0,
            stack_depth: 0,
            max_depth,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn get_input_slice(&self) -> &'de [u8] {
        &self.input
    }

    #[inline]
    fn require_remaining(&self, n: usize) -> Result<(), DecodeError> {
        if self.remaining_bytes() < n {
            Err(DecodeError::BufferUnderflow {
                required: n,
                remaining: self.remaining_bytes(),
            })
        } else {
            Ok(())
        }
    }

    #[inline]
    fn remaining_bytes(&self) -> usize {
        self.input.len() - self.offset
    }

    #[inline]
    pub fn track_stack_depth_increase(&mut self) -> Result<(), DecodeError> {
        self.stack_depth += 1;
        if self.stack_depth > self.max_depth {
            return Err(DecodeError::MaxDepthExceeded(self.max_depth));
        }
        Ok(())
    }

    #[inline]
    pub fn track_stack_depth_decrease(&mut self) -> Result<(), DecodeError> {
        self.stack_depth -= 1;
        Ok(())
    }
}

impl<'de, X: CustomValueKind> Decoder<X> for VecDecoder<'de, X> {
    fn decode_deeper_body_with_value_kind<T: Decode<X, Self>>(
        &mut self,
        value_kind: ValueKind<X>,
    ) -> Result<T, DecodeError> {
        self.track_stack_depth_increase()?;
        let decoded = T::decode_body_with_value_kind(self, value_kind)?;
        self.track_stack_depth_decrease()?;
        Ok(decoded)
    }

    #[inline]
    fn read_byte(&mut self) -> Result<u8, DecodeError> {
        self.require_remaining(1)?;
        let result = self.input[self.offset];
        self.offset += 1;
        Ok(result)
    }

    #[inline]
    fn read_slice(&mut self, n: usize) -> Result<&[u8], DecodeError> {
        // Note - the Decoder trait can't capture all the lifetimes correctly
        self.read_slice_from_payload(n)
    }

    #[inline]
    fn check_end(&self) -> Result<(), DecodeError> {
        let n = self.remaining_bytes();
        if n != 0 {
            Err(DecodeError::ExtraTrailingBytes(n))
        } else {
            Ok(())
        }
    }

    #[inline]
    fn peek_remaining(&self) -> &[u8] {
        &self.input[self.offset..]
    }

    #[inline]
    fn get_depth_limit(&self) -> usize {
        self.max_depth
    }

    #[inline]
    fn get_stack_depth(&self) -> usize {
        self.stack_depth
    }

    #[inline]
    fn get_offset(&self) -> usize {
        self.offset
    }

    #[inline]
    fn peek_byte(&self) -> Result<u8, DecodeError> {
        self.require_remaining(1)?;
        let result = self.input[self.offset];
        Ok(result)
    }
}

impl<'de, X: CustomValueKind> BorrowingDecoder<'de, X> for VecDecoder<'de, X> {
    #[inline]
    fn read_slice_from_payload(&mut self, n: usize) -> Result<&'de [u8], DecodeError> {
        self.require_remaining(n)?;
        let slice = &self.input[self.offset..self.offset + n];
        self.offset += n;
        Ok(slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal_prelude::*;

    fn encode_decode_size(size: usize) -> Result<(), DecodeError> {
        // Encode
        let mut bytes = Vec::with_capacity(512);
        let mut enc = BasicEncoder::new(&mut bytes, 256);
        enc.write_size(size).unwrap();

        let mut dec = BasicDecoder::new(&bytes, 256);
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
        let mut dec = BasicDecoder::new(&[0xff, 0xff, 0xff, 0xff, 0x00], 256);
        assert_eq!(dec.read_size(), Err(DecodeError::InvalidSize));
    }

    fn assert_decoding(dec: &mut BasicDecoder) {
        dec.decode::<()>().unwrap();
        assert_eq!(true, dec.decode::<bool>().unwrap());
        assert_eq!(1, dec.decode::<i8>().unwrap());
        assert_eq!(1, dec.decode::<i16>().unwrap());
        assert_eq!(1, dec.decode::<i32>().unwrap());
        assert_eq!(1, dec.decode::<i64>().unwrap());
        assert_eq!(1, dec.decode::<i128>().unwrap());
        assert_eq!(1, dec.decode::<u8>().unwrap());
        assert_eq!(1, dec.decode::<u16>().unwrap());
        assert_eq!(1, dec.decode::<u32>().unwrap());
        assert_eq!(1, dec.decode::<u64>().unwrap());
        assert_eq!(1, dec.decode::<u128>().unwrap());
        assert_eq!("hello", dec.decode::<String>().unwrap());

        assert_eq!([1u32, 2u32, 3u32], dec.decode::<[u32; 3]>().unwrap());
        assert_eq!((1u32, 2u32), dec.decode::<(u32, u32)>().unwrap());

        assert_eq!(vec![1u32, 2u32, 3u32], dec.decode::<Vec<u32>>().unwrap());
        let mut set = BTreeSet::<u8>::new();
        set.insert(1);
        set.insert(2);
        assert_eq!(set, dec.decode::<BTreeSet<u8>>().unwrap());
        let mut map = BTreeMap::<u8, u8>::new();
        map.insert(1, 2);
        map.insert(3, 4);
        assert_eq!(map, dec.decode::<BTreeMap<u8, u8>>().unwrap());

        assert_eq!(None, dec.decode::<Option<u32>>().unwrap());
        assert_eq!(Some(1u32), dec.decode::<Option<u32>>().unwrap());
        assert_eq!(Ok(1u32), dec.decode::<Result<u32, String>>().unwrap());
        assert_eq!(
            Err("hello".to_owned()),
            dec.decode::<Result<u32, String>>().unwrap()
        );
    }

    #[test]
    pub fn test_decoding() {
        let bytes = vec![
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
        ];
        let mut dec = BasicDecoder::new(&bytes, 256);
        assert_decoding(&mut dec);
    }

    #[test]
    pub fn test_decode_box() {
        let bytes = vec![7u8, 5u8];
        let mut dec = BasicDecoder::new(&bytes, 256);
        let x = dec.decode::<Box<u8>>().unwrap();
        assert_eq!(Box::new(5u8), x);
    }

    #[test]
    pub fn test_decode_rc() {
        let bytes = vec![7u8, 5u8];
        let mut dec = BasicDecoder::new(&bytes, 256);
        let x = dec.decode::<Rc<u8>>().unwrap();
        assert_eq!(Rc::new(5u8), x);
    }

    #[test]
    pub fn test_decode_ref_cell() {
        let bytes = vec![7u8, 5u8];
        let mut dec = BasicDecoder::new(&bytes, 256);
        let x = dec.decode::<RefCell<u8>>().unwrap();
        assert_eq!(RefCell::new(5u8), x);
    }

    #[test]
    pub fn test_decode_duplicates_in_set() {
        let input_with_duplicates = vec![5u16, 5u16];
        let payload = basic_encode(&input_with_duplicates).unwrap();
        // Check decode works into vec and BasicValue - which represent sets as arrays
        assert_eq!(
            basic_decode::<Vec<u16>>(&payload),
            Ok(input_with_duplicates)
        );
        assert_matches!(basic_decode::<BasicValue>(&payload), Ok(_));
        // Decode doesn't work into any typed sets
        assert_eq!(
            basic_decode::<HashSet<u16>>(&payload),
            Err(DecodeError::DuplicateKey)
        );
        assert_eq!(
            basic_decode::<BTreeSet<u16>>(&payload),
            Err(DecodeError::DuplicateKey)
        );
        assert_eq!(
            basic_decode::<IndexSet<u16>>(&payload),
            Err(DecodeError::DuplicateKey)
        );
    }

    #[test]
    pub fn test_decode_duplicates_in_map() {
        let input_with_duplicates = BasicValue::Map {
            key_value_kind: ValueKind::U16,
            value_value_kind: ValueKind::String,
            entries: vec![
                (
                    BasicValue::U16 { value: 5 },
                    BasicValue::String {
                        value: "test".to_string(),
                    },
                ),
                (
                    BasicValue::U16 { value: 5 },
                    BasicValue::String {
                        value: "test2".to_string(),
                    },
                ),
            ],
        };
        let payload = basic_encode(&input_with_duplicates).unwrap();
        // Check decode works into BasicValue - which represent sets as arrays of (k, v) tuples
        assert_matches!(basic_decode::<BasicValue>(&payload), Ok(_));
        // Decode doesn't work into any typed maps
        assert_eq!(
            basic_decode::<HashMap<u16, String>>(&payload),
            Err(DecodeError::DuplicateKey)
        );
        assert_eq!(
            basic_decode::<BTreeMap<u16, String>>(&payload),
            Err(DecodeError::DuplicateKey)
        );
        assert_eq!(
            basic_decode::<IndexMap<u16, String>>(&payload),
            Err(DecodeError::DuplicateKey)
        );
    }

    #[derive(sbor::Categorize, sbor::Encode, sbor::Decode, PartialEq, Eq, Debug)]
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
        let mut encoder = BasicEncoder::new(&mut bytes, 256);
        encoder.encode(&value1).unwrap();

        let mut decoder = BasicDecoder::new(&bytes, 256);
        let value2 = decoder.decode::<[NFA; 2]>().unwrap();
        assert_eq!(value1, value2);
    }

    #[test]
    pub fn test_invalid_size() {
        assert_eq!(
            BasicDecoder::new(&[0x80], 256).read_size(),
            Err(DecodeError::BufferUnderflow {
                required: 1,
                remaining: 0
            })
        );

        // Trailing zeros
        // LE: [0, 0]
        assert_eq!(
            BasicDecoder::new(&[0x80, 00], 256).read_size(),
            Err(DecodeError::InvalidSize)
        );
        // LE: [0, 1, 0]
        assert_eq!(
            BasicDecoder::new(&[0x80, 0x81, 0x00], 256).read_size(),
            Err(DecodeError::InvalidSize)
        );
        assert_eq!(
            BasicDecoder::new(&[0x80, 0x01], 256).read_size(),
            Ok(1 << 7)
        );

        // Out of range
        assert_eq!(
            BasicDecoder::new(&[0xFF, 0xFF, 0xFF, 0x80], 256).read_size(),
            Err(DecodeError::InvalidSize)
        );
        assert_eq!(
            BasicDecoder::new(&[0xFF, 0xFF, 0xFF, 0xFF], 256).read_size(),
            Err(DecodeError::InvalidSize)
        );
    }

    #[test]
    pub fn test_valid_size() {
        assert_eq!(BasicDecoder::new(&[00], 256).read_size(), Ok(0));
        assert_eq!(BasicDecoder::new(&[123], 256).read_size(), Ok(123));
        assert_eq!(
            BasicDecoder::new(&[0xff, 0xff, 0xff, 0x7f], 256).read_size(),
            Ok(0x0fffffff)
        );

        let delta = 0x1fffff;
        let ranges = [
            0..delta,                                       /* low */
            0x0fffffff / 2 - delta..0x0fffffff / 2 + delta, /* mid */
            0x0fffffff - delta..0x0fffffff,                 /* high */
        ];
        for range in ranges {
            for i in range {
                let mut vec = Vec::new();
                let mut enc = BasicEncoder::new(&mut vec, 256);
                enc.write_size(i).unwrap();
                let mut dec = BasicDecoder::new(&vec, 256);
                assert_eq!(dec.read_size(), Ok(i));
                assert_eq!(dec.remaining_bytes(), 0);
            }
        }
    }
}
