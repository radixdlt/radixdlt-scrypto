use crate::*;

pub struct SborFixedEnumVariant<const DISCRIMINATOR: u8, T> {
    pub fields: T,
}

impl<const DISCRIMINATOR: u8, T> SborFixedEnumVariant<DISCRIMINATOR, T> {
    pub fn new(fields: T) -> Self {
        Self { fields }
    }

    pub fn for_encoding(fields: &T) -> SborFixedEnumVariant<DISCRIMINATOR, &T> {
        SborFixedEnumVariant { fields }
    }

    pub fn into_fields(self) -> T {
        self.fields
    }
}

impl<X: CustomValueKind, const DISCRIMINATOR: u8, T: SborTuple<X>> Categorize<X>
    for SborFixedEnumVariant<DISCRIMINATOR, T>
{
    fn value_kind() -> ValueKind<X> {
        ValueKind::Enum
    }
}

impl<X: CustomValueKind, const DISCRIMINATOR: u8, T: SborTuple<X>> SborEnum<X>
    for SborFixedEnumVariant<DISCRIMINATOR, T>
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
    > Encode<X, E> for SborFixedEnumVariant<DISCRIMINATOR, T>
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
    > Decode<X, D> for SborFixedEnumVariant<DISCRIMINATOR, T>
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
// Now define a trait `IsSborFixedEnumVariant` - which hides the discriminator.
// This is only really needed because of https://github.com/rust-lang/rust/issues/76560
// In particular, see eg `TransactionPayload` where we couldn't define SborFixedEnumVariant<{ Self::DISCRIMINATOR }, X>
//=======================================================================================

pub trait IsSborFixedEnumVariant<X: CustomValueKind, T: SborTuple<X>>:
    SborEnum<X> + Categorize<X>
{
    const DISCRIMINATOR: u8;
    type EncodingSborFixedEnumVariant<'a>: IsSborFixedEnumVariant<X, &'a T>
    where
        T: 'a;
    fn new(fields: T) -> Self;
    fn for_encoding<'a>(fields: &'a T) -> Self::EncodingSborFixedEnumVariant<'a>;
    fn into_fields(self) -> T;
}

impl<X: CustomValueKind, const DISCRIMINATOR: u8, T: SborTuple<X>> IsSborFixedEnumVariant<X, T>
    for SborFixedEnumVariant<DISCRIMINATOR, T>
{
    const DISCRIMINATOR: u8 = DISCRIMINATOR;
    type EncodingSborFixedEnumVariant<'a> = SborFixedEnumVariant<DISCRIMINATOR, &'a T> where T: 'a;

    fn new(fields: T) -> Self {
        Self::new(fields)
    }

    fn for_encoding<'a>(fields: &'a T) -> Self::EncodingSborFixedEnumVariant<'a> {
        Self::for_encoding(fields)
    }

    fn into_fields(self) -> T {
        self.fields
    }
}

/// This trait is output for unique unskipped single children of enum variants, when
/// `#[sbor(impl_variant_traits)]` is specified on an Enum or
/// `#[sbor(impl_variant_trait)]` is specified on a single Enum variant.
///
/// This trait pairs well with the `#[sbor(flatten)]` attribute, for implementing
/// the "enum variant is singleton struct type" pattern, which allows handling a
/// variant as its own type.
///
/// This trait allows the variant type to easily be considered part of its parent,
/// or encoded as a variant under its parent enum.
///
/// ### Note on generic parameter ordering
/// On this trait, we do not put `X` first, as is normal with the SBOR traits.
///
/// Instead, `TEnum` comes before `X` so that the trait can be implemented on any foreign
/// type assuming `TEnum` is local. This is so that the cryptic orphan rule
/// discussed in this stack overflow comment is satisfied https://stackoverflow.com/a/63131661
///
/// With this ordering we have P0 = X, T0 = Child Type (possibly foreign), T1 = TEnum, T2 = X,
/// which passes the check.
pub trait SborEnumVariantFor<TEnum: SborEnum<X>, X: CustomValueKind> {
    const DISCRIMINATOR: u8;
    const IS_FLATTENED: bool;

    /// VariantFields is either Self if IS_FLATTENED else is (Self,)
    type VariantFields: SborTuple<X>;
    /// VariantFieldsRef is either &Self if IS_FLATTENED else is (&Self,)
    type VariantFieldsRef<'a>: SborTuple<X>
    where
        Self: 'a,
        Self::VariantFields: 'a;
    fn as_variant_fields_ref(&self) -> Self::VariantFieldsRef<'_>;

    /// Should always be `SborFixedEnumVariant<{ Self::DISCRIMINATOR as u8 }, Self::VariantFields>`
    type DecodableVariant: IsSborFixedEnumVariant<X, Self::VariantFields>;
    fn from_decodable_variant(variant: Self::DecodableVariant) -> Self;

    /// Should always be `SborFixedEnumVariant<{ Self::DISCRIMINATOR }, &'a Self::VariantFields>`
    type EncodableVariant<'a>: IsSborFixedEnumVariant<X, Self::VariantFieldsRef<'a>>
    where
        Self: 'a,
        Self::VariantFields: 'a;

    fn as_encodable_variant(&self) -> Self::EncodableVariant<'_> {
        Self::EncodableVariant::new(self.as_variant_fields_ref())
    }

    fn into_enum(self) -> TEnum;
}
