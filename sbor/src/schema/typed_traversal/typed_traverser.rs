use super::*;
use crate::basic_well_known_types::ANY_ID;
use crate::rust::prelude::*;
use crate::traversal::*;
use crate::*;

pub fn traverse_payload_with_types<'de, 's, E: CustomTypeExtension>(
    payload: &'de [u8],
    schema: &'s Schema<E>,
    index: LocalTypeIndex,
) -> TypedTraverser<'de, 's, E> {
    TypedTraverser::new(
        payload,
        schema,
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
    traverser: VecTraverser<'de, E::CustomTraversal>,
    state: TypedTraverserState<'s, E>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerType<'s> {
    Tuple(LocalTypeIndex, &'s [LocalTypeIndex]),
    EnumVariant(LocalTypeIndex, &'s [LocalTypeIndex]),
    Array(LocalTypeIndex, LocalTypeIndex),
    Map(LocalTypeIndex, LocalTypeIndex, LocalTypeIndex),
    Any(LocalTypeIndex),
}

impl<'s> ContainerType<'s> {
    pub fn self_type(&self) -> LocalTypeIndex {
        match self {
            ContainerType::Tuple(i, _)
            | ContainerType::EnumVariant(i, _)
            | ContainerType::Array(i, _)
            | ContainerType::Map(i, _, _)
            | ContainerType::Any(i) => *i,
        }
    }

    pub fn get_child_type_for_element(&self, index: usize) -> Option<LocalTypeIndex> {
        match self {
            Self::Tuple(_, types) => (*types).get(index).copied(),
            Self::EnumVariant(_, types) => (*types).get(index).copied(),
            Self::Array(_, child_type) => Some(*child_type),
            Self::Any(_) => Some(LocalTypeIndex::WellKnown(ANY_ID)),
            _ => None,
        }
    }

    pub fn get_child_type_for_map_key(&self) -> Option<LocalTypeIndex> {
        match self {
            Self::Map(_, key_type, _) => Some(*key_type),
            Self::Any(_) => Some(LocalTypeIndex::WellKnown(ANY_ID)),
            _ => None,
        }
    }

    pub fn get_child_type_for_map_value(&self) -> Option<LocalTypeIndex> {
        match self {
            Self::Map(_, _, value_type) => Some(*value_type),
            Self::Any(_) => Some(LocalTypeIndex::WellKnown(ANY_ID)),
            _ => None,
        }
    }
}

#[macro_export]
macro_rules! return_type_mismatch_error {
    ($location: ident, $error: expr) => {{
        return TypedTraversalEvent::Error(TypedTraversalError::ValueMismatchWithType($error));
    }};
}

#[macro_export]
macro_rules! look_up_type {
    ($self: ident, $type_index: expr) => {
        match $self.schema.resolve_type_kind($type_index) {
            Some(resolved_type) => resolved_type,
            None => {
                return TypedTraversalEvent::Error(TypedTraversalError::TypeIndexNotFound(
                    $type_index,
                ))
            }
        }
    };
}

impl<'de, 's, E: CustomTypeExtension> TypedTraverser<'de, 's, E> {
    pub fn new(
        input: &'de [u8],
        schema: &'s Schema<E>,
        type_index: LocalTypeIndex,
        max_depth: usize,
        payload_prefix: Option<u8>,
        check_exact_end: bool,
    ) -> Self {
        Self {
            traverser: VecTraverser::new(input, max_depth, payload_prefix, check_exact_end),
            state: TypedTraverserState {
                container_stack: Vec::with_capacity(max_depth),
                schema,
                root_type_index: type_index,
            },
        }
    }

    pub fn next_event<'t>(
        &'t mut self,
    ) -> TypedLocatedTraversalEvent<'t, 's, 'de, E::CustomTraversal> {
        let LocatedTraversalEvent { location, event } = self.traverser.next_event();
        let typed_event = match event {
            TraversalEvent::PayloadPrefix => TypedTraversalEvent::PayloadPrefix,
            TraversalEvent::ContainerStart(header) => {
                let type_index = self.state.get_type_index(&location);
                self.state.map_container_start_event(type_index, header)
            }
            TraversalEvent::TerminalValue(value) => {
                let type_index = self.state.get_type_index(&location);
                self.state.map_terminal_value_event(type_index, value)
            }
            TraversalEvent::TerminalValueBatch(value_batch) => {
                let type_index = self.state.get_type_index(&location);
                self.state
                    .map_terminal_value_batch_event(type_index, value_batch)
            }
            TraversalEvent::ContainerEnd(header) => self.state.map_container_end_event(header),
            TraversalEvent::End => TypedTraversalEvent::End,
            TraversalEvent::DecodeError(decode_error) => {
                TypedTraversalEvent::Error(TypedTraversalError::DecodeError(decode_error))
            }
        };

        TypedLocatedTraversalEvent {
            location: TypedLocation {
                location,
                typed_ancestor_path: &self.state.container_stack,
            },
            event: typed_event,
        }
    }
}

struct TypedTraverserState<'s, E: CustomTypeExtension> {
    container_stack: Vec<ContainerType<'s>>,
    schema: &'s Schema<E>,
    root_type_index: LocalTypeIndex,
}

impl<'s, E: CustomTypeExtension> TypedTraverserState<'s, E> {
    fn map_container_start_event<'t, 'de>(
        &'t mut self,
        type_index: LocalTypeIndex,
        header: ContainerHeader<E::CustomTraversal>,
    ) -> TypedTraversalEvent<'de, E::CustomTraversal> {
        let container_type = look_up_type!(self, type_index);

        match header {
            ContainerHeader::Tuple(TupleHeader { length }) => match container_type {
                TypeKind::Any => self.container_stack.push(ContainerType::Any(type_index)),
                TypeKind::Tuple { field_types } if field_types.len() == length => self
                    .container_stack
                    .push(ContainerType::Tuple(type_index, field_types)),
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
                    TypeKind::Any => self.container_stack.push(ContainerType::Any(type_index)),
                    TypeKind::Enum { variants } => match variants.get(&variant) {
                        Some(variant_child_types) if variant_child_types.len() == length => self
                            .container_stack
                            .push(ContainerType::EnumVariant(type_index, variant_child_types)),
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
                TypeKind::Any => self.container_stack.push(ContainerType::Any(type_index)),
                TypeKind::Array {
                    element_type: element_type_index,
                } => {
                    let element_type = look_up_type!(self, *element_type_index);
                    if !value_kind_matches_type_kind::<E>(element_value_kind, element_type) {
                        return_type_mismatch_error!(
                            location,
                            TypeMismatchError::MismatchingChildElementType {
                                expected: *element_type_index,
                                actual: element_value_kind
                            }
                        )
                    }
                    self.container_stack
                        .push(ContainerType::Array(type_index, *element_type_index))
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
                TypeKind::Any => self.container_stack.push(ContainerType::Any(type_index)),
                TypeKind::Map {
                    key_type: key_type_index,
                    value_type: value_type_index,
                } => {
                    let key_type = look_up_type!(self, *key_type_index);
                    if !value_kind_matches_type_kind::<E>(key_value_kind, key_type) {
                        return_type_mismatch_error!(
                            location,
                            TypeMismatchError::MismatchingChildKeyType {
                                expected: *key_type_index,
                                actual: key_value_kind
                            }
                        )
                    }
                    let value_type = look_up_type!(self, *value_type_index);
                    if !value_kind_matches_type_kind::<E>(value_value_kind, value_type) {
                        return_type_mismatch_error!(
                            location,
                            TypeMismatchError::MismatchingChildValueType {
                                expected: *value_type_index,
                                actual: key_value_kind
                            }
                        )
                    }
                    self.container_stack.push(ContainerType::Map(
                        type_index,
                        *key_type_index,
                        *value_type_index,
                    ))
                }
                _ => return_type_mismatch_error!(
                    location,
                    TypeMismatchError::MismatchingType {
                        expected: type_index,
                        actual: ValueKind::Map
                    }
                ),
            },
        }

        TypedTraversalEvent::ContainerStart(type_index, header)
    }

    fn map_terminal_value_event<'t, 'de>(
        &'t mut self,
        type_index: LocalTypeIndex,
        value_ref: TerminalValueRef<'de, E::CustomTraversal>,
    ) -> TypedTraversalEvent<'de, E::CustomTraversal> {
        let value_kind = value_ref.value_kind();
        let type_kind = look_up_type!(self, type_index);

        if !value_kind_matches_type_kind::<E>(value_kind, type_kind) {
            return_type_mismatch_error!(
                location,
                TypeMismatchError::MismatchingType {
                    expected: type_index,
                    actual: value_kind
                }
            )
        }

        TypedTraversalEvent::TerminalValue(type_index, value_ref)
    }

    fn map_terminal_value_batch_event<'t, 'de>(
        &'t mut self,
        type_index: LocalTypeIndex,
        value_batch_ref: TerminalValueBatchRef<'de>,
    ) -> TypedTraversalEvent<'de, E::CustomTraversal> {
        let value_kind = value_batch_ref.value_kind();
        let type_kind = look_up_type!(self, type_index);

        if !value_kind_matches_type_kind::<E>(value_kind, type_kind) {
            return_type_mismatch_error!(
                location,
                TypeMismatchError::MismatchingType {
                    expected: type_index,
                    actual: value_kind
                }
            )
        }

        TypedTraversalEvent::TerminalValueBatch(type_index, value_batch_ref)
    }

    fn map_container_end_event<'t, 'de>(
        &'t mut self,
        header: ContainerHeader<E::CustomTraversal>,
    ) -> TypedTraversalEvent<'de, E::CustomTraversal> {
        let container = self.container_stack.pop().unwrap();

        TypedTraversalEvent::ContainerEnd(container.self_type(), header)
    }

    fn get_type_index(&self, location: &Location<E::CustomTraversal>) -> LocalTypeIndex {
        match location.ancestor_path.last() {
            Some(container_child) => {
                let current_child_index = container_child.next_child_index - 1;
                match container_child.container_header {
                    ContainerHeader::Tuple(_)
                    | ContainerHeader::EnumVariant(_)
                    | ContainerHeader::Array(_) =>  {
                        self.container_stack.last().unwrap().get_child_type_for_element(current_child_index)
                    }
                    ContainerHeader::Map(_) =>  {
                        if current_child_index % 2 == 0 {
                            self.container_stack.last().unwrap().get_child_type_for_map_key()
                        } else {
                            self.container_stack.last().unwrap().get_child_type_for_map_value()
                        }
                    }
                }
            }
            None =>  Some(self.root_type_index),
        }.expect("Type index should be resolvable given checks on the parent and invariants from the untyped traverser")
    }
}

type TypeKindFor<E> = TypeKind<
    <E as CustomTypeExtension>::CustomValueKind,
    <E as CustomTypeExtension>::CustomTypeKind<LocalTypeIndex>,
    LocalTypeIndex,
>;

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
