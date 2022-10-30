use super::*;

// TODO - turn Decoder into a trait

/// Represents an error occurred during decoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {

    InvalidPayloadPrefix(u8),
    InvalidTypeEncodingClass(u8),

    InvalidInterpretation { expected: u8, actual: u8 },

    ExpectedRawBytes { actual: TypeEncodingClass },
    ExpectedProduct { actual: TypeEncodingClass },
    ExpectedSum { actual: TypeEncodingClass },
    ExpectedList { actual: TypeEncodingClass },
    ExpectedMap { actual: TypeEncodingClass },


    UnexpectedProductLengthType,
    UnexpectedU8Length { expected: u8, actual: u8 },
    UnexpectedU16Length { expected: u16, actual: u16 },
    UnexpectedLength { expected: usize, actual: usize },
    LengthInvalidForArchitecture,



    InvalidDiscriminatorType(SumTypeDiscriminator),
    InvalidU8Discriminator(u8),
    InvalidU16Discriminator(u16),
    InvalidU32Discriminator(u32),
    InvalidU64Discriminator(u64),
    InvalidStringDiscriminator(String),
    InvalidAnyDiscriminator,

    // Unused
    Underflow { required: usize, remaining: usize },

    InvalidType { expected: Option<u8>, actual: u8 },

    InvalidName { expected: String, actual: String },

    InvalidLength { expected: usize, actual: usize },

    InvalidIndex(u8),

    InvalidEnumVariant(String),

    InvalidUnit(u8),

    InvalidBool(u8),

    InvalidUtf8,

    NotAllBytesUsed(usize),

    CustomError(String),
}

/// A `Decoder` abstracts the logic for reading from a byte buffer.
pub struct Decoder<'a> {
    input: &'a [u8],
    offset: usize,
    decoder_stack_depth: u8,
}

