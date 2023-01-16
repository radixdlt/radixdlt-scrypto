use crate::value_kind::*;
use crate::*;

categorize_simple!(i8, ValueKind::I8);
categorize_simple!(i16, ValueKind::I16);
categorize_simple!(i32, ValueKind::I32);
categorize_simple!(i64, ValueKind::I64);
categorize_simple!(i128, ValueKind::I128);
categorize_simple!(isize, ValueKind::I64);
categorize_simple!(u8, ValueKind::U8);
categorize_simple!(u16, ValueKind::U16);
categorize_simple!(u32, ValueKind::U32);
categorize_simple!(u64, ValueKind::U64);
categorize_simple!(u128, ValueKind::U128);
categorize_simple!(usize, ValueKind::U64);

impl<X: CustomValueKind, E: Encoder<X>> Encode<X, E> for i8 {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_byte(*self as u8)
    }
}

impl<X: CustomValueKind, E: Encoder<X>> Encode<X, E> for u8 {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_byte(*self)
    }
}

macro_rules! encode_int {
    ($type:ident, $value_kind:ident) => {
        impl<X: CustomValueKind, E: Encoder<X>> Encode<X, E> for $type {
            #[inline]
            fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
                encoder.write_value_kind(Self::value_kind())
            }

            #[inline]
            fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
                encoder.write_slice(&(*self).to_le_bytes())
            }
        }
    };
}

encode_int!(i16, VALUE_KIND_I16);
encode_int!(i32, VALUE_KIND_I32);
encode_int!(i64, VALUE_KIND_I64);
encode_int!(i128, VALUE_KIND_I128);
encode_int!(u16, VALUE_KIND_U16);
encode_int!(u32, VALUE_KIND_U32);
encode_int!(u64, VALUE_KIND_U64);
encode_int!(u128, VALUE_KIND_U128);

impl<X: CustomValueKind, E: Encoder<X>> Encode<X, E> for isize {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        (*self as i64).encode_body(encoder)
    }
}

impl<X: CustomValueKind, E: Encoder<X>> Encode<X, E> for usize {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        (*self as u64).encode_body(encoder)
    }
}

impl<X: CustomValueKind, D: Decoder<X>> Decode<X, D> for i8 {
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        let value = decoder.read_byte()?;
        Ok(value as i8)
    }
}

impl<X: CustomValueKind, D: Decoder<X>> Decode<X, D> for u8 {
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        let value = decoder.read_byte()?;
        Ok(value)
    }
}

macro_rules! decode_int {
    ($type:ident, $value_kind:ident, $n:expr) => {
        impl<X: CustomValueKind, D: Decoder<X>> Decode<X, D> for $type {
            #[inline]
            fn decode_body_with_value_kind(
                decoder: &mut D,
                value_kind: ValueKind<X>,
            ) -> Result<Self, DecodeError> {
                decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
                let slice = decoder.read_slice($n)?;
                let mut bytes = [0u8; $n];
                bytes.copy_from_slice(&slice[..]);
                Ok(<$type>::from_le_bytes(bytes))
            }
        }
    };
}

decode_int!(i16, VALUE_KIND_I16, 2);
decode_int!(i32, VALUE_KIND_I32, 4);
decode_int!(i64, VALUE_KIND_I64, 8);
decode_int!(i128, VALUE_KIND_I128, 16);
decode_int!(u16, VALUE_KIND_U16, 2);
decode_int!(u32, VALUE_KIND_U32, 4);
decode_int!(u64, VALUE_KIND_U64, 8);
decode_int!(u128, VALUE_KIND_U128, 16);

impl<X: CustomValueKind, D: Decoder<X>> Decode<X, D> for isize {
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        i64::decode_body_with_value_kind(decoder, value_kind).map(|i| i as isize)
    }
}

impl<X: CustomValueKind, D: Decoder<X>> Decode<X, D> for usize {
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        u64::decode_body_with_value_kind(decoder, value_kind).map(|i| i as usize)
    }
}

pub use schema::*;

mod schema {
    use super::*;

    describe_basic_well_known_type!(u8, U8_ID);
    describe_basic_well_known_type!(u16, U16_ID);
    describe_basic_well_known_type!(u32, U32_ID);
    describe_basic_well_known_type!(u64, U64_ID);
    describe_basic_well_known_type!(u128, U128_ID);
    describe_basic_well_known_type!(i8, I8_ID);
    describe_basic_well_known_type!(i16, I16_ID);
    describe_basic_well_known_type!(i32, I32_ID);
    describe_basic_well_known_type!(i64, I64_ID);
    describe_basic_well_known_type!(i128, I128_ID);

    describe_basic_well_known_type!(usize, U64_ID);
    describe_basic_well_known_type!(isize, I64_ID);
}
