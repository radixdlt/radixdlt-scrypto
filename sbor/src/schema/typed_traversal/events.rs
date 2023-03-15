use super::*;
use crate::traversal::*;
use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedLocatedTraversalEvent<'t, 's, 'de, C: CustomTraversal> {
    pub location: TypedLocation<'t, 's, C>,
    pub event: TypedTraversalEvent<'de, C>,
}

impl<'t, 's, 'de, C: CustomTraversal> TypedLocatedTraversalEvent<'t, 's, 'de, C> {
    pub fn full_location(&self) -> FullLocation<'s, C> {
        FullLocation {
            start_offset: self.location.location.start_offset,
            end_offset: self.location.location.end_offset,
            ancestor_path: self
                .location
                .location
                .ancestor_path
                .iter()
                .cloned()
                .zip(self.location.typed_ancestor_path.iter().cloned())
                .collect(),
            current_value_info: self.event.current_value_info(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedTraversalEvent<'de, C: CustomTraversal> {
    PayloadPrefix,
    ContainerStart(LocalTypeIndex, ContainerHeader<C>),
    ContainerEnd(LocalTypeIndex, ContainerHeader<C>),
    TerminalValue(LocalTypeIndex, TerminalValueRef<'de, C>),
    TerminalValueBatch(LocalTypeIndex, TerminalValueBatchRef<'de>),
    End,
    Error(TypedTraversalError<C::CustomValueKind>),
}

impl<'de, C: CustomTraversal> TypedTraversalEvent<'de, C> {
    pub fn current_value_info(&self) -> Option<CurrentValueInfo<C::CustomValueKind>> {
        match self {
            TypedTraversalEvent::PayloadPrefix => None,
            TypedTraversalEvent::ContainerStart(type_index, header) => {
                Some(CurrentValueInfo::for_container(*type_index, header))
            }
            TypedTraversalEvent::ContainerEnd(type_index, header) => {
                Some(CurrentValueInfo::for_container(*type_index, header))
            }
            TypedTraversalEvent::TerminalValue(type_index, value_ref) => Some(
                CurrentValueInfo::for_value(*type_index, value_ref.value_kind()),
            ),
            TypedTraversalEvent::TerminalValueBatch(type_index, value_batch_ref) => Some(
                CurrentValueInfo::for_value(*type_index, value_batch_ref.value_kind()),
            ),
            TypedTraversalEvent::End => None,
            TypedTraversalEvent::Error(TypedTraversalError::DecodeError(_)) => None,
            TypedTraversalEvent::Error(TypedTraversalError::TypeIndexNotFound(_)) => None,
            TypedTraversalEvent::Error(TypedTraversalError::ValueMismatchWithType(
                type_mismatch_error,
            )) => match type_mismatch_error {
                TypeMismatchError::MismatchingType { expected, actual } => {
                    Some(CurrentValueInfo::for_value(*expected, *actual))
                }
                TypeMismatchError::MismatchingChildElementType { expected, actual } => {
                    Some(CurrentValueInfo::for_value(*expected, *actual))
                }
                TypeMismatchError::MismatchingChildKeyType { expected, actual } => {
                    Some(CurrentValueInfo::for_value(*expected, *actual))
                }
                TypeMismatchError::MismatchingChildValueType { expected, actual } => {
                    Some(CurrentValueInfo::for_value(*expected, *actual))
                }
                TypeMismatchError::MismatchingTupleLength { type_index, .. } => {
                    Some(CurrentValueInfo::for_value(*type_index, ValueKind::Tuple))
                }
                TypeMismatchError::MismatchingEnumVariantLength {
                    variant,
                    type_index,
                    ..
                } => Some(CurrentValueInfo::for_enum_variant(*type_index, *variant)),
                TypeMismatchError::UnknownEnumVariant {
                    type_index,
                    variant,
                } => Some(CurrentValueInfo::for_enum_variant(*type_index, *variant)),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentValueInfo<X: CustomValueKind> {
    pub type_index: LocalTypeIndex,
    pub value_kind: ValueKind<X>,
    pub variant: Option<u8>,
}

impl<X: CustomValueKind> CurrentValueInfo<X> {
    pub fn for_value(type_index: LocalTypeIndex, value_kind: ValueKind<X>) -> Self {
        Self {
            type_index,
            value_kind,
            variant: None,
        }
    }

    pub fn for_container<C: CustomTraversal>(
        type_index: LocalTypeIndex,
        container_header: &ContainerHeader<C>,
    ) -> Self {
        let (value_kind, variant) = match container_header {
            ContainerHeader::Tuple(_) => (ValueKind::Tuple, None),
            ContainerHeader::EnumVariant(header) => (ValueKind::Enum, Some(header.variant)),
            ContainerHeader::Array(_) => (ValueKind::Array, None),
            ContainerHeader::Map(_) => (ValueKind::Map, None),
        };
        Self {
            type_index,
            value_kind,
            variant,
        }
    }

    pub fn for_enum_variant(type_index: LocalTypeIndex, variant: u8) -> Self {
        Self {
            type_index,
            value_kind: ValueKind::Enum,
            variant: Some(variant),
        }
    }
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
