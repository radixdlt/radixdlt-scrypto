use super::*;
use crate::basic_well_known_types::ANY_TYPE;
use crate::rust::prelude::*;
use crate::traversal::*;
use crate::*;

pub fn traverse_payload_with_types<'de, 's, E: CustomExtension>(
    payload: &'de [u8],
    schema: &'s Schema<E::CustomSchema>,
    type_id: LocalTypeId,
    depth_limit: usize,
) -> TypedTraverser<'de, 's, E> {
    TypedTraverser::new(
        payload,
        schema,
        type_id,
        depth_limit,
        ExpectedStart::PayloadPrefix(E::PAYLOAD_PREFIX),
        true,
    )
}

pub fn traverse_partial_payload_with_types<'de, 's, E: CustomExtension>(
    partial_payload: &'de [u8],
    expected_start: ExpectedStart<E::CustomValueKind>,
    check_exact_end: bool,
    current_depth: usize,
    schema: &'s Schema<E::CustomSchema>,
    type_id: LocalTypeId,
    depth_limit: usize,
) -> TypedTraverser<'de, 's, E> {
    TypedTraverser::new(
        partial_payload,
        schema,
        type_id,
        depth_limit - current_depth,
        expected_start,
        check_exact_end,
    )
}

/// The `TypedTraverser` is for streamed decoding of a payload with type kinds.
///
/// It validates that the payload matches the given type kinds,
/// and adds the relevant type index to the events which are output.
pub struct TypedTraverser<'de, 's, E: CustomExtension> {
    traverser: VecTraverser<'de, E::CustomTraversal>,
    state: TypedTraverserState<'s, E>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerType<'s> {
    Tuple(LocalTypeId, &'s [LocalTypeId]),
    EnumVariant(LocalTypeId, &'s [LocalTypeId]),
    Array(LocalTypeId, LocalTypeId),
    Map(LocalTypeId, LocalTypeId, LocalTypeId),
    Any(LocalTypeId),
}

impl<'s> ContainerType<'s> {
    pub fn self_type(&self) -> LocalTypeId {
        match self {
            ContainerType::Tuple(i, _)
            | ContainerType::EnumVariant(i, _)
            | ContainerType::Array(i, _)
            | ContainerType::Map(i, _, _)
            | ContainerType::Any(i) => *i,
        }
    }

    pub fn get_child_type_for_element(&self, index: usize) -> Option<LocalTypeId> {
        match self {
            Self::Tuple(_, types) => (*types).get(index).copied(),
            Self::EnumVariant(_, types) => (*types).get(index).copied(),
            Self::Array(_, child_type) => Some(*child_type),
            Self::Any(_) => Some(LocalTypeId::WellKnown(ANY_TYPE)),
            _ => None,
        }
    }

    pub fn get_child_type_for_map_key(&self) -> Option<LocalTypeId> {
        match self {
            Self::Map(_, key_type, _) => Some(*key_type),
            Self::Any(_) => Some(LocalTypeId::WellKnown(ANY_TYPE)),
            _ => None,
        }
    }

    pub fn get_child_type_for_map_value(&self) -> Option<LocalTypeId> {
        match self {
            Self::Map(_, _, value_type) => Some(*value_type),
            Self::Any(_) => Some(LocalTypeId::WellKnown(ANY_TYPE)),
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
    ($self: ident, $type_id: expr) => {
        match $self.schema.resolve_type_kind($type_id) {
            Some(resolved_type) => resolved_type,
            None => {
                return TypedTraversalEvent::Error(TypedTraversalError::TypeIdNotFound($type_id))
            }
        }
    };
}

impl<'de, 's, E: CustomExtension> TypedTraverser<'de, 's, E> {
    pub fn new(
        input: &'de [u8],
        schema: &'s Schema<E::CustomSchema>,
        type_id: LocalTypeId,
        max_depth: usize,
        expected_start: ExpectedStart<E::CustomValueKind>,
        check_exact_end: bool,
    ) -> Self {
        Self {
            traverser: VecTraverser::new(
                input,
                expected_start,
                VecTraverserConfig {
                    max_depth,
                    check_exact_end,
                },
            ),
            state: TypedTraverserState {
                container_stack: Vec::with_capacity(max_depth),
                schema,
                root_type_id: type_id,
            },
        }
    }

    pub fn next_event(&mut self) -> TypedLocatedTraversalEvent<'_, 's, 'de, E> {
        let (typed_event, location) =
            Self::next_event_internal(&mut self.traverser, &mut self.state);

        TypedLocatedTraversalEvent {
            location: TypedLocation {
                location,
                typed_container_path: &self.state.container_stack,
            },
            event: typed_event,
        }
    }

    pub fn next_event_with_schema(
        &mut self,
    ) -> (
        TypedLocatedTraversalEvent<'_, 's, 'de, E>,
        &Schema<E::CustomSchema>,
    ) {
        let (typed_event, location) =
            Self::next_event_internal(&mut self.traverser, &mut self.state);

        (
            TypedLocatedTraversalEvent {
                location: TypedLocation {
                    location,
                    typed_container_path: &self.state.container_stack,
                },
                event: typed_event,
            },
            // We also return the schema here because it can't be read later as there are issues with &mut references
            &self.state.schema,
        )
    }

    fn next_event_internal<'t1, 'state>(
        inner_traverser: &'t1 mut VecTraverser<'de, E::CustomTraversal>,
        state: &'state mut TypedTraverserState<'s, E>,
    ) -> (
        TypedTraversalEvent<'de, E>,
        Location<'t1, E::CustomTraversal>,
    ) {
        let LocatedTraversalEvent { location, event } = inner_traverser.next_event();
        let typed_event = match event {
            TraversalEvent::ContainerStart(header) => {
                let type_id = state.get_type_id(&location);
                state.map_container_start_event(type_id, header)
            }
            TraversalEvent::TerminalValue(value) => {
                let type_id = state.get_type_id(&location);
                state.map_terminal_value_event(type_id, value)
            }
            TraversalEvent::TerminalValueBatch(value_batch) => {
                let type_id = state.get_type_id(&location);
                state.map_terminal_value_batch_event(type_id, value_batch)
            }
            TraversalEvent::ContainerEnd(header) => state.map_container_end_event(header),
            TraversalEvent::End => TypedTraversalEvent::End,
            TraversalEvent::DecodeError(decode_error) => {
                TypedTraversalEvent::Error(TypedTraversalError::DecodeError(decode_error))
            }
        };
        (typed_event, location)
    }

    pub fn consume_container_end_event(&mut self) -> Result<(), String> {
        let (typed_event, schema) = self.next_event_with_schema();
        match typed_event.event {
            TypedTraversalEvent::ContainerEnd { .. } => Ok(()),
            _ => Err(typed_event.display_as_unexpected_event("ContainerEnd", schema)),
        }
    }

    pub fn consume_end_event(&mut self) -> Result<(), String> {
        let (typed_event, schema) = self.next_event_with_schema();
        match typed_event.event {
            TypedTraversalEvent::End => Ok(()),
            _ => Err(typed_event.display_as_unexpected_event("End", schema)),
        }
    }

    /// And returns the start/end offset of the value body
    pub fn consume_value_tree(&mut self) -> Result<ValueTreeSummary<E::CustomValueKind>, String> {
        let start_depth = self.state.container_stack.len();
        let (first_event, schema) = self.next_event_with_schema();
        let value_body_start_offset = first_event
            .location
            .location
            .get_start_offset_of_value_body();
        match first_event.event {
            TypedTraversalEvent::ContainerStart(_, _) => {
                // Proceed forward to the loop below
            }
            TypedTraversalEvent::TerminalValue(type_id, value_ref) => {
                return Ok(ValueTreeSummary {
                    type_id,
                    value_kind: value_ref.value_kind(),
                    value_body_start_offset_inclusive: value_body_start_offset,
                    value_body_end_offset_exclusive: first_event.location.location.end_offset,
                });
            }
            _ => {
                return Err(first_event
                    .display_as_unexpected_event("TerminalValue | ContainerStart", schema))
            }
        }
        loop {
            let (next_event, schema) = self.next_event_with_schema();
            if matches!(
                next_event.event,
                TypedTraversalEvent::Error(_) | TypedTraversalEvent::End
            ) {
                return Err(
                    next_event.display_as_unexpected_event("ContainerEnd at correct level", schema)
                );
            }
            let back_at_start_depth = next_event.location.typed_container_path.len() == start_depth;
            if back_at_start_depth {
                match next_event.event {
                    TypedTraversalEvent::ContainerEnd(type_id, header) => {
                        return Ok(ValueTreeSummary {
                            type_id,
                            value_kind: header.get_own_value_kind(),
                            value_body_start_offset_inclusive: value_body_start_offset,
                            value_body_end_offset_exclusive: next_event
                                .location
                                .location
                                .end_offset,
                        });
                    }
                    _ => return Err(next_event.display_as_unexpected_event("ContainerEnd", schema)),
                }
            }
        }
    }
}

pub struct ValueTreeSummary<X: CustomValueKind> {
    pub type_id: LocalTypeId,
    pub value_kind: ValueKind<X>,
    pub value_body_start_offset_inclusive: usize,
    pub value_body_end_offset_exclusive: usize,
}

struct TypedTraverserState<'s, E: CustomExtension> {
    container_stack: Vec<ContainerType<'s>>,
    schema: &'s Schema<E::CustomSchema>,
    root_type_id: LocalTypeId,
}

impl<'s, E: CustomExtension> TypedTraverserState<'s, E> {
    fn map_container_start_event<'t, 'de>(
        &'t mut self,
        type_id: LocalTypeId,
        header: ContainerHeader<E::CustomTraversal>,
    ) -> TypedTraversalEvent<'de, E> {
        let container_type = look_up_type!(self, type_id);

        match header {
            ContainerHeader::Tuple(TupleHeader { length }) => match container_type {
                TypeKind::Any => self.container_stack.push(ContainerType::Any(type_id)),
                TypeKind::Tuple { field_types } if field_types.len() == length => self
                    .container_stack
                    .push(ContainerType::Tuple(type_id, &field_types)),
                TypeKind::Tuple { field_types } => return_type_mismatch_error!(
                    location,
                    TypeMismatchError::MismatchingTupleLength {
                        expected: field_types.len(),
                        actual: length,
                        type_id
                    }
                ),
                _ => return_type_mismatch_error!(
                    location,
                    TypeMismatchError::MismatchingType {
                        type_id,
                        expected_type_kind: container_type.label(),
                        actual_value_kind: ValueKind::Tuple
                    }
                ),
            },
            ContainerHeader::EnumVariant(EnumVariantHeader { variant, length }) => {
                match container_type {
                    TypeKind::Any => self.container_stack.push(ContainerType::Any(type_id)),
                    TypeKind::Enum { variants } => match variants.get(&variant) {
                        Some(variant_child_types) if variant_child_types.len() == length => self
                            .container_stack
                            .push(ContainerType::EnumVariant(type_id, variant_child_types)),
                        Some(variant_child_types) => return_type_mismatch_error!(
                            location,
                            TypeMismatchError::MismatchingEnumVariantLength {
                                expected: variant_child_types.len(),
                                actual: length,
                                type_id,
                                variant
                            }
                        ),
                        None => return_type_mismatch_error!(
                            location,
                            TypeMismatchError::UnknownEnumVariant { type_id, variant }
                        ),
                    },
                    _ => return_type_mismatch_error!(
                        location,
                        TypeMismatchError::MismatchingType {
                            type_id,
                            expected_type_kind: container_type.label(),
                            actual_value_kind: ValueKind::Enum
                        }
                    ),
                }
            }
            ContainerHeader::Array(ArrayHeader {
                element_value_kind, ..
            }) => match container_type {
                TypeKind::Any => self.container_stack.push(ContainerType::Any(type_id)),
                TypeKind::Array {
                    element_type: element_type_id,
                } => {
                    let element_type = look_up_type!(self, *element_type_id);
                    if !value_kind_matches_type_kind::<E>(
                        &self.schema,
                        element_value_kind,
                        element_type,
                    ) {
                        return_type_mismatch_error!(
                            location,
                            TypeMismatchError::MismatchingChildElementType {
                                type_id: *element_type_id,
                                expected_type_kind: element_type.label(),
                                actual_value_kind: element_value_kind
                            }
                        )
                    }
                    self.container_stack
                        .push(ContainerType::Array(type_id, *element_type_id))
                }
                _ => return_type_mismatch_error!(
                    location,
                    TypeMismatchError::MismatchingType {
                        type_id,
                        expected_type_kind: container_type.label(),
                        actual_value_kind: ValueKind::Array
                    }
                ),
            },
            ContainerHeader::Map(MapHeader {
                key_value_kind,
                value_value_kind,
                ..
            }) => match container_type {
                TypeKind::Any => self.container_stack.push(ContainerType::Any(type_id)),
                TypeKind::Map {
                    key_type: key_type_id,
                    value_type: value_type_id,
                } => {
                    let key_type = look_up_type!(self, *key_type_id);
                    if !value_kind_matches_type_kind::<E>(&self.schema, key_value_kind, key_type) {
                        return_type_mismatch_error!(
                            location,
                            TypeMismatchError::MismatchingChildKeyType {
                                type_id: *key_type_id,
                                expected_type_kind: key_type.label(),
                                actual_value_kind: key_value_kind
                            }
                        )
                    }
                    let value_type = look_up_type!(self, *value_type_id);
                    if !value_kind_matches_type_kind::<E>(
                        &self.schema,
                        value_value_kind,
                        value_type,
                    ) {
                        return_type_mismatch_error!(
                            location,
                            TypeMismatchError::MismatchingChildValueType {
                                type_id: *value_type_id,
                                expected_type_kind: value_type.label(),
                                actual_value_kind: value_value_kind
                            }
                        )
                    }
                    self.container_stack.push(ContainerType::Map(
                        type_id,
                        *key_type_id,
                        *value_type_id,
                    ))
                }
                _ => return_type_mismatch_error!(
                    location,
                    TypeMismatchError::MismatchingType {
                        type_id,
                        expected_type_kind: container_type.label(),
                        actual_value_kind: ValueKind::Map
                    }
                ),
            },
        }

        TypedTraversalEvent::ContainerStart(type_id, header)
    }

    fn map_terminal_value_event<'t, 'de>(
        &'t mut self,
        type_id: LocalTypeId,
        value_ref: TerminalValueRef<'de, E::CustomTraversal>,
    ) -> TypedTraversalEvent<'de, E> {
        let value_kind = value_ref.value_kind();
        let type_kind = look_up_type!(self, type_id);

        if !value_kind_matches_type_kind::<E>(&self.schema, value_kind, type_kind) {
            return_type_mismatch_error!(
                location,
                TypeMismatchError::MismatchingType {
                    type_id,
                    expected_type_kind: type_kind.label(),
                    actual_value_kind: value_kind
                }
            )
        }

        TypedTraversalEvent::TerminalValue(type_id, value_ref)
    }

    fn map_terminal_value_batch_event<'t, 'de>(
        &'t mut self,
        type_id: LocalTypeId,
        value_batch_ref: TerminalValueBatchRef<'de>,
    ) -> TypedTraversalEvent<'de, E> {
        let value_kind = value_batch_ref.value_kind();
        let type_kind = look_up_type!(self, type_id);

        if !value_kind_matches_type_kind::<E>(&self.schema, value_kind, type_kind) {
            return_type_mismatch_error!(
                location,
                TypeMismatchError::MismatchingType {
                    type_id,
                    expected_type_kind: type_kind.label(),
                    actual_value_kind: value_kind
                }
            )
        }

        TypedTraversalEvent::TerminalValueBatch(type_id, value_batch_ref)
    }

