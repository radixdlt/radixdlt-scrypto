use crate::rust::fmt;
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
    ContainerStart(LocalTypeId, ContainerHeader<E::CustomTraversal>),
    ContainerEnd(LocalTypeId, ContainerHeader<E::CustomTraversal>),
    TerminalValue(LocalTypeId, TerminalValueRef<'de, E::CustomTraversal>),
    TerminalValueBatch(LocalTypeId, TerminalValueBatchRef<'de>),
    End,
    Error(TypedTraversalError<E>),
}

impl<'de, E: CustomExtension> TypedTraversalEvent<'de, E> {
    pub fn current_value_info(&self) -> Option<CurrentValueInfo<E>> {
        match self {
            TypedTraversalEvent::ContainerStart(type_id, header) => {
                Some(CurrentValueInfo::for_container(*type_id, header))
            }
            TypedTraversalEvent::ContainerEnd(type_id, header) => {
                Some(CurrentValueInfo::for_container(*type_id, header))
            }
            TypedTraversalEvent::TerminalValue(type_id, value_ref) => Some(
                CurrentValueInfo::for_value(*type_id, value_ref.value_kind()),
            ),
            TypedTraversalEvent::TerminalValueBatch(type_id, value_batch_ref) => Some(
                CurrentValueInfo::for_value(*type_id, value_batch_ref.value_kind()),
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
                TypeMismatchError::MismatchingTupleLength { type_id, .. } => {
                    Some(CurrentValueInfo::for_value(*type_id, ValueKind::Tuple))
                }
                TypeMismatchError::MismatchingEnumVariantLength {
                    variant, type_id, ..
                } => Some(CurrentValueInfo::for_enum_variant(*type_id, *variant)),
                TypeMismatchError::UnknownEnumVariant { type_id, variant } => {
                    Some(CurrentValueInfo::for_enum_variant(*type_id, *variant))
                }
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentValueInfo<E: CustomExtension> {
    pub type_id: LocalTypeId,
    pub value_kind: ValueKind<E::CustomValueKind>,
    pub variant: Option<u8>,
    pub error: Option<TypedTraversalError<E>>,
}

impl<E: CustomExtension> CurrentValueInfo<E> {
    pub fn for_value(type_id: LocalTypeId, value_kind: ValueKind<E::CustomValueKind>) -> Self {
        Self {
            type_id,
            error: None,
            value_kind,
            variant: None,
        }
    }

    pub fn for_container(
        type_id: LocalTypeId,
        container_header: &ContainerHeader<E::CustomTraversal>,
    ) -> Self {
        let (value_kind, variant) = match container_header {
            ContainerHeader::Tuple(_) => (ValueKind::Tuple, None),
            ContainerHeader::EnumVariant(header) => (ValueKind::Enum, Some(header.variant)),
            ContainerHeader::Array(_) => (ValueKind::Array, None),
            ContainerHeader::Map(_) => (ValueKind::Map, None),
        };
        Self {
            type_id,
            error: None,
            value_kind,
            variant,
        }
    }

    pub fn for_enum_variant(type_id: LocalTypeId, variant: u8) -> Self {
        Self {
            type_id,
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
    TypeIndexNotFound(LocalTypeId),
    ValueMismatchWithType(TypeMismatchError<E>),
    DecodeError(DecodeError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeMismatchError<E: CustomExtension> {
    MismatchingType {
        expected_type_id: LocalTypeId,
        expected_type_kind:
            TypeKind<<E::CustomSchema as CustomSchema>::CustomTypeKind<LocalTypeId>, LocalTypeId>,
        actual_value_kind: ValueKind<E::CustomValueKind>,
    },
    MismatchingChildElementType {
        expected_type_id: LocalTypeId,
        expected_type_kind:
            TypeKind<<E::CustomSchema as CustomSchema>::CustomTypeKind<LocalTypeId>, LocalTypeId>,
        actual_value_kind: ValueKind<E::CustomValueKind>,
    },
    MismatchingChildKeyType {
        expected_type_id: LocalTypeId,
        expected_type_kind:
            TypeKind<<E::CustomSchema as CustomSchema>::CustomTypeKind<LocalTypeId>, LocalTypeId>,
        actual_value_kind: ValueKind<E::CustomValueKind>,
    },
    MismatchingChildValueType {
        expected_type_id: LocalTypeId,
        expected_type_kind:
            TypeKind<<E::CustomSchema as CustomSchema>::CustomTypeKind<LocalTypeId>, LocalTypeId>,
        actual_value_kind: ValueKind<E::CustomValueKind>,
    },
    MismatchingTupleLength {
        expected: usize,
        actual: usize,
        type_id: LocalTypeId,
    },
    MismatchingEnumVariantLength {
        expected: usize,
        actual: usize,
        variant: u8,
        type_id: LocalTypeId,
    },
    UnknownEnumVariant {
        type_id: LocalTypeId,
        variant: u8,
    },
}

impl<E: CustomExtension> fmt::Display for TypedTraversalError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypedTraversalError::ValueMismatchWithType(TypeMismatchError::MismatchingType {
                expected_type_kind,
                actual_value_kind,
                ..
            }) => {
                write!(
                    f,
                    "{{ expected_type: {:?}, found: {:?} }}",
                    expected_type_kind, actual_value_kind
                )
            }
            TypedTraversalError::ValueMismatchWithType(
                TypeMismatchError::MismatchingChildElementType {
                    expected_type_kind,
                    actual_value_kind,
                    ..
                },
            ) => {
                write!(
                    f,
                    "{{ expected_child_type: {:?}, found: {:?} }}",
                    expected_type_kind, actual_value_kind
                )
            }
            TypedTraversalError::ValueMismatchWithType(
                TypeMismatchError::MismatchingChildKeyType {
                    expected_type_kind,
                    actual_value_kind,
                    ..
                },
            ) => {
                write!(
                    f,
                    "{{ expected_key_type: {:?}, found: {:?} }}",
                    expected_type_kind, actual_value_kind
                )
            }
            TypedTraversalError::ValueMismatchWithType(
                TypeMismatchError::MismatchingChildValueType {
                    expected_type_kind,
                    actual_value_kind,
                    ..
                },
            ) => {
                write!(
                    f,
                    "{{ expected_value_type: {:?}, found: {:?} }}",
                    expected_type_kind, actual_value_kind
                )
            }
            TypedTraversalError::ValueMismatchWithType(
                TypeMismatchError::MismatchingTupleLength {
                    expected, actual, ..
                },
            ) => {
                write!(
                    f,
                    "{{ expected_field_count: {:?}, found: {:?} }}",
                    expected, actual
                )
            }
            TypedTraversalError::ValueMismatchWithType(
                TypeMismatchError::MismatchingEnumVariantLength {
                    expected, actual, ..
                },
            ) => {
                write!(
                    f,
                    "{{ expected_field_count: {:?}, found: {:?} }}",
                    expected, actual
                )
            }
            TypedTraversalError::ValueMismatchWithType(TypeMismatchError::UnknownEnumVariant {
                variant,
                ..
            }) => {
                write!(f, "{{ unknown_variant_id: {:?} }}", variant)
            }
            TypedTraversalError::TypeIndexNotFound(_) | TypedTraversalError::DecodeError(_) => {
                write!(f, "{:?}", self)
            }
        }
    }
}
