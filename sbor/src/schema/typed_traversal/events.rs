use super::*;
use crate::traversal::*;
use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedLocatedTraversalEvent<'t, 's, 'de, C: CustomTraversal> {
    pub location: TypedLocation<'t, 's, C>,
    pub event: TypedTraversalEvent<'de, C>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedTraversalEvent<'de, C: CustomTraversal> {
    PayloadPrefix,
    ContainerStart(TypedContainerHeader<C>),
    ContainerEnd(TypedContainerHeader<C>),
    TerminalValue(TypedTerminalValueRef<'de, C>),
    TerminalValueBatch(TypedTerminalValueBatchRef<'de, C>),
    End,
    Error(TypedTraversalError<C::CustomValueKind>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedLocation<'t, 's, C: CustomTraversal> {
    pub location: Location<'t, C>,
    pub typed_resultant_path: &'t [ContainerType<'s>],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypedTraversalError<X: CustomValueKind> {
    TypeIndexNotFound(LocalTypeIndex),
    ValueMismatchWithType(TypeMismatchError<X>),
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
        actual: u32,
        type_index: LocalTypeIndex,
    },
    MismatchingEnumVariantLength {
        expected: usize,
        actual: u32,
        variant: u8,
        type_index: LocalTypeIndex,
    },
    UnknownEnumVariant {
        type_index: LocalTypeIndex,
        variant: u8,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedContainerHeader<C: CustomTraversal> {
    pub type_index: LocalTypeIndex,
    pub header: ContainerHeader<C>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedTerminalValueRef<'de, C: CustomTraversal> {
    pub type_index: LocalTypeIndex,
    pub value: TerminalValueRef<'de, C>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedTerminalValueBatchRef<'de, C: CustomTraversal> {
    pub type_index: LocalTypeIndex,
    pub value_batch: TerminalValueBatchRef<'de, C>,
}
