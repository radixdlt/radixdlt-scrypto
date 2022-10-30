use super::*;

impl ConstInterpretation for () {
    const INTERPRETATION: u8 = DefaultInterpretations::UNIT;
}

impl Encode for () {
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_product_type_header_u8_length(0);
    }
}

impl Decode for () {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        decoder.read_product_type_header_u8_length(0)
    }
}

impl ConstInterpretation for bool {
    const INTERPRETATION: u8 = DefaultInterpretations::BOOLEAN;
}

impl Encode for bool {
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_raw_bytes(
            if *self {
                &[1]
            } else {
                &[0]
            }
        )
    }
}

impl Decode for bool {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let bytes = decoder.read_raw_bytes_fixed_length_array::<1>()?;
        match bytes {
            [0] => Ok(false),
            [1] => Ok(true),
            [other] => Err(DecodeError::InvalidBool(other)),
        }
    }
}

macro_rules! sbor_int {
    ($type:ident, $interpretation:expr, $bytes_length:expr) => {
        impl ConstInterpretation for $type {
            const INTERPRETATION: u8 = $interpretation;
        }

        impl Encode for $type {
            fn encode_value(&self, encoder: &mut Encoder) {
                encoder.write_raw_bytes(
                    &(*self).to_le_bytes(),
                );
            }
        }

        impl Decode for $type {
            fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
                let bytes = decoder.read_raw_bytes_fixed_length_array::<$bytes_length>()?;
                Ok(<$type>::from_le_bytes(bytes))
            }
        }
    };
}

// Unsigned
sbor_int!(u8, DefaultInterpretations::U8, 1);
sbor_int!(u16, DefaultInterpretations::U16, 2);
sbor_int!(u32, DefaultInterpretations::U32, 4);
sbor_int!(u64, DefaultInterpretations::U64, 8);
sbor_int!(u128, DefaultInterpretations::U128, 16);

// Signed
sbor_int!(i8, DefaultInterpretations::I8, 1);
sbor_int!(i16, DefaultInterpretations::I16, 2);
sbor_int!(i32, DefaultInterpretations::I32, 4);
sbor_int!(i64, DefaultInterpretations::I64, 8);
sbor_int!(i128, DefaultInterpretations::I128, 16);

impl ConstInterpretation for usize {
    const INTERPRETATION: u8 = DefaultInterpretations::USIZE;
}

impl Encode for usize {
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_raw_bytes(&(*self as u64).to_le_bytes());
    }
}

impl Decode for usize {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let bytes = decoder.read_raw_bytes_fixed_length_array::<8>()?;
        let size = u64::from_le_bytes(bytes)
            .try_into()
            .map_err(|_| DecodeError::LengthInvalidForArchitecture)?;
        Ok(size)
    }
}

impl ConstInterpretation for isize {
    const INTERPRETATION: u8 = DefaultInterpretations::ISIZE;
}

impl Encode for isize {
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_raw_bytes(&(*self as i64).to_le_bytes());
    }
}

impl Decode for isize {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let bytes = decoder.read_raw_bytes_fixed_length_array::<8>()?;
        let size = i64::from_le_bytes(bytes)
            .try_into()
            .map_err(|_| DecodeError::LengthInvalidForArchitecture)?;
        Ok(size)
    }
}

impl ConstInterpretation for String {
    const INTERPRETATION: u8 = DefaultInterpretations::UTF8_STRING;
}

impl Encode for String {
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_raw_bytes(self.as_bytes());
    }
}

impl Decode for String {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let slice = decoder.read_raw_bytes()?;
        String::from_utf8(slice.to_vec())
            .map_err(|_| DecodeError::InvalidUtf8)
    }
}

impl<T> ConstInterpretation for Option<T> {
    const INTERPRETATION: u8 = DefaultInterpretations::OPTION;
}

impl<T: Encodable> Encode for Option<T> {
    fn encode_value(&self, encoder: &mut Encoder) {
        match self {
            None => encoder.write_sum_type_u8_discriminator(0, &()),
            Some(value) => encoder.write_sum_type_u8_discriminator(1, value),
        }
    }
}

impl<T: Decodable> Decode for Option<T> {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let discriminator_type = decoder.read_sum_type_discriminator_header()?;
        Ok(match discriminator_type {
            SumTypeDiscriminator::U8 => {
                let discriminator = decoder.read_sum_type_u8_discriminator()?;
                match discriminator {
                    0 => {
                        decoder.decode::<()>()?;
                        None
                    },
                    1 => Some(decoder.decode()?),
                    _ => Err(DecodeError::InvalidU8Discriminator(discriminator))?
                }
            },
            SumTypeDiscriminator::Any => {
                let discriminator: String = decoder.read_sum_type_any_discriminator()?;
                match discriminator.as_str() {
                    "None" => {
                        decoder.decode::<()>()?;
                        None
                    },
                    "Some" => Some(decoder.decode()?),
                    _ => Err(DecodeError::InvalidStringDiscriminator(discriminator))?
                }
            }
            _ => Err(DecodeError::InvalidDiscriminatorType(discriminator_type))?
        })
    }
}
