use crate::*;

pub struct SborFixedEnumVariant<const DISCRIMINATOR: u8, T> {
    pub fields: T,
}

impl<const DISCRIMINATOR: u8, T> SborFixedEnumVariant<DISCRIMINATOR, T> {
    pub fn new(fields: T) -> Self {
        Self { fields }
    }

    pub fn discriminator() -> u8 {
        DISCRIMINATOR
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
// Now define a trait `IsSborFixedEnumVariant<T>` - this is intended to represent
//  `SborFixedEnumVariant<?, T>` of unknown discriminator.
// This is only really needed because of https://github.com/rust-lang/rust/issues/76560
//  and the explanation below "Why is this required as an associated type?".
// In particular, see eg `TransactionPayload` where we couldn't define `SborFixedEnumVariant<{ Self::DISCRIMINATOR }, X>`
//=======================================================================================
pub trait IsSborFixedEnumVariant<F> {
    const DISCRIMINATOR: u8;
    fn new(fields: F) -> Self;
    fn into_fields(self) -> F;
}

impl<const DISCRIMINATOR: u8, F> IsSborFixedEnumVariant<F>
    for SborFixedEnumVariant<DISCRIMINATOR, F>
{
    const DISCRIMINATOR: u8 = DISCRIMINATOR;

    fn new(fields: F) -> Self {
        Self::new(fields)
    }

    fn into_fields(self) -> F {
        self.fields
    }
}

/// This trait is output for unique unskipped single children of enum variants, when
/// `#[sbor(impl_variant_traits)]` is specified on an Enum or
/// `#[sbor(impl_variant_trait)]` is specified on a single Enum variant.
///
/// It allows considering this type as representing an enum variant type
/// under its parent enum. There are two flavours of how this embedding works in SBOR:
/// * In unflattened variants, it is a variant with fields (Self,)
/// * In flattened variants (only possible for tuple types) it is a variant with fields Self
///
/// This trait pairs well with the `#[sbor(flatten)]` attribute, for implementing
/// the "enum variant is singleton struct type" pattern, which allows a number of benefits:
/// * A function can take or return a particular variant
/// * If code size is important (e.g. when building Scrypto), we want to enable pruning of
///   any serialization code we don't need. Using `as_encodable_variant` and `from_decoded_variant`
///   avoids pulling in the parent `TEnum` serialization code; and the serialization code for any
///   unused types in other discriminators
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

    /// VariantFields is either `Self` if `IS_FLATTENED` else is `(Self,)`
    type VariantFields: SborTuple<X>;
    fn from_variant_fields(variant_fields: Self::VariantFields) -> Self;

    /// VariantFieldsRef is either `&Self` if `IS_FLATTENED` else is `(&Self,)`
    type VariantFieldsRef<'a>: SborTuple<X>
    where
        Self: 'a,
        Self::VariantFields: 'a;
    fn as_variant_fields_ref(&self) -> Self::VariantFieldsRef<'_>;

    /// This should always be `SborFixedEnumVariant<{ [DISCRIMINATOR] as u8 }, Self::VariantFields>`
    ///
    /// ### Why is this required as an associated type?
    /// Ideally we'd not need this and just have `as_encodable_variant` return
    /// `SborFixedEnumVariant<{ Self::DISCRIMINATOR as u8 }, Self::VariantFieldsRef<'_>>`
    /// But this gets "error: generic parameters may not be used in const operations"
    ///
    /// ### Why doesn't this require `VecDecode<X>`?
    /// We don't want a compiler error if a type only implements Categorize (and so gets this trait)
    /// but not Decode. Really I'd like to say `OwnedVariant: VecDecode<X> if Self: VecDecode<X>`
    /// but Rust doesn't support that. Instead, users will need to add the bound on the associated
    /// type themselves.
    type OwnedVariant: IsSborFixedEnumVariant<Self::VariantFields>;

    /// Should always be `SborFixedEnumVariant<{ [DISCRIMINATOR] as u8 }, &'a Self::VariantFields>`
    ///
    /// ### Why is this required as an associated type?
    /// Ideally we'd not need this and just have `as_encodable_variant` return
    /// `SborFixedEnumVariant<{ Self::DISCRIMINATOR }, Self::VariantFields>`
    /// But this gets "error: generic parameters may not be used in const operations" which needs
    /// the `const-generics` feature which has been in progress for a number of years.
    /// See https://github.com/rust-lang/project-const-generics/issues/31
    ///
    /// ### Why doesn't this require `VecEncode<X>`?
    /// We don't want a compiler error if a type only implements Categorize (and so gets this trait)
    /// but not Encode. Really I'd like to say `BorrowedVariant<'a>: VecEncode<X> if Self: VecEncode<X>`
    /// but Rust doesn't support that. Instead, users will need to add the bound on the associated
    /// type themselves.
    ///
    /// Instead, you can express a trait bound as such:
    /// ```ignore
    /// pub trait MyNewSuperTrait:
    ///     for<'a> SborEnumVariantFor<
    ///        TEnum,
    ///        X,
    ///        OwnedVariant: ManifestDecode,
    ///        BorrowedVariant<'a>: ManifestEncode,
    ///     >
    /// {}
    /// ```
    type BorrowedVariant<'a>: IsSborFixedEnumVariant<Self::VariantFieldsRef<'a>>
    where
        Self: 'a,
        Self::VariantFields: 'a;

    /// Can be used to encode the type as a variant under `TEnum`, like this:
    /// `encoder.encode(x.as_encodable_variant())`.
    ///
    /// To use this pattern in a generic context, you will likely need to add a bound like
    /// `for<'a> T::BorrowedVariant<'a>: VecEncode<X>`.
    fn as_encodable_variant<'a>(&'a self) -> Self::BorrowedVariant<'a> {
        Self::BorrowedVariant::new(self.as_variant_fields_ref())
    }

    /// Can be used to decode the type from an encoded variant, like this:
    /// `T::from_decoded_variant(decoder.decode()?)`.
    ///
    /// To use this pattern in a generic context, you will likely need to add a bound like
    /// `T::OwnedVariant: VecDecode<X>`.
    fn from_decoded_variant(variant: Self::OwnedVariant) -> Self
    where
        Self: core::marker::Sized,
    {
        Self::from_variant_fields(variant.into_fields())
    }

    fn into_enum(self) -> TEnum;
}
