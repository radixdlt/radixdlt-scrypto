use crate::*;

pub struct FixedEnumVariant<const DISCRIMINATOR: u8, T> {
    pub fields: T,
}

impl<const DISCRIMINATOR: u8, T> FixedEnumVariant<DISCRIMINATOR, T> {
    pub fn new(fields: T) -> Self {
        Self { fields }
    }

    pub fn for_encoding(fields: &T) -> FixedEnumVariant<DISCRIMINATOR, &T> {
        FixedEnumVariant { fields }
    }
}

impl<X: CustomValueKind, const DISCRIMINATOR: u8, T: SborTuple<X>> Categorize<X>
    for FixedEnumVariant<DISCRIMINATOR, T>
{
    fn value_kind() -> ValueKind<X> {
        ValueKind::Enum
    }
}

impl<X: CustomValueKind, const DISCRIMINATOR: u8, T: SborTuple<X>> SborEnum<X>
    for FixedEnumVariant<DISCRIMINATOR, T>
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
    > Encode<X, E> for FixedEnumVariant<DISCRIMINATOR, T>
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
    > Decode<X, D> for FixedEnumVariant<DISCRIMINATOR, T>
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

//=======================================================================================
// Now define a trait `IsFixedEnumVariant` - which hides the discriminator.
// This is only really needed because of https://github.com/rust-lang/rust/issues/76560
// In particular, see eg `TransactionPayload` where we couldn't define FixedEnumVariant<{ Self::DISCRIMINATOR }, X>
//=======================================================================================

pub trait IsFixedEnumVariant<X: CustomValueKind, T: SborTuple<X>>:
    SborEnum<X> + Categorize<X> + for<'a> Encode<X, VecEncoder<'a, X>>
where
    T: for<'a> Encode<X, VecEncoder<'a, X>>,
{
    const DISCRIMINATOR: u8;
    type EncodingFixedEnumVariant<'a>: IsFixedEnumVariant<X, &'a T>
    where
        T: 'a;
    fn new(fields: T) -> Self;
    fn for_encoding<'a>(fields: &'a T) -> Self::EncodingFixedEnumVariant<'a>;
    fn into_fields(self) -> T;
}

impl<X: CustomValueKind, const DISCRIMINATOR: u8, T: SborTuple<X>> IsFixedEnumVariant<X, T>
    for FixedEnumVariant<DISCRIMINATOR, T>
where
    T: for<'a> Encode<X, VecEncoder<'a, X>>,
{
    const DISCRIMINATOR: u8 = DISCRIMINATOR;
    type EncodingFixedEnumVariant<'a> = FixedEnumVariant<DISCRIMINATOR, &'a T> where T: 'a;

    fn new(fields: T) -> Self {
        Self::new(fields)
    }

    fn for_encoding<'a>(fields: &'a T) -> Self::EncodingFixedEnumVariant<'a> {
        Self::for_encoding(fields)
    }

    fn into_fields(self) -> T {
        self.fields
    }
}
