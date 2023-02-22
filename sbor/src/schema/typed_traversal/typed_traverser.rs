use super::*;
use crate::basic_well_known_types::ANY_ID;
use crate::rust::prelude::*;
use crate::traversal::*;
use crate::*;

pub fn traverse_payload_with_types<'de, 's, E: CustomTypeExtension>(
    payload: &'de [u8],
    type_kinds: &'s [SchemaTypeKind<E>],
    index: LocalTypeIndex,
) -> TypedTraverser<'de, 's, E> {
    TypedTraverser::new(
        payload,
        type_kinds,
        index,
        E::MAX_DEPTH,
        Some(E::PAYLOAD_PREFIX),
        true,
    )
}

/// The `TypedTraverser` is for streamed decoding of a payload with type kinds.
///
/// It validates that the payload matches the given type kinds,
/// and adds the relevant type index to the events which are output.
pub struct TypedTraverser<'de, 's, E: CustomTypeExtension> {
    inner: VecTraverser<'de, E::CustomTraversal>,
    container_stack: Vec<ContainerType<'s>>,
    schema_type_kinds: &'s [SchemaTypeKind<E>],
    root_type_index: LocalTypeIndex,
}

pub struct ContainerType<'s> {
    pub own_type: LocalTypeIndex,
    pub child_types: ContainerChildTypeRefs<'s>,
}