    fn map_container_end_event<'t, 'de>(
        &'t mut self,
        header: ContainerHeader<E::CustomTraversal>,
    ) -> TypedTraversalEvent<'de, E> {
        let container = self.container_stack.pop().unwrap();

        TypedTraversalEvent::ContainerEnd(container.self_type(), header)
    }

    fn get_type_id(&self, location: &Location<E::CustomTraversal>) -> LocalTypeId {
        match location.get_latest_ancestor() {
            Some(ancestor_state) => {
                let child_index = ancestor_state.current_child_index;
                match ancestor_state.container_header {
                    ContainerHeader::Tuple(_)
                    | ContainerHeader::EnumVariant(_)
                    | ContainerHeader::Array(_) =>  {
                        self.container_stack.last().unwrap().get_child_type_for_element(child_index)
                    }
                    ContainerHeader::Map(_) =>  {
                        if child_index % 2 == 0 {
                            self.container_stack.last().unwrap().get_child_type_for_map_key()
                        } else {
                            self.container_stack.last().unwrap().get_child_type_for_map_value()
                        }
                    }
                }
            }
            None =>  Some(self.root_type_id),
        }.expect("Type index should be resolvable given checks on the parent and invariants from the untyped traverser")
    }
}

fn value_kind_matches_type_kind<E: CustomExtension>(
    schema: &Schema<E::CustomSchema>,
    value_kind: ValueKind<E::CustomValueKind>,
    type_kind: &LocalTypeKind<E::CustomSchema>,
) -> bool {
    if matches!(type_kind, TypeKind::Any) {
        return true;
    }
    match value_kind {
        ValueKind::Custom(custom_value_kind) => {
            return E::custom_value_kind_matches_type_kind(schema, custom_value_kind, type_kind);
        }
        _ => {}
    }
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
            E::custom_type_kind_matches_non_custom_value_kind(schema, custom_type_kind, value_kind)
        }
    }
}
