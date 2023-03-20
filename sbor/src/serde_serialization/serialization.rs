use super::*;
use crate::rust::cell::RefCell;
use crate::rust::prelude::*;
use crate::traversal::*;
use crate::typed_traversal::*;
use crate::*;
use serde::ser::*;
use utils::*;
use TypedTraversalEvent::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SerializationMode {
    /// This "Invertible" encoding is intended to exactly capture the content of the scrypto value,
    /// in a way which can be inverted back into a scrypto value.
    ///
    /// SBOR values are generally wrapped in an object with a "kind" field. Fields are output as an
    /// array to faithfully represent the ordering in the SBOR value.
    ///
    /// If value/type data is included in the parent (Vecs and Map entries), it is not duplicated
    /// on the values. This avoids duplication in the output. In these cases, child tuples and
    /// single values lose their wrapper object, to keep the output concise. Other values keep
    /// their wrapper object, as there are other fields to convey.
    ///
    /// If a schema is available, variant names, type names and field names are added to the output.
    ///
    /// Some examples:
    /// ```jsonc
    /// // Array
    /// {
    ///     "kind": "Array",
    ///     "element_kind": "U16",
    ///     "elements": [1, 2, 3]
    /// }
    /// // Byte Array
    /// {
    ///     "kind": "Array",
    ///     "element_kind": "U8",
    ///     "hex": "deadbeef"
    /// }
    /// // Map
    /// {
    ///     "kind": "Map",
    ///     "key_kind": "Enum",
    ///     "key_name": "TestEnum",
    ///     "value_kind": "Tuple",
    ///     "value_name": "MyFieldStruct",
    ///     "entries": [
    ///         // Each entry is a [key, value] tuple
    ///         [
    ///             // Enums always have a wrapper object, but their key_kind and key_name
    ///             // are not repeated from the parent
    ///             { "variant_id": 3, "variant_name": "Test", "fields": [] },
    ///             // The tuple loses its wrapper object
    ///             [{ "kind": "String", "value": "one" }, { "kind": "U8", "value": 2 }]
    ///         ]
    ///     ]
    /// }
    /// // Struct / Named tuple with named fields
    /// {
    ///     "kind": "Tuple",
    ///     "name": "MyNamedStruct",
    ///     "fields": [
    ///          { "key": "a", "kind": "U8", "value": 1 },
    ///          { "key": "b", "kind": "U8", "value": 2 }
    ///     ]
    /// }
    /// // Enum Variant
    /// {
    ///     "kind": "Enum",
    ///     "name": "Employee",
    ///     "variant_id": 1,
    ///     "variant_name": "Bob",
    ///     "fields": [
    ///          { "key": "number", "kind": "U32", "value": 1 }
    ///     ]
    /// }
    /// // Single values
    /// {
    ///     "kind": "String",
    ///     "value": "Hello world!"
    /// }
    /// {
    ///     "kind": "U64",
    ///     "value": "1234123124" // U64/I64 and larger are encoded as strings for JS compatibility
    /// }
    /// ```
    ///
    Invertible,
    /// This "Simple" encoding is intended to be "nice to read" for a human.
    ///
    /// It can be used for values with a schema, or without a schema (equivalently, values with "Any"
    /// schema).
    ///
    /// It is not intended to be invertible - ie the output cannot be mapped back into a ScryptoValue
    /// It should favour simplicity for human comprehension, in particular:
    /// * It uses a JSON object rather than an array where possible, even if this loses field ordering
    ///   EG for structs, and for maps with string keys.
    /// * If the concept which is being represented (eg number/amount or address) is clear
    ///   to a human, information about the value kind is dropped.
    ///
    /// Compared with Invertible, it is more compact, but doesn't include type names.
    Simple,
}

pub struct SborPayloadWithSchema<'de, E: SerializableCustomTypeExtension> {
    payload: &'de [u8],
    type_index: LocalTypeIndex,
    type_extension: PhantomData<E>,
}

