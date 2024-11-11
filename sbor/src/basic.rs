use crate::internal_prelude::*;
use crate::representations::*;
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
pub type BasicEnumVariantValue = EnumVariantValue<NoCustomValueKind, NoCustomValue>;

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

pub trait BasicSborEnumVariantFor<E: BasicSborEnum>:
    SborEnumVariantFor<E, NoCustomValueKind>
{
}
impl<E: BasicSborEnum, T: SborEnumVariantFor<E, NoCustomValueKind> + ?Sized>
    BasicSborEnumVariantFor<E> for T
{
}

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
    basic_encode_with_depth_limit(v, BASIC_SBOR_V1_MAX_DEPTH)
}

pub fn basic_encode_with_depth_limit<T: BasicEncode + ?Sized>(
    v: &T,
    depth_limit: usize,
) -> Result<Vec<u8>, EncodeError> {
    let mut buf = Vec::with_capacity(512);
    let encoder = BasicEncoder::new(&mut buf, depth_limit);
    encoder.encode_payload(v, BASIC_SBOR_V1_PAYLOAD_PREFIX)?;
    Ok(buf)
}

/// Decode an instance of `T` from a slice.
pub fn basic_decode<T: BasicDecode>(buf: &[u8]) -> Result<T, DecodeError> {
    basic_decode_with_depth_limit(buf, BASIC_SBOR_V1_MAX_DEPTH)
}

pub fn basic_decode_with_depth_limit<T: BasicDecode>(
    buf: &[u8],
    depth_limit: usize,
) -> Result<T, DecodeError> {
    BasicDecoder::new(buf, depth_limit).decode_payload(BASIC_SBOR_V1_PAYLOAD_PREFIX)
}

/// Decodes a data structure from a byte array.
///
/// If an error occurs, the type's schema is exported and used to give a better error message.
///
/// NOTE:
/// * The error path runs very slowly. This should only be used where errors are NOT expected.
/// * This should not be used where the size of compiled code is an issue, as it will pull
///   in the schema aggregation code which is large.
pub fn basic_decode_with_nice_error<T: BasicDecode + BasicDescribe>(
    buf: &[u8],
) -> Result<T, String> {
    vec_decode_with_nice_error::<NoCustomExtension, T>(buf, BASIC_SBOR_V1_MAX_DEPTH)
}

/// Decodes a data structure from a byte array.
///
/// If an error occurs, the type's schema is exported and used to give a better error message.
///
/// NOTE:
/// * The error path runs very slowly. This should only be used where errors are NOT expected.
/// * This should not be used where the size of compiled code is an issue, as it will pull
///   in the schema aggregation code which is large.
pub fn basic_decode_with_depth_limit_and_nice_error<T: BasicDecode + BasicDescribe>(
    buf: &[u8],
    depth_limit: usize,
) -> Result<T, String> {
    vec_decode_with_nice_error::<NoCustomExtension, T>(buf, depth_limit)
}

impl CustomValueKind for NoCustomValueKind {
    fn as_u8(&self) -> u8 {
        panic!("No custom type")
    }

    fn from_u8(_id: u8) -> Option<Self> {
        None
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

    fn read_custom_value_body<'de, R>(
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
        ExpectedStart::PayloadPrefix(BASIC_SBOR_V1_PAYLOAD_PREFIX),
        VecTraverserConfig {
            max_depth: BASIC_SBOR_V1_MAX_DEPTH,
            check_exact_end: true,
        },
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum NoCustomTypeKind {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sbor)]
pub enum NoCustomTypeKindLabel {}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum NoCustomTypeValidation {}

impl CustomTypeValidation for NoCustomTypeValidation {
    fn compare(_base: &Self, _compared: &Self) -> ValidationChange {
        unreachable!("No custom validations exist")
    }
}

impl<L: SchemaTypeLink> CustomTypeKind<L> for NoCustomTypeKind {
    type CustomTypeValidation = NoCustomTypeValidation;
    type CustomTypeKindLabel = NoCustomTypeKindLabel;

    fn label(&self) -> Self::CustomTypeKindLabel {
        unreachable!("No custom type kinds exist")
    }
}

impl CustomTypeKindLabel for NoCustomTypeKindLabel {
    fn name(&self) -> &'static str {
        unreachable!("No custom type kinds exist")
    }
}