pub enum ContainerChildTypeRefs<'s> {
    Tuple(&'s [LocalTypeIndex]),
    EnumVariant(&'s [LocalTypeIndex]),
    Array(LocalTypeIndex),
    Map(LocalTypeIndex, LocalTypeIndex),
    Any,
}

impl<'s> ContainerChildTypeRefs<'s> {
    pub fn get_child_type_for_element(&self, index: usize) -> Option<LocalTypeIndex> {
        match self {
            ContainerChildTypeRefs::Tuple(types) => (*types).get(index).copied(),
            ContainerChildTypeRefs::EnumVariant(types) => (*types).get(index).copied(),
            ContainerChildTypeRefs::Array(child_type) => Some(*child_type),
            ContainerChildTypeRefs::Any => Some(LocalTypeIndex::WellKnown(ANY_ID)),
            _ => None,
        }
    }

    pub fn get_child_type_for_map_key(&self) -> Option<LocalTypeIndex> {
        match self {
            ContainerChildTypeRefs::Map(key_type, _) => Some(*key_type),
            ContainerChildTypeRefs::Any => Some(LocalTypeIndex::WellKnown(ANY_ID)),
            _ => None,
        }
    }

    pub fn get_child_type_for_map_value(&self) -> Option<LocalTypeIndex> {
        match self {
            ContainerChildTypeRefs::Map(_, value_type) => Some(*value_type),
            ContainerChildTypeRefs::Any => Some(LocalTypeIndex::WellKnown(ANY_ID)),
            _ => None,
        }
    }
}

type ContainerHeaderFor<E> = ContainerHeader<
    <<E as CustomTypeExtension>::CustomTraversal as CustomTraversal>::CustomContainerHeader,
>;
type TerminalValueFor<'de, E> = TerminalValueRef<
    'de,
    <<E as CustomTypeExtension>::CustomTraversal as CustomTraversal>::CustomTerminalValueRef<'de>,
>;
type TerminalValueBatchRefFor<'de, E> = TerminalValueBatchRef<
    'de,
    <<E as CustomTypeExtension>::CustomTraversal as CustomTraversal>::CustomTerminalValueBatchRef<
        'de,
    >,
>;
type TypeKindFor<E> = TypeKind<
    <E as CustomTypeExtension>::CustomValueKind,
    <E as CustomTypeExtension>::CustomTypeKind<LocalTypeIndex>,
    LocalTypeIndex,
>;

#[macro_export]
macro_rules! return_type_mismatch_error {
    ($location: ident, $error: expr) => {{
        return TypedTraversalEvent::Error(LocatedError {
            error: TypedTraversalError::TypeMismatch($error),
            location: $location,
        });
    }};
}

#[macro_export]
macro_rules! look_up_type {
    ($self: ident, $e: ident, $location: expr, $type_index: expr) => {
        match resolve_type_kind::<$e>($self.schema_type_kinds, $type_index) {
            Some(resolved_type) => resolved_type,
            None => {
                return TypedTraversalEvent::Error(LocatedError {
                    error: TypedTraversalError::TypeNotFound($type_index),
                    location: $location,
                })
            }
        }
    };
}

impl<'de, 's, E: CustomTypeExtension> TypedTraverser<'de, 's, E> {
    pub fn new(
        input: &'de [u8],
        type_kinds: &'s [SchemaTypeKind<E>],
        type_index: LocalTypeIndex,
        max_depth: u8,
        payload_prefix: Option<u8>,
        check_exact_end: bool,
    ) -> Self {
        Self {
            inner: VecTraverser::new(input, max_depth, payload_prefix, check_exact_end),
            container_stack: Vec::with_capacity(max_depth as usize),
            schema_type_kinds: type_kinds,
            root_type_index: type_index,
        }
    }

    pub fn next_event(&mut self) -> TypedTraversalEvent<'de, E::CustomTraversal> {
        let inner_event = self.inner.next_event();
        match inner_event {
            TraversalEvent::PayloadPrefix(location) => TypedTraversalEvent::PayloadPrefix(location),
            TraversalEvent::ContainerStart(located_event) => {
                self.map_container_start_event(located_event)
            }
            TraversalEvent::TerminalValue(located_event) => {
                self.map_terminal_value_event(located_event)
            }
            TraversalEvent::TerminalValueBatch(located_event) => {
                self.map_terminal_value_batch_event(located_event)
            }
            TraversalEvent::ContainerEnd(located_event) => {
                self.map_container_end_event(located_event)
            }
            TraversalEvent::End(location) => TypedTraversalEvent::End(location),
            TraversalEvent::DecodeError(located_event) => {
                Self::map_decode_error_event(located_event)
            }
        }
    }

    fn map_container_start_event(
        &mut self,
        located_event: LocatedDecoding<ContainerHeaderFor<E>>,
    ) -> TypedTraversalEvent<'de, E::CustomTraversal> {
        let LocatedDecoding {
            inner: header,
            parent_relationship,
            location,
        } = located_event;
        let type_index = self.get_type_index(&parent_relationship);

        let container_type = look_up_type!(self, E, location, type_index);

        match header {
            ContainerHeader::Tuple(TupleHeader { length }) => match container_type {
                TypeKind::Any => self.container_stack.push(ContainerType {
                    own_type: type_index,
                    child_types: ContainerChildTypeRefs::Any,
                }),
                TypeKind::Tuple { field_types } if field_types.len() == length => {
                    self.container_stack.push(ContainerType {
                        own_type: type_index,
                        child_types: ContainerChildTypeRefs::Tuple(field_types),
                    })
                }
                TypeKind::Tuple { field_types } => return_type_mismatch_error!(
                    location,
                    TypeMismatchError::MismatchingTupleLength {
                        expected: field_types.len(),
                        actual: length,
                        type_index
                    }
                ),
                _ => return_type_mismatch_error!(
                    location,
                    TypeMismatchError::MismatchingType {
                        expected: type_index,
                        actual: ValueKind::Tuple
                    }
                ),
            },
            ContainerHeader::EnumVariant(EnumVariantHeader { variant, length }) => {
                match container_type {
                    TypeKind::Any => self.container_stack.push(ContainerType {
                        own_type: type_index,
                        child_types: ContainerChildTypeRefs::Any,
                    }),
                    TypeKind::Enum { variants } => match variants.get(&variant) {
                        Some(variant_child_types) if variant_child_types.len() == length => {
                            self.container_stack.push(ContainerType {
                                own_type: type_index,
                                child_types: ContainerChildTypeRefs::EnumVariant(
                                    variant_child_types,
                                ),
                            })
                        }
                        Some(variant_child_types) => return_type_mismatch_error!(
                            location,
                            TypeMismatchError::MismatchingEnumVariantLength {
                                expected: variant_child_types.len(),
                                actual: length,
                                type_index,
                                variant
                            }
                        ),
                        None => return_type_mismatch_error!(
                            location,
                            TypeMismatchError::UnknownEnumVariant {
                                type_index,
                                variant
                            }
                        ),
                    },
                    _ => return_type_mismatch_error!(
                        location,
                        TypeMismatchError::MismatchingType {
                            expected: type_index,
                            actual: ValueKind::Enum
                        }
                    ),
                }
            }
            ContainerHeader::Array(ArrayHeader {
                element_value_kind, ..
            }) => match container_type {
                TypeKind::Any => self.container_stack.push(ContainerType {
                    own_type: type_index,
                    child_types: ContainerChildTypeRefs::Any,
                }),
                TypeKind::Array {
                    element_type: element_type_index,
                } => {
                    let element_type = look_up_type!(self, E, location, *element_type_index);
                    if !value_kind_matches_type_kind::<E>(element_value_kind, element_type) {
                        return_type_mismatch_error!(
                            location,
                            TypeMismatchError::MismatchingChildElementType {
                                expected: *element_type_index,
                                actual: element_value_kind
                            }
                        )
                    }
                    self.container_stack.push(ContainerType {
                        own_type: type_index,
                        child_types: ContainerChildTypeRefs::Array(*element_type_index),
                    })
                }
                _ => return_type_mismatch_error!(
                    location,
                    TypeMismatchError::MismatchingType {
                        expected: type_index,
                        actual: ValueKind::Array
                    }
                ),
            },
            ContainerHeader::Map(MapHeader {
                key_value_kind,
                value_value_kind,
                ..
            }) => match container_type {
                TypeKind::Any => self.container_stack.push(ContainerType {
                    own_type: type_index,
                    child_types: ContainerChildTypeRefs::Any,
                }),
                TypeKind::Map {
                    key_type: key_type_index,
                    value_type: value_type_index,
                } => {
                    let key_type = look_up_type!(self, E, location, *key_type_index);
                    if !value_kind_matches_type_kind::<E>(key_value_kind, key_type) {
                        return_type_mismatch_error!(
                            location,
                            TypeMismatchError::MismatchingChildKeyType {
                                expected: *key_type_index,
                                actual: key_value_kind
                            }
                        )
                    }
                    let value_type = look_up_type!(self, E, location, *value_type_index);
                    if !value_kind_matches_type_kind::<E>(value_value_kind, value_type) {
                        return_type_mismatch_error!(
                            location,
                            TypeMismatchError::MismatchingChildValueType {
                                expected: *value_type_index,
                                actual: key_value_kind
                            }
                        )
                    }
                    self.container_stack.push(ContainerType {
                        own_type: type_index,
                        child_types: ContainerChildTypeRefs::Map(
                            *key_type_index,
                            *value_type_index,
                        ),
                    })
                }
                _ => return_type_mismatch_error!(
                    location,
                    TypeMismatchError::MismatchingType {
                        expected: type_index,
                        actual: ValueKind::Map
                    }
                ),
            },
            ContainerHeader::Custom(_) => {
                unimplemented!("Custom containers are not yet fully supported")
            }
        }

        TypedTraversalEvent::ContainerStart(TypedLocatedDecoding {
            inner: header,
            parent_relationship,
            type_index,
            location,
        })
    }

    fn map_terminal_value_event(
        &mut self,
        located_event: LocatedDecoding<TerminalValueFor<'de, E>>,
    ) -> TypedTraversalEvent<'de, E::CustomTraversal> {
        let LocatedDecoding {
            inner: value_ref,
            parent_relationship,
            location,
        } = located_event;
        let type_index = self.get_type_index(&parent_relationship);

        let value_kind = value_ref.value_kind();
        let type_kind = look_up_type!(self, E, location, type_index);

        if !value_kind_matches_type_kind::<E>(value_kind, type_kind) {
            return_type_mismatch_error!(
                location,
                TypeMismatchError::MismatchingType {
                    expected: type_index,
                    actual: value_kind
                }
            )
        }

        TypedTraversalEvent::TerminalValue(TypedLocatedDecoding {
            inner: value_ref,
            parent_relationship,
            type_index,
            location,
        })
    }

    fn map_terminal_value_batch_event(
        &mut self,
        located_event: LocatedDecoding<TerminalValueBatchRefFor<'de, E>>,
    ) -> TypedTraversalEvent<'de, E::CustomTraversal> {
        let LocatedDecoding {
            inner: value_batch_ref,
            parent_relationship,
            location,
        } = located_event;
        let type_index = self.get_type_index(&parent_relationship);

        let value_kind = value_batch_ref.value_kind();
        let type_kind = look_up_type!(self, E, location, type_index);

        if !value_kind_matches_type_kind::<E>(value_kind, type_kind) {
            return_type_mismatch_error!(
                location,
                TypeMismatchError::MismatchingType {
                    expected: type_index,
                    actual: value_kind
                }
            )
        }

        TypedTraversalEvent::TerminalValueBatch(TypedLocatedDecoding {
            inner: value_batch_ref,
            parent_relationship,
            type_index,
            location,
        })
    }

    fn map_container_end_event(
        &mut self,
        located_event: LocatedDecoding<ContainerHeaderFor<E>>,
    ) -> TypedTraversalEvent<'de, E::CustomTraversal> {
        let LocatedDecoding {
            inner: header,
            parent_relationship,
            location,
        } = located_event;

        let container = self.container_stack.pop().unwrap();

        TypedTraversalEvent::ContainerEnd(TypedLocatedDecoding {
            inner: header,
            parent_relationship,
            type_index: container.own_type,
            location,
        })
    }

    fn map_decode_error_event(
        located_error: LocatedError<DecodeError>,
    ) -> TypedTraversalEvent<'de, E::CustomTraversal> {
        TypedTraversalEvent::Error(LocatedError {
            location: located_error.location,
            error: TypedTraversalError::DecodeError(located_error.error),
        })
    }

    fn get_type_index(&self, parent_relationship: &ParentRelationship) -> LocalTypeIndex {
        match parent_relationship {
            ParentRelationship::Root => Some(self.root_type_index),
            ParentRelationship::Element { index } => {
                self.container_stack.last().unwrap().child_types.get_child_type_for_element(*index)
            },
            ParentRelationship::ArrayElementBatch { from_index, .. } => {
                self.container_stack.last().unwrap().child_types.get_child_type_for_element(*from_index)
            },
            ParentRelationship::MapKey { .. } => {
                self.container_stack.last().unwrap().child_types.get_child_type_for_map_key()
            },
            ParentRelationship::MapValue { .. } => {
                self.container_stack.last().unwrap().child_types.get_child_type_for_map_value()
            },
        }.expect("Type index should be resolvable given checks on the parent and invariants from the untyped traverser")
    }
}