impl<'de, E: SerializableCustomTypeExtension> SborPayloadWithSchema<'de, E> {
    pub fn new(payload: &'de [u8], type_index: LocalTypeIndex) -> Self {
        Self {
            payload,
            type_index,
            type_extension: PhantomData,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SerializationContext<'s, 'a, E: SerializableCustomTypeExtension> {
    pub schema: &'s Schema<E>,
    pub mode: SerializationMode,
    pub custom_context: E::CustomSerializationContext<'a>,
}

impl<'s, 'a, 'de, E: SerializableCustomTypeExtension>
    ContextualSerialize<SerializationContext<'s, 'a, E>> for SborPayloadWithSchema<'de, E>
{
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &SerializationContext<'s, 'a, E>,
    ) -> Result<S::Ok, S::Error> {
        simple_serialize(serializer, self.payload, context, self.type_index)
    }
}

pub struct SborPayloadWithoutSchema<'de, E: SerializableCustomTypeExtension> {
    payload: &'de [u8],
    type_extension: PhantomData<E>,
}

impl<'de, E: SerializableCustomTypeExtension> SborPayloadWithoutSchema<'de, E> {
    pub fn new(payload: &'de [u8]) -> Self {
        Self {
            payload,
            type_extension: PhantomData,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SchemalessSerializationContext<'a, E: SerializableCustomTypeExtension> {
    pub mode: SerializationMode,
    pub custom_context: E::CustomSerializationContext<'a>,
}

impl<'s, 'a, 'de, E: SerializableCustomTypeExtension>
    ContextualSerialize<SchemalessSerializationContext<'a, E>>
    for SborPayloadWithoutSchema<'de, E>
{
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &SchemalessSerializationContext<E>,
    ) -> Result<S::Ok, S::Error> {
        SborPayloadWithSchema::<E>::new(&self.payload, LocalTypeIndex::any()).serialize(
            serializer,
            SerializationContext {
                schema: &Schema::<E>::empty(),
                mode: context.mode,
                custom_context: context.custom_context,
            },
        )
    }
}

pub fn simple_serialize<S: Serializer, E: SerializableCustomTypeExtension>(
    serializer: S,
    payload: &[u8],
    context: &SerializationContext<'_, '_, E>,
    index: LocalTypeIndex,
) -> Result<S::Ok, S::Error> {
    let mut traverser = traverse_payload_with_types(payload, context.schema, index);
    consume_payload_start_events::<S, E>(&mut traverser, context)?;
    let success =
        serialize_value_tree::<S, E>(serializer, &mut traverser, context, &ValueContext::Default)?;
    consume_payload_end_events::<S, E>(&mut traverser, context)?;
    Ok(success)
}

pub fn consume_payload_start_events<S: Serializer, E: SerializableCustomTypeExtension>(
    traverser: &mut TypedTraverser<E>,
    context: &SerializationContext<'_, '_, E>,
) -> Result<(), S::Error> {
    let typed_event = traverser.next_event();
    match typed_event.event {
        PayloadPrefix => Ok(()),
        Error(error) => Err(map_error::<S, E>(
            context,
            &typed_event,
            SerializationError::TraversalError(error),
        )),
        _ => panic!(
            "Expected first event to be PayloadPrefix or Error, got {:?}",
            typed_event.event
        ),
    }
}

pub fn consume_payload_end_events<S: Serializer, E: SerializableCustomTypeExtension>(
    traverser: &mut TypedTraverser<E>,
    context: &SerializationContext<'_, '_, E>,
) -> Result<(), S::Error> {
    let typed_event = traverser.next_event();
    match typed_event.event {
        End => Ok(()),
        Error(error) => Err(map_error::<S, E>(
            context,
            &typed_event,
            SerializationError::TraversalError(error),
        )),
        _ => panic!(
            "Expected end event to be End or Error, got {:?}",
            typed_event.event
        ),
    }
}

pub fn expect_container_end<S: Serializer, E: SerializableCustomTypeExtension>(
    traverser: &mut TypedTraverser<E>,
    context: &SerializationContext<'_, '_, E>,
) -> Result<(), S::Error> {
    let typed_event = traverser.next_event();
    match typed_event.event {
        ContainerEnd(_, _) => Ok(()),
        Error(error) => Err(map_error::<S, E>(
            context,
            &typed_event,
            SerializationError::TraversalError(error),
        )),
        _ => panic!("Expected container end event, got {:?}", typed_event.event),
    }
}

fn map_error<S: Serializer, E: SerializableCustomTypeExtension>(
    context: &SerializationContext<'_, '_, E>,
    typed_event: &TypedLocatedTraversalEvent<E::CustomTraversal>,
    error: SerializationError<E>,
) -> S::Error {
    let full_location = typed_event.full_location();
    S::Error::custom(format!(
        "{:?} occurred at byte offset {}-{} and value path {}",
        error,
        full_location.start_offset,
        full_location.end_offset,
        full_location.path_to_string(&context.schema)
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerializationError<E: CustomTypeExtension> {
    TraversalError(TypedTraversalError<E::CustomValueKind>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueContext {
    /// So it doesn't need to include its own kind details
    VecOrMapChild,
    /// The default context - should include its own kind details
    Default,
    /// A named field wrapper - should include its own kind details, and a key field
    IncludeFieldKey { key: String },
}

struct SerializableValueTree<'t, 'de, 's1, E: CustomTypeExtension> {
    traverser: RefCell<&'t mut TypedTraverser<'de, 's1, E>>,
    value_context: ValueContext,
}

impl<'t, 'de, 's1, E: SerializableCustomTypeExtension> SerializableValueTree<'t, 'de, 's1, E> {
    fn new(traverser: &'t mut TypedTraverser<'de, 's1, E>, value_context: ValueContext) -> Self {
        Self {
            traverser: RefCell::new(traverser),
            value_context,
        }
    }
}

impl<'t, 'de, 's1, 's, 'a, E: SerializableCustomTypeExtension>
    ContextualSerialize<SerializationContext<'s, 'a, E>>
    for SerializableValueTree<'t, 'de, 's1, E>
{
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &SerializationContext<'s, 'a, E>,
    ) -> Result<S::Ok, S::Error> {
        serialize_value_tree(
            serializer,
            &mut self.traverser.borrow_mut(),
            context,
            &self.value_context,
        )
    }
}

fn serialize_value_tree<S: Serializer, E: SerializableCustomTypeExtension>(
    serializer: S,
    traverser: &mut TypedTraverser<E>,
    context: &SerializationContext<'_, '_, E>,
    value_context: &ValueContext,
) -> Result<S::Ok, S::Error> {
    let typed_event = traverser.next_event();
    match typed_event.event {
        ContainerStart(type_index, container_header) => match container_header {
            ContainerHeader::Tuple(header) => serialize_tuple(
                serializer,
                traverser,
                context,
                type_index,
                header,
                value_context,
            ),
            ContainerHeader::EnumVariant(header) => serialize_enum_variant(
                serializer,
                traverser,
                context,
                type_index,
                header,
                value_context,
            ),
            ContainerHeader::Array(header) => serialize_array(
                serializer,
                traverser,
                context,
                type_index,
                header,
                value_context,
            ),
            ContainerHeader::Map(header) => serialize_map(
                serializer,
                traverser,
                context,
                type_index,
                header,
                value_context,
            ),
        },
        TerminalValue(type_index, value_ref) => {
            serialize_terminal_value(serializer, context, type_index, value_ref, value_context)
        }
        Error(error) => Err(map_error::<S, E>(
            context,
            &typed_event,
            SerializationError::TraversalError(error),
        )),
        PayloadPrefix | End | ContainerEnd(_, _) | TerminalValueBatch(_, _) => {
            panic!("Unexpected event {:?}", typed_event.event)
        }
    }
}

/// Consumes the number of value-trees from the traverser, either:
/// * If there are child field names - into a serde map, with keys by child name
/// * If there are no child field names - into a serde tuple
///
/// Note that it doesn't consume the container end event, because it's also
/// used for (eg) map entry pairs, which don't have a container end event
pub struct SerializableFields<'t, 'de, 's1, 's2, E: CustomTypeExtension> {
    traverser: RefCell<&'t mut TypedTraverser<'de, 's1, E>>,
    fields_type: FieldsType<'s2>,
    length: usize,
}

pub enum FieldsType<'s2> {
    NamedFields(&'s2 [Cow<'static, str>]),
    UnnamedFields,
    MapEntry,
}

impl<'s2> From<Option<&'s2 ChildNames>> for FieldsType<'s2> {
    fn from(child_names: Option<&'s2 ChildNames>) -> Self {
        match child_names {
            Some(ChildNames::NamedFields(names)) => Self::NamedFields(names),
            _ => Self::UnnamedFields,
        }
    }
}

impl<'t, 'de, 's1, 's2, E: CustomTypeExtension> SerializableFields<'t, 'de, 's1, 's2, E> {
    fn new(
        traverser: &'t mut TypedTraverser<'de, 's1, E>,
        fields_type: FieldsType<'s2>,
        length: usize,
    ) -> Self {
        Self {
            traverser: RefCell::new(traverser),
            fields_type,
            length,
        }
    }
}

impl<'t, 'de, 's1, 's2, 's, 'a, E: SerializableCustomTypeExtension>
    ContextualSerialize<SerializationContext<'s, 'a, E>>
    for SerializableFields<'t, 'de, 's1, 's2, E>
{
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &SerializationContext<'s, 'a, E>,
    ) -> Result<S::Ok, S::Error> {
        serialize_fields_to_value(
            serializer,
            &mut self.traverser.borrow_mut(),
            context,
            &self.fields_type,
            self.length,
        )
    }
}

fn serialize_fields_to_value<S: Serializer, E: SerializableCustomTypeExtension>(
    serializer: S,
    traverser: &mut TypedTraverser<E>,
    context: &SerializationContext<'_, '_, E>,
    fields_type: &FieldsType<'_>,
    length: usize,
) -> Result<S::Ok, S::Error> {
    match (context.mode, fields_type) {
        // In simple mode, we serialize structs as JSON objects
        (SerializationMode::Simple, FieldsType::NamedFields(field_names)) => {
            let mut serde_map = serializer.serialize_map(Some(length))?;
            for field_name in field_names.iter() {
                serde_map.serialize_entry(
                    field_name,
                    &SerializableValueTree::new(traverser, ValueContext::Default)
                        .serializable(*context),
                )?;
            }
            serde_map.end()
        }
        // In invertible mode, we serialize structs as a JSON array of field objects to preserve ordering
        (SerializationMode::Invertible, FieldsType::NamedFields(field_names)) => {
            let mut serde_tuple = serializer.serialize_tuple(length)?;
            for field_name in field_names.iter() {
                serde_tuple.serialize_element(
                    &SerializableValueTree::new(
                        traverser,
                        ValueContext::IncludeFieldKey {
                            key: field_name.to_string(),
                        },
                    )
                    .serializable(*context),
                )?;
            }
            serde_tuple.end()
        }
        // If we're encoding a map entry tuple, we include ValueContext::VecOrMapChild so the values
        // aren't serialized with their type information
        (_, FieldsType::MapEntry) => {
            let mut serde_tuple = serializer.serialize_tuple(length)?;
            for _ in 0..length {
                serde_tuple.serialize_element(
                    &SerializableValueTree::new(traverser, ValueContext::VecOrMapChild)
                        .serializable(*context),
                )?;
            }
            serde_tuple.end()
        }
        // Otherwise, we're encoding an unnamed tuple, so we just serialize the values normally in a JSON array
        (_, _) => {
            let mut serde_tuple = serializer.serialize_tuple(length)?;
            for _ in 0..length {
                serde_tuple.serialize_element(
                    &SerializableValueTree::new(traverser, ValueContext::Default)
                        .serializable(*context),
                )?;
            }
            serde_tuple.end()
        }
    }
}

pub struct SerializableArrayElements<'t, 'de, 's1, E: CustomTypeExtension> {
    traverser: RefCell<&'t mut TypedTraverser<'de, 's1, E>>,
    length: usize,
}

