use crate::representations::*;
use crate::rust::prelude::*;
use crate::traversal::*;
use crate::*;

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type") // See https://serde.rs/enum-representations.html
)]
#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum NoCustomValueKind {}

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type") // See https://serde.rs/enum-representations.html
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoCustomValue {}

impl CustomValue<NoCustomValueKind> for NoCustomValue {
    fn get_custom_value_kind(&self) -> NoCustomValueKind {
        panic!("No custom value")
    }
}

pub type BasicEncoder<'a> = VecEncoder<'a, NoCustomValueKind>;
pub type BasicDecoder<'a> = VecDecoder<'a, NoCustomValueKind>;
pub type BasicTraverser<'a> = VecTraverser<'a, NoCustomTraversal>;
pub type BasicValue = Value<NoCustomValueKind, NoCustomValue>;
pub type BasicValueKind = ValueKind<NoCustomValueKind>;

// 5b for (basic) [5b]or - (90 in decimal)
pub const BASIC_SBOR_V1_PAYLOAD_PREFIX: u8 = 0x5b;
pub const BASIC_SBOR_V1_MAX_DEPTH: usize = 64;

// The following trait "aliases" are to be used in parameters.
//
// They are much nicer to read than the underlying traits, but because they are "new", and are defined
// via blanket impls, they can only be used for parameters, but cannot be used for implementations.
//
// Implementations should instead implement the underlying traits:
// * Categorize<X> (impl over all X: CustomValueKind)
// * Encode<X, E> (impl over all X: CustomValueKind, E: Encoder<X>)
// * Decode<X, D> (impl over all X: CustomValueKind, D: Decoder<X>)
//
// TODO: Change these to be Trait aliases once stable in rust: https://github.com/rust-lang/rust/issues/41517
pub trait BasicCategorize: Categorize<NoCustomValueKind> {}
impl<T: Categorize<NoCustomValueKind> + ?Sized> BasicCategorize for T {}

pub trait BasicSborEnum: SborEnum<NoCustomValueKind> {}
impl<T: SborEnum<NoCustomValueKind> + ?Sized> BasicSborEnum for T {}

pub trait BasicSborTuple: SborTuple<NoCustomValueKind> {}
impl<T: SborTuple<NoCustomValueKind> + ?Sized> BasicSborTuple for T {}

pub trait BasicDecode: for<'a> Decode<NoCustomValueKind, BasicDecoder<'a>> {}
impl<T: for<'a> Decode<NoCustomValueKind, BasicDecoder<'a>>> BasicDecode for T {}

pub trait BasicEncode: for<'a> Encode<NoCustomValueKind, BasicEncoder<'a>> {}
impl<T: for<'a> Encode<NoCustomValueKind, BasicEncoder<'a>> + ?Sized> BasicEncode for T {}

pub trait BasicDescribe: for<'a> Describe<NoCustomTypeKind> {}
impl<T: Describe<NoCustomTypeKind> + ?Sized> BasicDescribe for T {}

pub trait BasicSbor: BasicCategorize + BasicDecode + BasicEncode + BasicDescribe {}
impl<T: BasicCategorize + BasicDecode + BasicEncode + BasicDescribe> BasicSbor for T {}

/// Encode a `T` into byte array.
pub fn basic_encode<T: BasicEncode + ?Sized>(v: &T) -> Result<Vec<u8>, EncodeError> {
    let mut buf = Vec::with_capacity(512);
    let encoder = BasicEncoder::new(&mut buf, BASIC_SBOR_V1_MAX_DEPTH);
    encoder.encode_payload(v, BASIC_SBOR_V1_PAYLOAD_PREFIX)?;
    Ok(buf)
}

/// Decode an instance of `T` from a slice.
pub fn basic_decode<T: BasicDecode>(buf: &[u8]) -> Result<T, DecodeError> {
    BasicDecoder::new(buf, BASIC_SBOR_V1_MAX_DEPTH).decode_payload(BASIC_SBOR_V1_PAYLOAD_PREFIX)
}

impl CustomValueKind for NoCustomValueKind {
    fn as_u8(&self) -> u8 {
        panic!("No custom type")
    }

