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
    ContainerStart(LocalTypeIndex, ContainerHeader<C>),
    ContainerEnd(LocalTypeIndex, ContainerHeader<C>),
    TerminalValue(LocalTypeIndex, TerminalValueRef<'de, C>),
    TerminalValueBatch(LocalTypeIndex, TerminalValueBatchRef<'de, C>),
    End,
    Error(TypedTraversalError<C::CustomValueKind>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedLocation<'t, 's, C: CustomTraversal> {
    pub location: Location<'t, C>,
    /// The path of container types from the root to the current value.
    /// If the event is ContainerStart/End, this does not include the newly started/ended container.
    pub typed_ancestor_path: &'t [ContainerType<'s>],
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