fn value_kind_matches_type_kind<E: CustomTypeExtension>(
    value_kind: ValueKind<E::CustomValueKind>,
    type_kind: &TypeKindFor<E>,
) -> bool {
    match type_kind {
        TypeKind::Any => true,
        TypeKind::Bool => matches!(value_kind, ValueKind::Bool),
        TypeKind::I8 => matches!(value_kind, ValueKind::I8),
        TypeKind::I16 => matches!(value_kind, ValueKind::I16),
        TypeKind::I32 => matches!(value_kind, ValueKind::I32),
        TypeKind::I64 => matches!(value_kind, ValueKind::I64),
        TypeKind::I128 => matches!(value_kind, ValueKind::I128),
        TypeKind::U8 => matches!(value_kind, ValueKind::U8),
        TypeKind::U16 => matches!(value_kind, ValueKind::U16),
        TypeKind::U32 => matches!(value_kind, ValueKind::U32),
        TypeKind::U64 => matches!(value_kind, ValueKind::U64),
        TypeKind::U128 => matches!(value_kind, ValueKind::U128),
        TypeKind::String => matches!(value_kind, ValueKind::String),
        TypeKind::Array { .. } => matches!(value_kind, ValueKind::Array),
        TypeKind::Tuple { .. } => matches!(value_kind, ValueKind::Tuple),
        TypeKind::Enum { .. } => matches!(value_kind, ValueKind::Enum),
        TypeKind::Map { .. } => matches!(value_kind, ValueKind::Map),
        TypeKind::Custom(custom_type_kind) => {
            E::custom_type_kind_matches_value_kind(custom_type_kind, value_kind)
        }
    }
}