impl<'t, 'de, 's1, E: CustomTypeExtension> SerializableArrayElements<'t, 'de, 's1, E> {
    fn new(traverser: &'t mut TypedTraverser<'de, 's1, E>, length: usize) -> Self {
        Self {
            traverser: RefCell::new(traverser),
            length,
        }
    }
}

impl<'t, 'de, 's1, 's, 'a, E: SerializableCustomTypeExtension>
    ContextualSerialize<SerializationContext<'s, 'a, E>>
    for SerializableArrayElements<'t, 'de, 's1, E>
{
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &SerializationContext<'s, 'a, E>,
    ) -> Result<S::Ok, S::Error> {
        let mut serde_tuple = serializer.serialize_tuple(self.length)?;
        let traverser = &mut self.traverser.borrow_mut();
        for _ in 0..self.length {
            serde_tuple.serialize_element(
                &SerializableValueTree::new(traverser, ValueContext::VecOrMapChild)
                    .serializable(*context),
            )?;
        }
        expect_container_end::<S, E>(traverser, context)?;
        serde_tuple.end()
    }
}

pub struct SerializableMapElements<'t, 'de, 's1, E: CustomTypeExtension> {
    traverser: RefCell<&'t mut TypedTraverser<'de, 's1, E>>,
    key_value_kind: ValueKind<E::CustomValueKind>,
    length: usize,
}

