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
            ancestor_path: self.location.typed_ancestor_path(),
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
            TypedTraversalEvent::Error(TypedTraversalError::TypeIdNotFound(_)) => None,
            TypedTraversalEvent::Error(TypedTraversalError::ValueMismatchWithType(
                type_mismatch_error,
            )) => match type_mismatch_error {
                // For these, we have a type mismatch - so it's not 100% clear what we should display as "current value".
                // It probably makes sense to show the expected type name in the error message, but this will require some
                // refactoring (e.g. adding the parent type to the Mismatching Child errors, and replacing CurrentValueInfo
                // with something like AnnotatedSborPartialLeaf)
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
pub struct TypedLocation<'t, 's, T: CustomTraversal> {
    pub location: Location<'t, T>,
    /// The path of container types from the root to the current value.
    /// If the event is ContainerStart/End, this does not include the newly started/ended container.
    ///
    /// NOTE: This list includes types for newly read container headers _before_ any children are read,
    /// which is before the Location adds them to `ancestor_path`. So in some instances this `typed_container_path`
    /// may be strictly longer than `location.ancestor_path`.
    pub typed_container_path: &'t [ContainerType<'s>],
}

impl<'t, 's, T: CustomTraversal> TypedLocation<'t, 's, T> {
    pub fn typed_ancestor_path(&self) -> Vec<(AncestorState<T>, ContainerType<'s>)> {
        let untyped_ancestor_path = self.location.ancestor_path;

        // As per the note on `typed_container_path`, it can be longer than the ancestor list.
        // But zip will end when the shortest iterator ends, so this will correct only return the types of
        // the full ancestors.
        untyped_ancestor_path
            .iter()
            .cloned()
            .zip(self.typed_container_path.iter().cloned())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedTraversalError<E: CustomExtension> {
    TypeIdNotFound(LocalTypeId),
    ValueMismatchWithType(TypeMismatchError<E>),
    DecodeError(DecodeError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeMismatchError<E: CustomExtension> {
    MismatchingType {
        type_id: LocalTypeId,
        expected_type_kind: TypeKindLabel<<E::CustomSchema as CustomSchema>::CustomTypeKindLabel>,
        actual_value_kind: ValueKind<E::CustomValueKind>,
    },
    MismatchingChildElementType {
        type_id: LocalTypeId,
        expected_type_kind: TypeKindLabel<<E::CustomSchema as CustomSchema>::CustomTypeKindLabel>,
        actual_value_kind: ValueKind<E::CustomValueKind>,
    },
    MismatchingChildKeyType {
        type_id: LocalTypeId,
        expected_type_kind: TypeKindLabel<<E::CustomSchema as CustomSchema>::CustomTypeKindLabel>,
        actual_value_kind: ValueKind<E::CustomValueKind>,
    },
    MismatchingChildValueType {
        type_id: LocalTypeId,
        expected_type_kind: TypeKindLabel<<E::CustomSchema as CustomSchema>::CustomTypeKindLabel>,
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
            TypedTraversalError::TypeIdNotFound(_) | TypedTraversalError::DecodeError(_) => {
                write!(f, "{:?}", self)
            }
        }
    }
}
