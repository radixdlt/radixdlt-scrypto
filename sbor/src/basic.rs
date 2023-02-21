use crate::decoder::PayloadTraverser;
use crate::rust::collections::*;
use crate::rust::vec::Vec;
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

pub const DEFAULT_BASIC_MAX_DEPTH: u8 = 64;
pub type BasicEncoder<'a> = VecEncoder<'a, NoCustomValueKind>;
pub type BasicDecoder<'a> = VecDecoder<'a, NoCustomValueKind>;
pub type BasicTraverser<'a> = VecTraverser<'a, NoCustomTraversal>;
pub type BasicValue = Value<NoCustomValueKind, NoCustomValue>;
pub type BasicValueKind = ValueKind<NoCustomValueKind>;

// 5b for (basic) [5b]or - (90 in decimal)
pub const BASIC_SBOR_V1_PAYLOAD_PREFIX: u8 = 0x5b;

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
    let encoder = BasicEncoder::new(&mut buf, DEFAULT_BASIC_MAX_DEPTH);
    encoder.encode_payload(v, BASIC_SBOR_V1_PAYLOAD_PREFIX)?;
    Ok(buf)
}

/// Decode an instance of `T` from a slice.
pub fn basic_decode<T: BasicDecode>(buf: &[u8]) -> Result<T, DecodeError> {
    BasicDecoder::new(buf, DEFAULT_BASIC_MAX_DEPTH).decode_payload(BASIC_SBOR_V1_PAYLOAD_PREFIX)
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

impl CustomTerminalValueRef for NoCustomTerminalValueRef {}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum NoCustomTerminalValueBatchRef {}

impl CustomTerminalValueBatchRef for NoCustomTerminalValueBatchRef {}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum NoCustomContainerHeader {}

impl CustomContainerHeader for NoCustomContainerHeader {
    type CustomValueKind = NoCustomValueKind;

    fn get_child_count(&self) -> usize {
        unreachable!("NoCustomContainerHeader can't exist")
    }

    fn get_implicit_child_value_kind(&self, _: usize) -> Option<ValueKind<Self::CustomValueKind>> {
        unreachable!("NoCustomContainerHeader can't exist")
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum NoCustomTraversal {}

impl CustomTraversal for NoCustomTraversal {
    type CustomValueKind = NoCustomValueKind;
    type CustomTerminalValueRef = NoCustomTerminalValueRef;
    type CustomTerminalValueBatchRef = NoCustomTerminalValueBatchRef;
    type CustomContainerHeader = NoCustomContainerHeader;
    type CustomValueTraverser = NoCustomTraverser;

    fn new_value_traversal(_: Self::CustomValueKind, _: u8, _: u8) -> Self::CustomValueTraverser {
        unreachable!("The NoCustomValueKind parameter can't exist")
    }
}

pub enum NoCustomTraverser {}

impl CustomValueTraverser for NoCustomTraverser {
    type CustomTraversal = NoCustomTraversal;

    fn next_event<
        'de,
        R: PayloadTraverser<'de, <Self::CustomTraversal as CustomTraversal>::CustomValueKind>,
    >(
        &mut self,
        _: &mut R,
    ) -> TraversalEvent<'de, Self::CustomTraversal> {
        unreachable!("NoCustomTraverser can't exist")
    }
}

/// Creates a traverser from the buffer
pub fn basic_traverser<'b>(buf: &'b [u8]) -> Result<BasicTraverser<'b>, DecodeError> {
    let mut traverser = BasicTraverser::new(buf, DEFAULT_BASIC_MAX_DEPTH);
    traverser.read_and_check_payload_prefix(BASIC_SBOR_V1_PAYLOAD_PREFIX)?;
    Ok(traverser)
}

pub use schema::*;

mod schema {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Sbor)]
    pub enum NoCustomTypeKind {}

    impl<L: SchemaTypeLink> CustomTypeKind<L> for NoCustomTypeKind {
        type CustomValueKind = NoCustomValueKind;

        type CustomTypeExtension = NoCustomTypeExtension;
    }

    #[derive(Debug, Clone, PartialEq, Eq, Sbor)]
    pub enum NoCustomTypeValidation {}

    impl CustomTypeValidation for NoCustomTypeValidation {}

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum NoCustomTypeExtension {}

    impl CustomTypeExtension for NoCustomTypeExtension {
        const MAX_DEPTH: u8 = DEFAULT_BASIC_MAX_DEPTH;
        const PAYLOAD_PREFIX: u8 = BASIC_SBOR_V1_PAYLOAD_PREFIX;
        type CustomValueKind = NoCustomValueKind;
        type CustomTypeKind<L: SchemaTypeLink> = NoCustomTypeKind;
        type CustomTypeValidation = NoCustomTypeValidation;
        type CustomTraversal = NoCustomTraversal;

        fn linearize_type_kind(
            _: Self::CustomTypeKind<GlobalTypeId>,
            _: &IndexSet<TypeHash>,
        ) -> Self::CustomTypeKind<LocalTypeIndex> {
            unreachable!("No custom type kinds exist")
        }

        fn resolve_custom_well_known_type(
            _: u8,
        ) -> Option<TypeData<Self::CustomTypeKind<LocalTypeIndex>, LocalTypeIndex>> {
            None
        }

        fn validate_type_kind(
            _: &TypeValidationContext,
            _: &SchemaCustomTypeKind<Self>,
        ) -> Result<(), SchemaValidationError> {
            unreachable!("No custom type kinds exist")
        }

        fn validate_type_metadata_with_type_kind(
            _: &TypeValidationContext,
            _: &SchemaCustomTypeKind<Self>,
            _: &TypeMetadata,
        ) -> Result<(), SchemaValidationError> {
            unreachable!("No custom type kinds exist")
        }

        fn validate_type_validation_with_type_kind(
            _: &TypeValidationContext,
            _: &SchemaCustomTypeKind<Self>,
            _: &SchemaCustomTypeValidation<Self>,
        ) -> Result<(), SchemaValidationError> {
            unreachable!("No custom type kinds exist")
        }
    }

    pub type BasicTypeKind<L> = TypeKind<NoCustomValueKind, NoCustomTypeKind, L>;
    pub type BasicSchema = Schema<NoCustomTypeExtension>;
}