impl<'t, 'de, 's1, E: CustomTypeExtension> SerializableMapElements<'t, 'de, 's1, E> {
    fn new(
        traverser: &'t mut TypedTraverser<'de, 's1, E>,
        key_value_kind: ValueKind<E::CustomValueKind>,
        length: usize,
    ) -> Self {
        Self {
            traverser: RefCell::new(traverser),
            key_value_kind,
            length,
        }
    }
}

impl<'t, 'de, 's1, 's, 'a, E: SerializableCustomTypeExtension>
    ContextualSerialize<SerializationContext<'s, 'a, E>>
    for SerializableMapElements<'t, 'de, 's1, E>
{
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &SerializationContext<'s, 'a, E>,
    ) -> Result<S::Ok, S::Error> {
        let traverser = &mut self.traverser.borrow_mut();
        match (context.mode, self.key_value_kind) {
            (SerializationMode::Simple, ValueKind::String) => {
                let mut serde_map = serializer.serialize_map(Some(self.length))?;
                for _ in 0..self.length {
                    serde_map.serialize_key(
                        &SerializableValueTree::new(traverser, ValueContext::VecOrMapChild)
                            .serializable(*context),
                    )?;
                    serde_map.serialize_value(
                        &SerializableValueTree::new(traverser, ValueContext::VecOrMapChild)
                            .serializable(*context),
                    )?;
                }
                expect_container_end::<S, E>(traverser, context)?;
                serde_map.end()
            }
            _ => {
                let mut serde_tuple = serializer.serialize_tuple(self.length)?;
                for _ in 0..self.length {
                    serde_tuple.serialize_element(
                        &SerializableFields::new(traverser, FieldsType::MapEntry, 2)
                            .serializable(*context),
                    )?;
                }
                expect_container_end::<S, E>(traverser, context)?;
                serde_tuple.end()
            }
        }
    }
}

