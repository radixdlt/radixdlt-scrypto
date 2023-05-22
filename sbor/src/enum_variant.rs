use crate::*;

pub struct EnumVariant<const DISCRIMINATOR: u8, T> {
    pub fields: T,
}

impl<const DISCRIMINATOR: u8, T> EnumVariant<DISCRIMINATOR, T> {
    pub fn new(fields: T) -> Self {
        Self { fields }
    }
}

impl<X: CustomValueKind, const DISCRIMINATOR: u8, T: SborTuple<X>> Categorize<X>
    for EnumVariant<DISCRIMINATOR, T>
{
    fn value_kind() -> ValueKind<X> {
        ValueKind::Enum
    }
}

impl<X: CustomValueKind, const DISCRIMINATOR: u8, T: SborTuple<X>> SborEnum<X>
    for EnumVariant<DISCRIMINATOR, T>
{
    fn get_length(&self) -> usize {
        self.fields.get_length()
    }

    fn get_discriminator(&self) -> u8 {
        DISCRIMINATOR
    }
}

impl<
        X: CustomValueKind,
        E: Encoder<X>,
        const DISCRIMINATOR: u8,
        T: Encode<X, E> + SborTuple<X>,
    > Encode<X, E> for EnumVariant<DISCRIMINATOR, T>
{
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_discriminator(DISCRIMINATOR)?;
        self.fields.encode_body(encoder)
    }
}

impl<
        X: CustomValueKind,
        D: Decoder<X>,
        const DISCRIMINATOR: u8,
        T: Decode<X, D> + SborTuple<X>,
    > Decode<X, D> for EnumVariant<DISCRIMINATOR, T>
{
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<X>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        decoder.read_expected_discriminator(DISCRIMINATOR)?;
        // The fields is actually a tuple type - so we pass in ValueKind::Tuple to trick the encoding
        let fields = T::decode_body_with_value_kind(decoder, ValueKind::Tuple)?;
        Ok(Self { fields })
    }
}