lazy_static::lazy_static! {
    static ref EMPTY_SCHEMA: Schema<NoCustomSchema> = {
        Schema::empty()
    };
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum NoCustomSchema {}

impl CustomSchema for NoCustomSchema {
    type CustomLocalTypeKind = NoCustomTypeKind;
    type CustomAggregatorTypeKind = NoCustomTypeKind;
    type CustomTypeKindLabel = NoCustomTypeKindLabel;
    type CustomTypeValidation = NoCustomTypeValidation;
    type DefaultCustomExtension = NoCustomExtension;

    fn linearize_type_kind(
        _: Self::CustomAggregatorTypeKind,
        _: &IndexSet<TypeHash>,
    ) -> Self::CustomLocalTypeKind {
        unreachable!("No custom type kinds exist")
    }

    fn resolve_well_known_type(
        well_known_id: WellKnownTypeId,
    ) -> Option<&'static LocalTypeData<Self>> {
        WELL_KNOWN_LOOKUP
            .get(well_known_id.as_index())
            .and_then(|x| x.as_ref())
    }

    fn validate_custom_type_validation(
        _: &SchemaContext,
        _: &Self::CustomLocalTypeKind,
        _: &Self::CustomTypeValidation,
    ) -> Result<(), SchemaValidationError> {
        unreachable!("No custom type validation")
    }

    fn validate_custom_type_kind(
        _: &SchemaContext,
        _: &Self::CustomLocalTypeKind,
    ) -> Result<(), SchemaValidationError> {
        unreachable!("No custom type kinds exist")
    }

    fn validate_type_metadata_with_custom_type_kind(
        _: &SchemaContext,
        _: &Self::CustomLocalTypeKind,
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

create_well_known_lookup!(
    WELL_KNOWN_LOOKUP,
    well_known_basic_custom_types,
    NoCustomTypeKind,
    []
);

impl CustomExtension for NoCustomExtension {
    const PAYLOAD_PREFIX: u8 = BASIC_SBOR_V1_PAYLOAD_PREFIX;
    type CustomValueKind = NoCustomValueKind;
    type CustomTraversal = NoCustomTraversal;
    type CustomSchema = NoCustomSchema;

    fn custom_value_kind_matches_type_kind(
        _: &Schema<Self::CustomSchema>,
        _: Self::CustomValueKind,
        _: &TypeKind<<Self::CustomSchema as CustomSchema>::CustomLocalTypeKind, LocalTypeId>,
    ) -> bool {
        unreachable!("No custom value kinds exist")
    }

    fn custom_type_kind_matches_non_custom_value_kind(
        _: &Schema<Self::CustomSchema>,
        _: &<Self::CustomSchema as CustomSchema>::CustomLocalTypeKind,
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
pub type BasicLocalTypeKind = LocalTypeKind<NoCustomSchema>;
pub type BasicAggregatorTypeKind = AggregatorTypeKind<NoCustomSchema>;
pub type BasicSchema = Schema<NoCustomSchema>;
pub type BasicVersionedSchema = VersionedSchema<NoCustomSchema>;
pub type BasicTypeData<L> = TypeData<NoCustomTypeKind, L>;
pub type BasicLocalTypeData = LocalTypeData<NoCustomSchema>;
pub type BasicAggregatorTypeData = LocalTypeData<NoCustomSchema>;
pub type BasicTypeAggregator = TypeAggregator<NoCustomTypeKind>;

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

    fn code_generation_string_content<'s, 'de, 'a, 't, 's1, 's2, F: fmt::Write>(
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
        _: LocalTypeId,
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
mod serde_serialization {
    use super::*;

    impl SerializableCustomExtension for NoCustomExtension {
        fn map_value_for_serialization<'s, 'de, 'a, 't, 's1, 's2>(
            _: &SerializationContext<'s, 'a, Self>,
            _: LocalTypeId,
            _: <Self::CustomTraversal as CustomTraversal>::CustomTerminalValueRef<'de>,
        ) -> CustomTypeSerialization<'a, 't, 'de, 's1, 's2, Self> {
            unreachable!("No custom values exist")
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::rust::prelude::*;
    use crate::{
        basic_decode_with_depth_limit, basic_encode_with_depth_limit, BasicValue, BasicValueKind,
    };

    #[test]
    fn depth_counting() {
        // DEPTH(vec![]) = 1
        // * 1 => Vec
        //
        // DEPTH(vec![1u32]) = 2
        // * 1 => Vec
        // * 2 => u32

        let depth1 = BasicValue::Array {
            element_value_kind: BasicValueKind::U32,
            elements: vec![],
        };
        let depth2 = BasicValue::Array {
            element_value_kind: BasicValueKind::U32,
            elements: vec![BasicValue::U32 { value: 1 }],
        };

        // encode
        assert!(basic_encode_with_depth_limit(&depth1, 0).is_err());
        assert!(basic_encode_with_depth_limit(&depth1, 1).is_ok());
        assert!(basic_encode_with_depth_limit(&depth1, 2).is_ok());
        assert!(basic_encode_with_depth_limit(&depth2, 0).is_err());
        assert!(basic_encode_with_depth_limit(&depth2, 1).is_err());
        assert!(basic_encode_with_depth_limit(&depth2, 2).is_ok());

        let buffer1 = basic_encode_with_depth_limit(&depth1, 128).unwrap();
        let buffer2 = basic_encode_with_depth_limit(&depth2, 128).unwrap();

        // decode
        assert!(basic_decode_with_depth_limit::<Vec<u32>>(&buffer1, 0).is_err());
        assert!(basic_decode_with_depth_limit::<Vec<u32>>(&buffer1, 1).is_ok());
        assert!(basic_decode_with_depth_limit::<Vec<u32>>(&buffer1, 2).is_ok());
        assert!(basic_decode_with_depth_limit::<Vec<u32>>(&buffer2, 0).is_err());
        assert!(basic_decode_with_depth_limit::<Vec<u32>>(&buffer2, 1).is_err());
        assert!(basic_decode_with_depth_limit::<Vec<u32>>(&buffer2, 2).is_ok());
    }
}