fn serialize_tuple<S: Serializer, E: SerializableCustomTypeExtension>(
    serializer: S,
    traverser: &mut TypedTraverser<E>,
    context: &SerializationContext<'_, '_, E>,
    type_index: LocalTypeIndex,
    tuple_header: TupleHeader,
    value_context: &ValueContext,
) -> Result<S::Ok, S::Error> {
    let metadata = context.schema.resolve_type_metadata(type_index);
    let child_names = metadata.and_then(|m| m.child_names.as_ref());
    let mut map_aggregator = SerdeValueMapAggregator::new(context, value_context);

    if !map_aggregator.should_embed_value_in_contextual_json_map() {
        let result_ok = SerializableFields::new(traverser, child_names.into(), tuple_header.length)
            .serialize(serializer, *context)?;
        expect_container_end::<S, E>(traverser, context)?;
        return Ok(result_ok);
    }
    map_aggregator.add_initial_details(ValueKind::Tuple, metadata);
    map_aggregator.add_field(
        "fields",
        SerializableType::SerializableFields(SerializableFields::new(
            traverser,
            child_names.into(),
            tuple_header.length,
        )),
    );
    let success = map_aggregator.into_map(serializer)?;
    expect_container_end::<S, E>(traverser, context)?;
    Ok(success)
}

fn serialize_enum_variant<S: Serializer, E: SerializableCustomTypeExtension>(
    serializer: S,
    traverser: &mut TypedTraverser<E>,
    context: &SerializationContext<'_, '_, E>,
    type_index: LocalTypeIndex,
    variant_header: EnumVariantHeader,
    value_context: &ValueContext,
) -> Result<S::Ok, S::Error> {
    let enum_metadata = context.schema.resolve_type_metadata(type_index);
    let mut map_aggregator = SerdeValueMapAggregator::new(context, value_context);

    map_aggregator.add_initial_details(ValueKind::Enum, enum_metadata);
    let child_names =
        map_aggregator.add_enum_variant_details(variant_header.variant, enum_metadata);
    map_aggregator.add_field(
        "fields",
        SerializableType::SerializableFields(SerializableFields::new(
            traverser,
            child_names.into(),
            variant_header.length,
        )),
    );
    let success = map_aggregator.into_map(serializer)?;
    expect_container_end::<S, E>(traverser, context)?;
    Ok(success)
}

fn serialize_array<S: Serializer, E: SerializableCustomTypeExtension>(
    serializer: S,
    traverser: &mut TypedTraverser<E>,
    context: &SerializationContext<'_, '_, E>,
    type_index: LocalTypeIndex,
    array_header: ArrayHeader<E::CustomValueKind>,
    value_context: &ValueContext,
) -> Result<S::Ok, S::Error> {
    let mut map_aggregator = SerdeValueMapAggregator::new(context, value_context);
    if !map_aggregator.should_embed_value_in_contextual_json_map()
        && array_header.element_value_kind != ValueKind::U8
    {
        // We don't need the wrapper object
        return SerializableArrayElements::new(traverser, array_header.length)
            .serialize(serializer, *context);
    }
    let metadata = context.schema.resolve_type_metadata(type_index);
    map_aggregator.add_initial_details(ValueKind::Array, metadata);
    map_aggregator.add_element_details(array_header.element_value_kind, type_index);

    match (array_header.element_value_kind, array_header.length) {
        (ValueKind::U8, 0) => {
            map_aggregator.add_field("hex", SerializableType::Str(""));
            expect_container_end::<S, E>(traverser, context)?;
        }
        (ValueKind::U8, _) => {
            let typed_event = traverser.next_event();
            match typed_event.event {
                TerminalValueBatch(_, TerminalValueBatchRef::U8(bytes)) => {
                    map_aggregator.add_field("hex", SerializableType::String(hex::encode(bytes)));
                }
                Error(error) => Err(map_error::<S, E>(
                    context,
                    &typed_event,
                    SerializationError::TraversalError(error),
                ))?,
                _ => panic!("Unexpected event {:?}", typed_event.event),
            };
            expect_container_end::<S, E>(traverser, context)?;
        }
        (_, _) => {
            map_aggregator.add_field(
                "elements",
                SerializableType::SerializableArrayElements(SerializableArrayElements::new(
                    traverser,
                    array_header.length,
                )),
            );
        }
    }
    map_aggregator.into_map(serializer)
}

fn serialize_map<S: Serializer, E: SerializableCustomTypeExtension>(
    serializer: S,
    traverser: &mut TypedTraverser<E>,
    context: &SerializationContext<'_, '_, E>,
    type_index: LocalTypeIndex,
    map_header: MapHeader<E::CustomValueKind>,
    value_context: &ValueContext,
) -> Result<S::Ok, S::Error> {
    let mut map_aggregator = SerdeValueMapAggregator::new(context, value_context);
    if !map_aggregator.should_embed_value_in_contextual_json_map() {
        // We don't need the wrapper object
        return SerializableMapElements::new(
            traverser,
            map_header.key_value_kind,
            map_header.length,
        )
        .serialize(serializer, *context);
    }
    let metadata = context.schema.resolve_type_metadata(type_index);
    map_aggregator.add_initial_details(ValueKind::Map, metadata);
    map_aggregator.add_map_child_details(
        map_header.key_value_kind,
        map_header.value_value_kind,
        type_index,
    );
    map_aggregator.add_field(
        "entries",
        SerializableType::SerializableMapElements(SerializableMapElements::new(
            traverser,
            map_header.key_value_kind,
            map_header.length,
        )),
    );
    map_aggregator.into_map(serializer)
}

