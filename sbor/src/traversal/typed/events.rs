use crate::rust::prelude::*;
use crate::traversal::*;
use crate::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedLocatedTraversalEvent<'t, 's, 'de, E: CustomExtension> {
    pub location: TypedLocation<'t, 's, E::CustomTraversal>,
    pub event: TypedTraversalEvent<'de, E>,
}

impl<'t, 's, 'de, E: CustomExtension> TypedLocatedTraversalEvent<'t, 's, 'de, E> {
    pub fn full_location(&self) -> FullLocation<'s, E> {
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
            error: match &self.event {
                TypedTraversalEvent::Error(error) => Some(error.clone()),
                _ => None,
            },
        }
    }

    pub fn display_as_unexpected_event(
        &self,
        expected: &'static str,
        schema: &Schema<E::CustomSchema>,
    ) -> String {
        let error_display = match &self.event {
            TypedTraversalEvent::Error(error) => format!("{:?} occurred", error),
            _ => format!("Expected {} but found {:?}", expected, self.event),
        };
        let full_location = self.full_location();
        format!(
            "{} at byte offset {}-{} and value path {}",
            error_display,
            full_location.start_offset,
            full_location.end_offset,
            full_location.path_to_string(&schema)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedTraversalEvent<'de, E: CustomExtension> {
    ContainerStart(LocalTypeIndex, ContainerHeader<E::CustomTraversal>),
    ContainerEnd(LocalTypeIndex, ContainerHeader<E::CustomTraversal>),
    TerminalValue(LocalTypeIndex, TerminalValueRef<'de, E::CustomTraversal>),
    TerminalValueBatch(LocalTypeIndex, TerminalValueBatchRef<'de>),
    End,
    Error(TypedTraversalError<E>),
}

impl<'de, E: CustomExtension> TypedTraversalEvent<'de, E> {
    pub fn current_value_info(&self) -> Option<CurrentValueInfo<E>> {
        match self {
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
                // For these, we have a type mismatch - so we can't return accurate information on "current value"
                // Instead, let's handle these when we print the full location
                TypeMismatchError::MismatchingType { .. }
                | TypeMismatchError::MismatchingChildElementType { .. }
                | TypeMismatchError::MismatchingChildKeyType { .. }
                | TypeMismatchError::MismatchingChildValueType { .. } => None,
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
pub struct CurrentValueInfo<E: CustomExtension> {
    pub type_index: LocalTypeIndex,
    pub value_kind: ValueKind<E::CustomValueKind>,
    pub variant: Option<u8>,
    pub error: Option<TypedTraversalError<E>>,
}

impl<E: CustomExtension> CurrentValueInfo<E> {
    pub fn for_value(
        type_index: LocalTypeIndex,
        value_kind: ValueKind<E::CustomValueKind>,
    ) -> Self {
        Self {
            type_index,
            error: None,
            value_kind,
            variant: None,
        }
    }

    pub fn for_container(
        type_index: LocalTypeIndex,
        container_header: &ContainerHeader<E::CustomTraversal>,
    ) -> Self {
        let (value_kind, variant) = match container_header {
            ContainerHeader::Tuple(_) => (ValueKind::Tuple, None),
            ContainerHeader::EnumVariant(header) => (ValueKind::Enum, Some(header.variant)),
            ContainerHeader::Array(_) => (ValueKind::Array, None),
            ContainerHeader::Map(_) => (ValueKind::Map, None),
        };
        Self {
            type_index,
            error: None,
            value_kind,
            variant,
        }
    }

    pub fn for_enum_variant(type_index: LocalTypeIndex, variant: u8) -> Self {
        Self {
            type_index,
            error: None,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedTraversalError<E: CustomExtension> {
    TypeIndexNotFound(LocalTypeIndex),
    ValueMismatchWithType(TypeMismatchError<E>),
    DecodeError(DecodeError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeMismatchError<E: CustomExtension> {
    MismatchingType {
        expected_type_index: LocalTypeIndex,
        expected_type_kind: TypeKind<
            <E::CustomSchema as CustomSchema>::CustomTypeKind<LocalTypeIndex>,
            LocalTypeIndex,
        >,
        actual_value_kind: ValueKind<E::CustomValueKind>,
    },
    MismatchingChildElementType {
        expected_type_index: LocalTypeIndex,
        expected_type_kind: TypeKind<
            <E::CustomSchema as CustomSchema>::CustomTypeKind<LocalTypeIndex>,
            LocalTypeIndex,
        >,
        actual_value_kind: ValueKind<E::CustomValueKind>,
    },
    MismatchingChildKeyType {
        expected_type_index: LocalTypeIndex,
        expected_type_kind: TypeKind<
            <E::CustomSchema as CustomSchema>::CustomTypeKind<LocalTypeIndex>,
            LocalTypeIndex,
        >,
        actual_value_kind: ValueKind<E::CustomValueKind>,
    },
    MismatchingChildValueType {
        expected_type_index: LocalTypeIndex,
        expected_type_kind: TypeKind<
            <E::CustomSchema as CustomSchema>::CustomTypeKind<LocalTypeIndex>,
            LocalTypeIndex,
        >,
        actual_value_kind: ValueKind<E::CustomValueKind>,
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
