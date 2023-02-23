use super::*;
use crate::traversal::*;
use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedTraversalEvent<'t, 's, 'de, C: CustomTraversal> {
    PayloadPrefix(Location),
    ContainerStart(TypedLocatedDecoding<'t, 's, ContainerHeader<C>, C>),
    ContainerEnd(TypedLocatedDecoding<'t, 's, ContainerHeader<C>, C>),
    TerminalValue(
        TypedLocatedDecoding<'t, 's, TerminalValueRef<'de, C::CustomTerminalValueRef<'de>>, C>,
    ),
    TerminalValueBatch(
        TypedLocatedDecoding<
            't,
            's,
            TerminalValueBatchRef<'de, C::CustomTerminalValueBatchRef<'de>>,
            C,
        >,
    ),
    End(Location),
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
pub struct TypedLocatedDecoding<'t, 's, T, C: CustomTraversal> {
    pub inner: T,
    pub parent_relationship: ParentRelationship,
    pub type_index: LocalTypeIndex,
    pub location: Location,
    pub resultant_path: &'t [ContainerChild<C>],
    pub typed_resultant_path: &'t [ContainerType<'s>],
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