fn serialize_terminal_value<S: Serializer, E: SerializableCustomTypeExtension>(
    serializer: S,
    context: &SerializationContext<'_, '_, E>,
    type_index: LocalTypeIndex,
    value_ref: TerminalValueRef<E::CustomTraversal>,
    value_context: &ValueContext,
) -> Result<S::Ok, S::Error> {
    let value_kind = value_ref.value_kind();
    let (serializable_value, include_type_tag_in_simple_mode) = match value_ref {
        // Javascript only safely decodes JSON integers up to 2^53
        // So to be safe, we encode I64s as strings
        // Moreover, I128 isn't supported by the JSON serializer
        TerminalValueRef::Bool(value) => (SerializableType::Bool(value), false),
        TerminalValueRef::I8(value) => (SerializableType::I8(value), false),
        TerminalValueRef::I16(value) => (SerializableType::I16(value), false),
        TerminalValueRef::I32(value) => (SerializableType::I32(value), false),
        TerminalValueRef::I64(value) => (SerializableType::String(value.to_string()), false),
        TerminalValueRef::I128(value) => (SerializableType::String(value.to_string()), false),
        TerminalValueRef::U8(value) => (SerializableType::U8(value), false),
        TerminalValueRef::U16(value) => (SerializableType::U16(value), false),
        TerminalValueRef::U32(value) => (SerializableType::U32(value), false),
        TerminalValueRef::U64(value) => (SerializableType::String(value.to_string()), false),
        TerminalValueRef::U128(value) => (SerializableType::String(value.to_string()), false),
        TerminalValueRef::String(value) => (SerializableType::Str(value), false),
        TerminalValueRef::Custom(custom_value) => {
            let CustomTypeSerialization {
                include_type_tag_in_simple_mode,
                serialization,
            } = E::serialize_value(context, type_index, custom_value);
            (serialization, include_type_tag_in_simple_mode)
        }
    };
    let mut map_aggregator = if include_type_tag_in_simple_mode {
        SerdeValueMapAggregator::new_with_kind_tag(context, value_context)
    } else {
        SerdeValueMapAggregator::new(context, value_context)
    };
    if map_aggregator.should_embed_value_in_contextual_json_map() {
        let metadata = context.schema.resolve_type_metadata(type_index);
        map_aggregator.add_initial_details(value_kind, metadata);
        map_aggregator.add_field("value", serializable_value);
        map_aggregator.into_map(serializer)
    } else {
        serializable_value.serialize(serializer, *context)
    }
}

#[cfg(test)]
#[cfg(feature = "serde")] // Ensures that VS Code runs this module with the features serde tag!
mod tests {
    use super::*;
    use serde_json::{json, to_string, to_value, Value as JsonValue};

    pub fn assert_json_eq<T: Serialize>(actual: T, expected: JsonValue) {
        let actual = to_value(&actual).unwrap();
        if actual != expected {
            panic!(
                "Mismatching JSON:\nActual:\n{}\nExpected:\n{}\n",
                to_string(&actual).unwrap(),
                to_string(&expected).unwrap()
            );
        }
    }

    #[test]
    #[cfg(feature = "serde")] // Workaround for VS Code "Run Test" feature
    fn test_schemaless_value_encoding() {
        let value = BasicValue::Tuple {
            fields: vec![
                Value::Bool { value: true },
                Value::U8 { value: 5 },
                Value::U16 { value: 5 },
                Value::U32 { value: 5 },
                Value::U64 { value: u64::MAX },
                Value::U128 {
                    value: 9912313323213,
                },
                Value::I8 { value: -5 },
                Value::I16 { value: -5 },
                Value::I32 { value: -5 },
                Value::I64 { value: -5 },
                Value::I128 { value: -5 },
                Value::Array {
                    element_value_kind: ValueKind::U8,
                    elements: vec![Value::U8 { value: 0x3a }, Value::U8 { value: 0x92 }],
                },
                Value::Array {
                    element_value_kind: ValueKind::U8,
                    elements: vec![],
                },
                Value::Array {
                    element_value_kind: ValueKind::U32,
                    elements: vec![Value::U32 { value: 153 }, Value::U32 { value: 62 }],
                },
                Value::Enum {
                    discriminator: 0,
                    fields: vec![],
                },
                Value::Enum {
                    discriminator: 1,
                    fields: vec![Value::U32 { value: 153 }],
                },
                Value::Enum {
                    discriminator: 2,
                    fields: vec![Value::U32 { value: 153 }, Value::Bool { value: true }],
                },
                Value::Map {
                    key_value_kind: ValueKind::U32,
                    value_value_kind: ValueKind::U32,
                    entries: vec![(Value::U32 { value: 153 }, Value::U32 { value: 62 })],
                },
                Value::Tuple {
                    fields: vec![
                        Value::String {
                            value: "hello".to_string(),
                        },
                        Value::U32 { value: 1234 },
                    ],
                },
            ],
        };

        let expected_simple = json!([
            true,
            5,
            5,
            5,
            "18446744073709551615",
            "9912313323213",
            -5,
            -5,
            -5,
            "-5",
            "-5",
            {
                "hex": "3a92"
            },
            {
                "hex": ""
            },
            [
                153,
                62
            ],
            {
                "variant_id": 0,
                "fields": [],
            },
            {
                "variant_id": 1,
                "fields": [
                    153
                ],
            },
            {
                "variant_id": 2,
                "fields": [
                    153,
                    true
                ],
            },
            [
                [
                    153,
                    62
                ]
            ],
            [
                "hello",
                1234
            ]
        ]);

        let payload = basic_encode(&value).unwrap();
        assert_json_eq(
            SborPayloadWithoutSchema::<NoCustomTypeExtension>::new(&payload).serializable(
                SchemalessSerializationContext {
                    mode: SerializationMode::Simple,
                    custom_context: (),
                },
            ),
            expected_simple,
        );
    }