impl<'a> Decoder<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            offset: 0,
            decoder_stack_depth: 0,
        }
    }

    /// For decoding a full payload
    pub fn decode_payload<T: Decodable>(mut self) -> Result<T, DecodeError> {
        let prefix_byte = self.read_u8()?;
        if prefix_byte != SBOR_V1_PREFIX_BYTE {
            return Err(DecodeError::InvalidPayloadPrefix(prefix_byte));
        }
        let value = self.decode()?;
        // TODO check bytes fully used
        Ok(value)
    }

    /// For decoding the type from the buffer
    pub fn decode<T: Decodable>(&mut self) -> Result<T, DecodeError> {
        T::check_interpretation(self.read_interpretation()?)?;
        T::decode_value_from(self)
    }

    #[inline]
    pub fn read_raw_bytes(&mut self) -> Result<&'a [u8], DecodeError> {
        let length_type = Self::expect_raw_bytes_type(self.read_type_encoding_class()?)?;
        let length = self.read_length(length_type)?;
        Ok(self.read_variable_bytes(length)?)
    }

    #[inline]
    pub fn read_raw_bytes_fixed_length<const N: usize>(&mut self) -> Result<&'a [u8], DecodeError> {
        let length_type = Self::expect_raw_bytes_type(self.read_type_encoding_class()?)?;
        let length = self.read_length(length_type)?;
        if length != N {
            return Err(DecodeError::InvalidLength { expected: N, actual: length });
        }
        Ok(self.read_fixed_bytes::<N>()?)
    }

    #[inline]
    pub fn read_raw_bytes_fixed_length_array<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        let length_type = Self::expect_raw_bytes_type(self.read_type_encoding_class()?)?;
        let length = self.read_length(length_type)?;
        if length != N {
            return Err(DecodeError::InvalidLength { expected: N, actual: length });
        }
        Ok(self.read_fixed_bytes_array::<N>()?)
    }

    #[inline]
    pub fn read_product_type_header_u8_length(&mut self, expected_len: u8) -> Result<(), DecodeError> {
        let length_type = Self::expect_product_type(self.read_type_encoding_class()?)?;
        let read_length = match length_type {
            ProductTypeLength::U8 => self.read_u8()?,
            ProductTypeLength::U16 => return Err(DecodeError::UnexpectedProductLengthType)?,
        };
        if read_length != expected_len {
            return Err(DecodeError::UnexpectedU8Length { expected: expected_len, actual: read_length });
        }
        Ok(())
    }

    #[inline]
    pub fn read_product_type_header_u16_length(&mut self, expected_len: u16) -> Result<(), DecodeError> {
        let length_type = Self::expect_product_type(self.read_type_encoding_class()?)?;
        let read_length = match length_type {
            ProductTypeLength::U8 => return Err(DecodeError::UnexpectedProductLengthType),
            ProductTypeLength::U16 => self.read_u16()?,
        };
        if read_length != expected_len {
            return Err(DecodeError::UnexpectedU16Length { expected: expected_len, actual: read_length });
        }
        Ok(())
    }

    #[inline]
    pub fn read_sum_type_discriminator_header(&mut self) -> Result<SumTypeDiscriminator, DecodeError> {
        let discriminator = Self::expect_sum_type(self.read_type_encoding_class()?)?;
        Ok(discriminator)
    }

    #[inline]
    pub fn read_sum_type_u8_discriminator(&mut self) -> Result<u8, DecodeError> {
        self.read_u8()
    }

    #[inline]
    pub fn read_sum_type_u16_discriminator(&mut self) -> Result<u16, DecodeError> {
        self.read_u16()
    }

    #[inline]
    pub fn read_sum_type_u32_discriminator(&mut self) -> Result<u32, DecodeError> {
        self.read_u32()
    }

    #[inline]
    pub fn read_sum_type_u64_discriminator(&mut self) -> Result<u64, DecodeError> {
        self.read_u64()
    }

    #[inline]
    pub fn read_sum_type_any_discriminator<T: Decodable>(&mut self) -> Result<T, DecodeError> {
        self.decode()
    }

    #[inline]
    fn read_type_encoding_class(&mut self) -> Result<TypeEncodingClass, DecodeError> {
        let byte = self.read_single_byte()?;
        let class = TypeEncodingClass::try_from_byte(byte)
            .ok_or_else(|| DecodeError::InvalidTypeEncodingClass(byte))?;
        Ok(class)
    }

    #[inline]
    fn read_interpretation(&mut self) -> Result<u8, DecodeError> {
        self.read_single_byte()
    }

    #[inline]
    fn expect_raw_bytes_type(class: TypeEncodingClass) -> Result<LengthType, DecodeError> {
        match class {
            TypeEncodingClass::RawBytes(val) => Ok(val),
            _ => Err(DecodeError::ExpectedRawBytes { actual: class })
        }
    }

    #[inline]
    fn expect_product_type(class: TypeEncodingClass) -> Result<ProductTypeLength, DecodeError> {
        match class {
            TypeEncodingClass::ProductType(val) => Ok(val),
            _ => Err(DecodeError::ExpectedProduct { actual: class })
        }
    }

    #[inline]
    fn expect_sum_type(class: TypeEncodingClass) -> Result<SumTypeDiscriminator, DecodeError> {
        match class {
            TypeEncodingClass::SumType(val) => Ok(val),
            _ => Err(DecodeError::ExpectedSum { actual: class })
        }
    }

    #[inline]
    fn expect_list_type(class: TypeEncodingClass) -> Result<LengthType, DecodeError> {
        match class {
            TypeEncodingClass::List(val) => Ok(val),
            _ => Err(DecodeError::ExpectedList { actual: class })
        }
    }

    #[inline]
    fn expect_map_type(class: TypeEncodingClass) -> Result<LengthType, DecodeError> {
        match class {
            TypeEncodingClass::Map(val) => Ok(val),
            _ => Err(DecodeError::ExpectedMap { actual: class })
        }
    }

    #[inline]
    fn read_length(&mut self, length_type: LengthType) -> Result<usize, DecodeError> {
        Ok(match length_type {
            LengthType::U8 => self.read_u8()?
                .try_into()
                .map_err(|_| DecodeError::LengthInvalidForArchitecture)?,
            LengthType::U16 => self.read_u16()?
                .try_into()
                .map_err(|_| DecodeError::LengthInvalidForArchitecture)?,
            LengthType::U32 => self.read_u32()?
                .try_into()
                .map_err(|_| DecodeError::LengthInvalidForArchitecture)?,
            LengthType::U64 => self.read_u64()?
                .try_into()
                .map_err(|_| DecodeError::LengthInvalidForArchitecture)?
        })
    }

    #[inline]
    fn read_u8(&mut self) -> Result<u8, DecodeError> {
        Ok(u8::from_le(self.read_single_byte()?))
    }

    #[inline]
    fn read_u16(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_le_bytes(self.read_fixed_bytes_array::<2>()?))
    }

    #[inline]
    fn read_u32(&mut self) -> Result<u32, DecodeError> {
        Ok(u32::from_le_bytes(self.read_fixed_bytes_array::<4>()?))
    }

    #[inline]
    fn read_u64(&mut self) -> Result<u64, DecodeError> {
        Ok(u64::from_le_bytes(self.read_fixed_bytes_array::<8>()?))
    }

    #[inline]
    fn read_single_byte(&mut self) -> Result<u8, DecodeError> {
        // TODO - add require
        let result = self.input[self.offset];
        self.offset += 1;
        Ok(result)
    }

    #[inline]
    fn read_fixed_bytes<const N: usize>(&mut self) -> Result<&'a [u8], DecodeError> {
        // TODO - add require
        let slice = &self.input[self.offset..self.offset + N];
        self.offset += N;
        Ok(slice)
    }

    #[inline]
    fn read_fixed_bytes_array<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        // TODO - add require
        let mut bytes_out = [0u8; N];
        bytes_out.copy_from_slice(self.read_fixed_bytes::<N>()?);
        Ok(bytes_out)
    }

    #[inline]
    fn read_variable_bytes(&mut self, count: usize) -> Result<&'a [u8], DecodeError> {
        // TODO - add require
        let slice = &self.input[self.offset..self.offset + count];
        self.offset += count;
        Ok(slice)
    }

    #[inline]
    fn track_encode_depth_increase(&mut self) {
        self.decoder_stack_depth += 1;
        if self.decoder_stack_depth > DEFAULT_MAX_ENCODING_DEPTH {
            panic!("Max encoding depth reached encoding SBOR");
        }
    }

    #[inline]
    fn track_encode_depth_decrease(&mut self) {
        self.decoder_stack_depth -= 1;
    }
}
