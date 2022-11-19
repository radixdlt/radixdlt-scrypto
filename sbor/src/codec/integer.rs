use crate::type_id::*;
use crate::*;

impl<X: CustomTypeId> Encode<X> for i8 {
    #[inline]
    fn encode_type_id(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(Self::type_id());
    }
    #[inline]
    fn encode_body(&self, encoder: &mut Encoder<X>) {
        encoder.write_byte(*self as u8);
    }
}

impl<X: CustomTypeId> Encode<X> for u8 {
    #[inline]
    fn encode_type_id(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(Self::type_id());
    }
    #[inline]
    fn encode_body(&self, encoder: &mut Encoder<X>) {
        encoder.write_byte(*self);
    }
}

macro_rules! encode_int {
    ($type:ident, $type_id:ident) => {
        impl<X: CustomTypeId> Encode<X> for $type {
            #[inline]
            fn encode_type_id(&self, encoder: &mut Encoder<X>) {
                encoder.write_type_id(Self::type_id());
            }
            #[inline]
            fn encode_body(&self, encoder: &mut Encoder<X>) {
                encoder.write_slice(&(*self).to_le_bytes());
            }
        }
    };
}

encode_int!(i16, TYPE_I16);
encode_int!(i32, TYPE_I32);
encode_int!(i64, TYPE_I64);
encode_int!(i128, TYPE_I128);
encode_int!(u16, TYPE_U16);
encode_int!(u32, TYPE_U32);
encode_int!(u64, TYPE_U64);
encode_int!(u128, TYPE_U128);

impl<X: CustomTypeId> Encode<X> for isize {
    #[inline]
    fn encode_type_id(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(Self::type_id());
    }
    #[inline]
    fn encode_body(&self, encoder: &mut Encoder<X>) {
        (*self as i64).encode_body(encoder);
    }
}

impl<X: CustomTypeId> Encode<X> for usize {
    #[inline]
    fn encode_type_id(&self, encoder: &mut Encoder<X>) {
        encoder.write_type_id(Self::type_id());
    }
    #[inline]
    fn encode_body(&self, encoder: &mut Encoder<X>) {
        (*self as u64).encode_body(encoder);
    }
}

impl<X: CustomTypeId, D: Decoder<X>> Decode<X, D> for i8 {
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        let value = decoder.read_byte()?;
        Ok(value as i8)
    }
}

impl<X: CustomTypeId, D: Decoder<X>> Decode<X, D> for u8 {
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        let value = decoder.read_byte()?;
        Ok(value)
    }
}

macro_rules! decode_int {
    ($type:ident, $type_id:ident, $n:expr) => {
        impl<X: CustomTypeId, D: Decoder<X>> Decode<X, D> for $type {
            fn decode_body_with_type_id(
                decoder: &mut D,
                type_id: SborTypeId<X>,
            ) -> Result<Self, DecodeError> {
                decoder.check_preloaded_type_id(type_id, Self::type_id())?;
                let slice = decoder.read_slice($n)?;
                let mut bytes = [0u8; $n];
                bytes.copy_from_slice(&slice[..]);
                Ok(<$type>::from_le_bytes(bytes))
            }
        }
    };
}

decode_int!(i16, TYPE_I16, 2);
decode_int!(i32, TYPE_I32, 4);
decode_int!(i64, TYPE_I64, 8);
decode_int!(i128, TYPE_I128, 16);
decode_int!(u16, TYPE_U16, 2);
decode_int!(u32, TYPE_U32, 4);
decode_int!(u64, TYPE_U64, 8);
decode_int!(u128, TYPE_U128, 16);

impl<X: CustomTypeId, D: Decoder<X>> Decode<X, D> for isize {
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        i64::decode_body_with_type_id(decoder, type_id).map(|i| i as isize)
    }
}

impl<X: CustomTypeId, D: Decoder<X>> Decode<X, D> for usize {
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        u64::decode_body_with_type_id(decoder, type_id).map(|i| i as usize)
    }
}