    #[derive(Sbor, Hash, Eq, PartialEq)]
    enum TestEnum {
        UnitVariant,
        SingleFieldVariant { field: u8 },
        DoubleStructVariant { field1: u8, field2: u8 },
    }

    #[derive(Sbor)]
    struct MyUnitStruct;

    #[derive(Sbor)]
    #[sbor(custom_value_kind = "NoCustomValueKind")]
    struct MyComplexTupleStruct(
        Vec<u16>,
        Vec<u16>,
        Vec<u8>,
        Vec<u8>,
        IndexMap<TestEnum, MyFieldStruct>,
        BTreeMap<String, MyUnitStruct>,
        TestEnum,
        TestEnum,
        TestEnum,
        MyFieldStruct,
        Vec<MyUnitStruct>,
        BasicValue,
    );

    #[derive(Sbor)]
    struct MyFieldStruct {
        field1: u64,
        field2: Vec<String>,
    }

    #[test]
    #[cfg(feature = "serde")] // Workaround for VS Code "Run Test" feature
    fn complex_value_encoding() {
        let (type_index, schema) =
            generate_full_schema_from_single_type::<MyComplexTupleStruct, NoCustomTypeExtension>();
        let value = MyComplexTupleStruct(
            vec![1, 2, 3],
            vec![],
            vec![],
            vec![1, 2, 3],
            indexmap! {
                TestEnum::UnitVariant => MyFieldStruct { field1: 1, field2: vec!["hello".to_string()] },
                TestEnum::SingleFieldVariant { field: 1 } => MyFieldStruct { field1: 2, field2: vec!["world".to_string()] },
                TestEnum::DoubleStructVariant { field1: 1, field2: 2 } => MyFieldStruct { field1: 3, field2: vec!["!".to_string()] },
            },
            btreemap! {
                "hello".to_string() => MyUnitStruct,
                "world".to_string() => MyUnitStruct,
            },
            TestEnum::UnitVariant,
            TestEnum::SingleFieldVariant { field: 1 },
            TestEnum::DoubleStructVariant {
                field1: 3,
                field2: 5,
            },
            MyFieldStruct {
                field1: 21,
                field2: vec!["hello".to_string(), "world!".to_string()],
            },
            vec![MyUnitStruct, MyUnitStruct],
            Value::Tuple {
                fields: vec![
                    Value::Enum {
                        discriminator: 32,
                        fields: vec![],
                    },
                    Value::Enum {
                        discriminator: 21,
                        fields: vec![Value::I32 { value: -3 }],
                    },
                ],
            },
        );
        let payload = basic_encode(&value).unwrap();

        let expected_simple = json!([
            [1, 2, 3],
            [],
            { "hex": "" },
            { "hex": "010203" },
            // IndexMap<TestEnum, MyFieldStruct>
            [
                [
                    { "variant_id": 0, "variant_name": "UnitVariant", "fields": [] },
                    { "field1": "1", "field2": ["hello"] }
                ],
                [
                    { "variant_id": 1, "variant_name": "SingleFieldVariant", "fields": { "field": 1 } },
                    { "field1": "2", "field2": ["world"] }
                ],
                [
                    { "variant_id": 2, "variant_name": "DoubleStructVariant", "fields": { "field1": 1, "field2": 2 } },
                    { "field1": "3", "field2": ["!"] }
                ]
            ],
            // BTreeMap<String, MyUnitStruct>
            { "hello": [], "world": [] },
            { "variant_id": 0, "variant_name": "UnitVariant", "fields": [] },
            { "variant_id": 1, "variant_name": "SingleFieldVariant", "fields": { "field": 1 } },
            { "variant_id": 2, "variant_name": "DoubleStructVariant", "fields": { "field1": 3, "field2": 5 } },
            { "field1": "21", "field2": ["hello","world!"] },
            [[],[]],
            [
                { "variant_id": 32, "fields": [], },
                { "variant_id": 21, "fields": [-3], },
            ]
        ]);

        assert_json_eq(
            SborPayloadWithSchema::<NoCustomTypeExtension>::new(&payload, type_index).serializable(
                SerializationContext {
                    mode: SerializationMode::Simple,
                    schema: &schema,
                    custom_context: (),
                },
            ),
            expected_simple,
        );

        let expected_invertible = json!({
            "fields": [
                {
                    "element_kind": "U16",
                    "elements": [
                        1,
                        2,
                        3
                    ],
                    "kind": "Array"
                },
                {
                    "element_kind": "U16",
                    "elements": [

                    ],
                    "kind": "Array"
                },
                {
                    "element_kind": "U8",
                    "hex": "",
                    "kind": "Array",
                    "name": "Bytes"
                },
                {
                    "element_kind": "U8",
                    "hex": "010203",
                    "kind": "Array",
                    "name": "Bytes"
                },
                {
                    "entries": [
                        [
                            {
                                "fields": [

                                ],
                                "variant_name": "UnitVariant",
                                "variant_id": 0
                            },
                            [
                                {
                                    "key": "field1",
                                    "kind": "U64",
                                    "value": "1"
                                },
                                {
                                    "element_kind": "String",
                                    "elements": [
                                        "hello"
                                    ],
                                    "key": "field2",
                                    "kind": "Array"
                                }
                            ]
                        ],
                        [
                            {
                                "fields": [
                                    {
                                        "key": "field",
                                        "kind": "U8",
                                        "value": 1
                                    }
                                ],
                                "variant_name": "SingleFieldVariant",
                                "variant_id": 1
                            },
                            [
                                {
                                    "key": "field1",
                                    "kind": "U64",
                                    "value": "2"
                                },
                                {
                                    "element_kind": "String",
                                    "elements": [
                                        "world"
                                    ],
                                    "key": "field2",
                                    "kind": "Array"
                                }
                            ],
                        ],
                        [
                            {
                                "fields": [
                                    {
                                        "key": "field1",
                                        "kind": "U8",
                                        "value": 1
                                    },
                                    {
                                        "key": "field2",
                                        "kind": "U8",
                                        "value": 2
                                    }
                                ],
                                "variant_name": "DoubleStructVariant",
                                "variant_id": 2
                            },
                            [
                                {
                                    "key": "field1",
                                    "kind": "U64",
                                    "value": "3"
                                },
                                {
                                    "element_kind": "String",
                                    "elements": [
                                        "!"
                                    ],
                                    "key": "field2",
                                    "kind": "Array"
                                }
                            ]
                        ]
                    ],
                    "key_kind": "Enum",
                    "key_name": "TestEnum",
                    "kind": "Map",
                    "value_kind": "Tuple",
                    "value_name": "MyFieldStruct"
                },
                {
                    "entries": [
                        [
                            "hello",
                            []
                        ],
                        [
                            "world",
                            []
                        ]
                    ],
                    "key_kind": "String",
                    "kind": "Map",
                    "value_kind": "Tuple",
                    "value_name": "MyUnitStruct"
                },
                {
                    "fields": [

                    ],
                    "kind": "Enum",
                    "name": "TestEnum",
                    "variant_name": "UnitVariant",
                    "variant_id": 0
                },
                {
                    "fields": [
                        {
                            "key": "field",
                            "kind": "U8",
                            "value": 1
                        }
                    ],
                    "kind": "Enum",
                    "name": "TestEnum",
                    "variant_name": "SingleFieldVariant",
                    "variant_id": 1
                },
                {
                    "fields": [
                        {
                            "key": "field1",
                            "kind": "U8",
                            "value": 3
                        },
                        {
                            "key": "field2",
                            "kind": "U8",
                            "value": 5
                        }
                    ],
                    "kind": "Enum",
                    "name": "TestEnum",
                    "variant_name": "DoubleStructVariant",
                    "variant_id": 2
                },
                {
                    "fields": [
                        {
                            "key": "field1",
                            "kind": "U64",
                            "value": "21"
                        },
                        {
                            "element_kind": "String",
                            "elements": [
                                "hello",
                                "world!"
                            ],
                            "key": "field2",
                            "kind": "Array"
                        }
                    ],
                    "kind": "Tuple",
                    "name": "MyFieldStruct"
                },
                {
                    "element_kind": "Tuple",
                    "element_name": "MyUnitStruct",
                    "elements": [
                        [

                        ],
                        [

                        ]
                    ],
                    "kind": "Array"
                },
                {
                    "fields": [
                        {
                            "fields": [

                            ],
                            "kind": "Enum",
                            "variant_id": 32
                        },
                        {
                            "fields": [
                                {
                                    "kind": "I32",
                                    "value": -3
                                }
                            ],
                            "kind": "Enum",
                            "variant_id": 21
                        }
                    ],
                    "kind": "Tuple"
                }
            ],
            "kind": "Tuple",
            "name": "MyComplexTupleStruct"
        });

        assert_json_eq(
            SborPayloadWithSchema::<NoCustomTypeExtension>::new(&payload, type_index).serializable(
                SerializationContext {
                    mode: SerializationMode::Invertible,
                    schema: &schema,
                    custom_context: (),
                },
            ),
            expected_invertible,
        );
    }
}
