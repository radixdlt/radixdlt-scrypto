use crate::traversal::*;
use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedTraversalEvent<'de, C: CustomTraversal> {
    ContainerStart(TypedLocatedDecoding<ContainerHeader<C::CustomContainerHeader>>),
    ContainerEnd(TypedLocatedDecoding<ContainerHeader<C::CustomContainerHeader>>),
    TerminalValue(TypedLocatedDecoding<TerminalValueRef<'de, C::CustomTerminalValueRef<'de>>>),
    TerminalValueBatch(
        TypedLocatedDecoding<TerminalValueBatchRef<'de, C::CustomTerminalValueBatchRef<'de>>>,
    ),
    PayloadEnd(Location),
    Error(LocatedError<TypedTraversalError<C::CustomValueKind>>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypedTraversalError<X: CustomValueKind> {
    TypeNotFound(LocalTypeIndex),
    TypeMismatch(TypeMismatchError<X>),
    DecodeError(DecodeError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeMismatchError<X: CustomValueKind> {
    MismatchingType {
        expected: LocalTypeIndex,
        actual: ValueKind<X>,
    },
    MismatchingChildElementType {
        expected: LocalTypeIndex,
        actual: ValueKind<X>,
    },
    MismatchingChildKeyType {
        expected: LocalTypeIndex,
        actual: ValueKind<X>,
    },
    MismatchingChildValueType {
        expected: LocalTypeIndex,
        actual: ValueKind<X>,
    },
    MismatchingTupleLength {
        expected: usize,
        actual: usize,
        type_index: LocalTypeIndex,
    },
    MismatchingEnumVariantLength {
        expected: usize,
        actual: usize,
        variant: u8,
        type_index: LocalTypeIndex,
    },
    UnknownEnumVariant {
        type_index: LocalTypeIndex,
        variant: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypedLocatedDecoding<T> {
    pub inner: T,
    pub parent_relationship: ParentRelationship,
    pub type_index: LocalTypeIndex,
    pub location: Location,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedTerminalValueRef<'de, V: CustomTerminalValueRef> {
    type_index: LocalTypeIndex,
    value: TerminalValueRef<'de, V>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedTerminalValueBatchRef<'de, B: CustomTerminalValueBatchRef> {
    type_index: LocalTypeIndex,
    value: TerminalValueBatchRef<'de, B>,
}
