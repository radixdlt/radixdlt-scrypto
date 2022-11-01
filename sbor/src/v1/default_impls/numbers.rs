use super::super::*;

impl Interpretation for u8 {
    const INTERPRETATION: u8 = DefaultInterpretations::U8;
    const IS_BYTE: bool = true;
}

impl Encode for u8 {
    fn encode_value(&self, encoder: &mut Encoder) {
        encoder.write_raw_bytes(
            &[*self],
        );
    }
}

impl Decode for u8 {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let bytes = decoder.read_raw_bytes_fixed_length_array::<1>()?;
        Ok(bytes[0])
    }
}

macro_rules! sbor_int {
    ($type:ident, $interpretation:expr, $bytes_length:expr) => {
        impl Interpretation for $type {
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

impl Interpretation for usize {
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

impl Interpretation for isize {
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