    fn from_u8(_id: u8) -> Option<Self> {
        panic!("No custom type")
    }
}

impl<X: CustomValueKind, E: Encoder<X>> Encode<X, E> for NoCustomValue {
    fn encode_value_kind(&self, _encoder: &mut E) -> Result<(), EncodeError> {
        panic!("No custom value")
    }

    fn encode_body(&self, _encoder: &mut E) -> Result<(), EncodeError> {
        panic!("No custom value")
    }
}

impl<X: CustomValueKind, D: Decoder<X>> Decode<X, D> for NoCustomValue {
    fn decode_body_with_value_kind(_: &mut D, _: ValueKind<X>) -> Result<Self, DecodeError>
    where
        Self: Sized,
    {
        panic!("No custom value")
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum NoCustomTerminalValueRef {}

impl CustomTerminalValueRef for NoCustomTerminalValueRef {
    type CustomValueKind = NoCustomValueKind;

    fn custom_value_kind(&self) -> Self::CustomValueKind {
        unreachable!("NoCustomTerminalValueRef can't exist")
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum NoCustomTraversal {}

impl CustomTraversal for NoCustomTraversal {
    type CustomValueKind = NoCustomValueKind;
    type CustomTerminalValueRef<'de> = NoCustomTerminalValueRef;

    fn decode_custom_value_body<'de, R>(
        _custom_value_kind: Self::CustomValueKind,
        _reader: &mut R,
    ) -> Result<Self::CustomTerminalValueRef<'de>, DecodeError>
    where
        R: BorrowingDecoder<'de, Self::CustomValueKind>,
    {
        unreachable!("NoCustomTraversal can't exist")
    }
}

/// Creates a payload traverser from the buffer
pub fn basic_payload_traverser<'b>(buf: &'b [u8]) -> BasicTraverser<'b> {
    BasicTraverser::new(
        buf,
        BASIC_SBOR_V1_MAX_DEPTH,
        ExpectedStart::PayloadPrefix(BASIC_SBOR_V1_PAYLOAD_PREFIX),
        true,
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum NoCustomTypeKind {}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum NoCustomTypeValidation {}

impl CustomTypeValidation for NoCustomTypeValidation {}

impl<L: SchemaTypeLink> CustomTypeKind<L> for NoCustomTypeKind {
    type CustomTypeValidation = NoCustomTypeValidation;
}

lazy_static::lazy_static! {
    static ref EMPTY_SCHEMA: Schema<NoCustomSchema> = {
        Schema::empty()
    };
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum NoCustomSchema {}

impl CustomSchema for NoCustomSchema {
    type CustomTypeKind<L: SchemaTypeLink> = NoCustomTypeKind;
    type CustomTypeValidation = NoCustomTypeValidation;

    fn linearize_type_kind(
        _: Self::CustomTypeKind<GlobalTypeId>,
        _: &IndexSet<TypeHash>,
    ) -> Self::CustomTypeKind<LocalTypeIndex> {
        unreachable!("No custom type kinds exist")
    }

    fn resolve_well_known_type(
        well_known_index: u8,
    ) -> Option<&'static TypeData<Self::CustomTypeKind<LocalTypeIndex>, LocalTypeIndex>> {
        // We know that WELL_KNOWN_LOOKUP has 256 elements, so can use `get_unchecked` for fast look-ups
        unsafe {
            WELL_KNOWN_LOOKUP
                .get_unchecked(well_known_index as usize)
                .as_ref()
        }
    }

    fn validate_custom_type_validation(
        _: &SchemaContext,
        _: &Self::CustomTypeKind<LocalTypeIndex>,
        _: &Self::CustomTypeValidation,
    ) -> Result<(), SchemaValidationError> {
        unreachable!("No custom type validation")
    }

    fn validate_custom_type_kind(
        _: &SchemaContext,
        _: &Self::CustomTypeKind<LocalTypeIndex>,
    ) -> Result<(), SchemaValidationError> {
        unreachable!("No custom type kinds exist")
    }

    fn validate_type_metadata_with_custom_type_kind(
        _: &SchemaContext,
        _: &Self::CustomTypeKind<LocalTypeIndex>,
        _: &TypeMetadata,
    ) -> Result<(), SchemaValidationError> {
        unreachable!("No custom type kinds exist")
    }

    fn empty_schema() -> &'static Schema<Self> {
        &EMPTY_SCHEMA
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum NoCustomExtension {}

create_well_known_lookup!(WELL_KNOWN_LOOKUP, NoCustomTypeKind, []);

impl CustomExtension for NoCustomExtension {
    const MAX_DEPTH: usize = BASIC_SBOR_V1_MAX_DEPTH;
    const PAYLOAD_PREFIX: u8 = BASIC_SBOR_V1_PAYLOAD_PREFIX;
    type CustomValueKind = NoCustomValueKind;
    type CustomTraversal = NoCustomTraversal;
    type CustomSchema = NoCustomSchema;

    fn custom_value_kind_matches_type_kind(
        _: &Schema<Self::CustomSchema>,
        _: Self::CustomValueKind,
        _: &TypeKind<
            <Self::CustomSchema as CustomSchema>::CustomTypeKind<LocalTypeIndex>,
            LocalTypeIndex,
        >,
    ) -> bool {
        unreachable!("No custom value kinds exist")
    }

    fn custom_type_kind_matches_non_custom_value_kind(
        _: &Schema<Self::CustomSchema>,
        _: &<Self::CustomSchema as CustomSchema>::CustomTypeKind<LocalTypeIndex>,
        _: ValueKind<Self::CustomValueKind>,
    ) -> bool {
        unreachable!("No custom type kinds exist")
    }
}

pub type BasicRawPayload<'a> = RawPayload<'a, NoCustomExtension>;
pub type BasicOwnedRawPayload = RawPayload<'static, NoCustomExtension>;
pub type BasicRawValue<'a> = RawValue<'a, NoCustomExtension>;
pub type BasicOwnedRawValue = RawValue<'static, NoCustomExtension>;
pub type BasicTypeKind<L> = TypeKind<NoCustomTypeKind, L>;
pub type BasicSchema = Schema<NoCustomSchema>;
pub type BasicTypeData<L> = TypeData<NoCustomTypeKind, L>;

impl<'a> CustomDisplayContext<'a> for () {
    type CustomExtension = NoCustomExtension;
}

impl FormattableCustomExtension for NoCustomExtension {
    type CustomDisplayContext<'a> = ();

    fn display_string_content<'s, 'de, 'a, 't, 's1, 's2, F: fmt::Write>(
        _: &mut F,
        _: &Self::CustomDisplayContext<'a>,
        _: &<Self::CustomTraversal as CustomTraversal>::CustomTerminalValueRef<'de>,
    ) -> Result<(), fmt::Error> {
        unreachable!("No custom values exist")
    }
}

impl ValidatableCustomExtension<()> for NoCustomExtension {
    fn apply_validation_for_custom_value<'de>(
        _: &Schema<Self::CustomSchema>,
        _: &<Self::CustomTraversal as CustomTraversal>::CustomTerminalValueRef<'de>,
        _: LocalTypeIndex,
        _: &(),
    ) -> Result<(), PayloadValidationError<Self>> {
        unreachable!("No custom values exist")
    }

    fn apply_custom_type_validation_for_non_custom_value<'de>(
        _: &Schema<Self::CustomSchema>,
        _: &<Self::CustomSchema as CustomSchema>::CustomTypeValidation,
        _: &TerminalValueRef<'de, Self::CustomTraversal>,
        _: &(),
    ) -> Result<(), PayloadValidationError<Self>> {
        unreachable!("No custom type validationss exist")
    }
}

#[cfg(feature = "serde")]
pub use self::serde_serialization::*;

#[cfg(feature = "serde")]
mod serde_serialization {
    use super::*;

    impl SerializableCustomExtension for NoCustomExtension {
        fn map_value_for_serialization<'s, 'de, 'a, 't, 's1, 's2>(
            _: &SerializationContext<'s, 'a, Self>,
            _: LocalTypeIndex,
            _: <Self::CustomTraversal as CustomTraversal>::CustomTerminalValueRef<'de>,
        ) -> CustomTypeSerialization<'a, 't, 'de, 's1, 's2, Self> {
            unreachable!("No custom values exist")
        }
    }
}
