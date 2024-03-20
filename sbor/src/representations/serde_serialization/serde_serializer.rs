use super::*;
use crate::rust::cell::RefCell;
use crate::rust::prelude::*;
use crate::traversal::*;
use crate::*;
use radix_rust::*;
use serde::ser::*;
use TypedTraversalEvent::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SerializationMode {
    /// The "(Annotated) Programmatic" encoding is a default invertible API format which captures the
    /// SBOR value model, and supports optional name annotations from a schema.
    ///
    /// SBOR values are generally wrapped in an object with a "kind" field. Fields are output as an
    /// array to faithfully represent the ordering in the SBOR value.
    ///
    /// If a schema is available, variant names, type names and field names are added to the output.
    ///
    /// If value/type data is included in the parent (Vecs and Map entries), it is still duplicated
    /// on the child object for simplicity.
    Programmatic,
    /// ==THIS FORMAT IS DEPRECATED - BUT KEPT FOR THE TIME BEING FOR NODE COMPATIBILITY==
    ///
    /// This "Model" encoding is intended to exactly capture the content of the scrypto value,
    /// in a way which can be inverted back into a scrypto value.
    ///
    /// SBOR values are generally wrapped in an object with a "kind" field. Fields are output as an
    /// array to faithfully represent the ordering in the SBOR value.
    ///
    /// It is more compact than the Programmatic format, but more complex to describe, because
    /// children of arrays/maps are not wrapped in a JSON object with the kind field.
    ///
    /// If value/type data is included in the parent (Vecs and Map entries), it is not duplicated
    /// on the values. This avoids duplication in the output. In these cases, child tuples and
    /// single values lose their wrapper object, to keep the output concise. Other values keep
    /// their wrapper object, as there are other fields to convey.
    ///
    /// If a schema is available, variant names, type names and field names are added to the output.
    Model,
    /// ==THIS FORMAT IS NOT INTENDED TO BE USED YET==
    ///
    /// An API format designed for elegantly reading values with a well-known schema - intended for
    /// eg DApp Builders writing their front-ends.
    ///
    /// It outputs values in a “JSON-native” manner - designed primary for use with a schema,
    /// and for mapping into models like you’d find on an Open API schema.
    ///
    /// Its JSON schema is dependent on its SBOR schema, and it's not intended to be invertible
    /// without the SBOR schema. We could even consider generating an Open API schema for a given
    /// SBOR schema (eg for a blueprint) and allow developers to have a strongly-typed API to their
    /// blueprint.
    ///
    /// Compared with Programmatic, it is more compact, but doesn't include type names / enum variant names.
    ///
    /// It should favour simplicity for human comprehension, in particular:
    /// * It uses a JSON object rather than an array where possible, even if this loses field ordering
    ///   EG for structs, and for maps with string keys.
    /// * If the concept which is being represented (eg number/amount or address) is clear
    ///   to a human, information about the value kind is dropped.
    /// * It prefers to use the JSON number type where the number can be confirmed to be <= JS max safe int.
    Natural,
}

#[derive(Debug, Clone, Copy)]
pub struct SerializationContext<'s, 'a, E: SerializableCustomExtension> {
    pub schema: &'s Schema<E::CustomSchema>,
    pub mode: SerializationMode,
    pub custom_context: E::CustomDisplayContext<'a>,
}

pub(crate) fn serialize_payload<S: Serializer, E: SerializableCustomExtension>(
    serializer: S,
    payload: &[u8],
    context: &SerializationContext<'_, '_, E>,
    type_id: LocalTypeId,
    depth_limit: usize,
) -> Result<S::Ok, S::Error> {
    let mut traverser = traverse_payload_with_types(payload, context.schema, type_id, depth_limit);
    let success =
        serialize_value_tree::<S, E>(serializer, &mut traverser, context, &ValueContext::Default)?;
    consume_end_event::<S, E>(&mut traverser)?;
    Ok(success)
}

pub(crate) fn serialize_partial_payload<S: Serializer, E: SerializableCustomExtension>(
    serializer: S,
    partial_payload: &[u8],
    expected_start: ExpectedStart<E::CustomValueKind>,
    check_exact_end: bool,
    current_depth: usize,
    context: &SerializationContext<'_, '_, E>,
    type_id: LocalTypeId,
    depth_limit: usize,
) -> Result<S::Ok, S::Error> {
    let mut traverser = traverse_partial_payload_with_types(
        partial_payload,
        expected_start,
        check_exact_end,
        current_depth,
        context.schema,
        type_id,
        depth_limit,
    );
    let success =
        serialize_value_tree::<S, E>(serializer, &mut traverser, context, &ValueContext::Default)?;
    if check_exact_end {
        consume_end_event::<S, E>(&mut traverser)?;
    }
    Ok(success)
}

fn consume_end_event<S: Serializer, E: SerializableCustomExtension>(
    traverser: &mut TypedTraverser<E>,
) -> Result<(), S::Error> {
    traverser.consume_end_event().map_err(S::Error::custom)
}

fn consume_container_end_event<S: Serializer, E: SerializableCustomExtension>(
    traverser: &mut TypedTraverser<E>,
) -> Result<(), S::Error> {
    traverser
        .consume_container_end_event()
        .map_err(S::Error::custom)
}

fn map_unexpected_event<S: Serializer, E: SerializableCustomExtension>(
    context: &SerializationContext<'_, '_, E>,
    expected: &'static str,
    typed_event: TypedLocatedTraversalEvent<E>,
) -> S::Error {
    S::Error::custom(typed_event.display_as_unexpected_event(expected, &context.schema))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SerializationError<E: CustomExtension> {
    TraversalError(TypedTraversalError<E>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueContext {
    /// So it doesn't need to include its own kind details
    VecOrMapChild,
    /// The default context - should include its own kind details
    Default,
    /// A named field wrapper - should include its own kind details, and a field_name
    IncludeFieldName { field_name: String },
}

struct SerializableValueTree<'t, 'de, 's1, E: CustomExtension> {
    traverser: RefCell<&'t mut TypedTraverser<'de, 's1, E>>,
    value_context: ValueContext,
}

impl<'t, 'de, 's1, E: SerializableCustomExtension> SerializableValueTree<'t, 'de, 's1, E> {
    fn new(traverser: &'t mut TypedTraverser<'de, 's1, E>, value_context: ValueContext) -> Self {
        Self {
            traverser: RefCell::new(traverser),
            value_context,
        }
    }
}

impl<'t, 'de, 's1, 's, 'a, E: SerializableCustomExtension>
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

fn serialize_value_tree<S: Serializer, E: SerializableCustomExtension>(
    serializer: S,
    traverser: &mut TypedTraverser<E>,
    context: &SerializationContext<'_, '_, E>,
    value_context: &ValueContext,
) -> Result<S::Ok, S::Error> {
    let typed_event = traverser.next_event();
    match typed_event.event {
        ContainerStart(type_id, container_header) => match container_header {
            ContainerHeader::Tuple(header) => serialize_tuple(
                serializer,
                traverser,
                context,
                type_id,
                header,
                value_context,
            ),
            ContainerHeader::EnumVariant(header) => serialize_enum_variant(
                serializer,
                traverser,
                context,
                type_id,
                header,
                value_context,
            ),
            ContainerHeader::Array(header) => serialize_array(
                serializer,
                traverser,
                context,
                type_id,
                header,
                value_context,
            ),
            ContainerHeader::Map(header) => serialize_map(
                serializer,
                traverser,
                context,
                type_id,
                header,
                value_context,
            ),
        },
        TerminalValue(type_id, value_ref) => {
            serialize_terminal_value(serializer, context, type_id, value_ref, value_context)
        }
        _ => Err(map_unexpected_event::<S, E>(
            context,
            "ContainerStart | TerminalValue",
            typed_event,
        )),
    }
}

/// Consumes the number of value-trees from the traverser, and depending on
/// the serialization mode and presence of field names, either outputs as a
/// serde map/JSON object or serde tuple/JSON array.
///
/// Note that it doesn't consume the container end event, because it could also
/// be used for a set of sub-fields.
pub struct SerializableFields<'t, 'de, 's1, 's2, E: CustomExtension> {
    traverser: RefCell<&'t mut TypedTraverser<'de, 's1, E>>,
    field_names: Option<&'s2 [Cow<'static, str>]>,
    length: usize,
}

impl<'t, 'de, 's1, 's2, E: CustomExtension> SerializableFields<'t, 'de, 's1, 's2, E> {
    fn new(
        traverser: &'t mut TypedTraverser<'de, 's1, E>,
        field_names: Option<&'s2 [Cow<'static, str>]>,
        length: usize,
    ) -> Self {
        Self {
            traverser: RefCell::new(traverser),
            field_names,
            length,
        }
    }
}

impl<'t, 'de, 's1, 's2, 's, 'a, E: SerializableCustomExtension>
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
            &self.field_names,
            self.length,
        )
    }
}

fn serialize_fields_to_value<S: Serializer, E: SerializableCustomExtension>(
    serializer: S,
    traverser: &mut TypedTraverser<E>,
    context: &SerializationContext<'_, '_, E>,
    field_names: &Option<&'_ [Cow<'_, str>]>,
    length: usize,
) -> Result<S::Ok, S::Error> {
    match (context.mode, field_names) {
        // In simple mode, we serialize structs as JSON objects
        (SerializationMode::Natural, Some(field_names)) if field_names.len() == length => {
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
        (_, Some(field_names)) if field_names.len() == length => {
            let mut serde_tuple = serializer.serialize_tuple(length)?;
            for field_name in field_names.iter() {
                serde_tuple.serialize_element(
                    &SerializableValueTree::new(
                        traverser,
                        ValueContext::IncludeFieldName {
                            field_name: field_name.to_string(),
                        },
                    )
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

pub struct SerializableMapEntry<'t, 'de, 's1, E: CustomExtension> {
    traverser: RefCell<&'t mut TypedTraverser<'de, 's1, E>>,
}

impl<'t, 'de, 's1, E: CustomExtension> SerializableMapEntry<'t, 'de, 's1, E> {
    fn new(traverser: &'t mut TypedTraverser<'de, 's1, E>) -> Self {
        Self {
            traverser: RefCell::new(traverser),
        }
    }
}

impl<'t, 'de, 's1, 's, 'a, E: SerializableCustomExtension>
    ContextualSerialize<SerializationContext<'s, 'a, E>> for SerializableMapEntry<'t, 'de, 's1, E>
{
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &SerializationContext<'s, 'a, E>,
    ) -> Result<S::Ok, S::Error> {
        let traverser = &mut self.traverser.borrow_mut();
        let mut serde_tuple = serializer.serialize_map(Some(2))?;
        serde_tuple.serialize_entry(
            "key",
            &SerializableValueTree::new(traverser, ValueContext::VecOrMapChild)
                .serializable(*context),
        )?;
        serde_tuple.serialize_entry(
            "value",
            &SerializableValueTree::new(traverser, ValueContext::VecOrMapChild)
                .serializable(*context),
        )?;
        serde_tuple.end()
    }
}

pub struct SerializableArrayElements<'t, 'de, 's1, E: CustomExtension> {
    traverser: RefCell<&'t mut TypedTraverser<'de, 's1, E>>,
    length: usize,
}

impl<'t, 'de, 's1, E: CustomExtension> SerializableArrayElements<'t, 'de, 's1, E> {
    fn new(traverser: &'t mut TypedTraverser<'de, 's1, E>, length: usize) -> Self {
        Self {
            traverser: RefCell::new(traverser),
            length,
        }
    }
}

impl<'t, 'de, 's1, 's, 'a, E: SerializableCustomExtension>
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
        consume_container_end_event::<S, E>(traverser)?;
        serde_tuple.end()
    }
}

pub struct SerializableMapElements<'t, 'de, 's1, E: CustomExtension> {
    traverser: RefCell<&'t mut TypedTraverser<'de, 's1, E>>,
    key_value_kind: ValueKind<E::CustomValueKind>,
    length: usize,
}

impl<'t, 'de, 's1, E: CustomExtension> SerializableMapElements<'t, 'de, 's1, E> {
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

impl<'t, 'de, 's1, 's, 'a, E: SerializableCustomExtension>
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
            // If string keys and in simple mode, then serialize as a JSON object
            (SerializationMode::Natural, ValueKind::String) => {
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
                consume_container_end_event::<S, E>(traverser)?;
                serde_map.end()
            }
            // Otherwise, serialize as a JSON array of key-value objects, which keeps the order, and allows for
            // complex keys
            _ => {
                let mut serde_tuple = serializer.serialize_tuple(self.length)?;
                for _ in 0..self.length {
                    serde_tuple.serialize_element(
                        &SerializableMapEntry::new(traverser).serializable(*context),
                    )?;
                }
                consume_container_end_event::<S, E>(traverser)?;
                serde_tuple.end()
            }
        }
    }
}

fn serialize_tuple<S: Serializer, E: SerializableCustomExtension>(
    serializer: S,
    traverser: &mut TypedTraverser<E>,
    context: &SerializationContext<'_, '_, E>,
    type_id: LocalTypeId,
    tuple_header: TupleHeader,
    value_context: &ValueContext,
) -> Result<S::Ok, S::Error> {
    let tuple_metadata = context
        .schema
        .resolve_matching_tuple_metadata(type_id, tuple_header.length);
    let mut map_aggregator = SerdeValueMapAggregator::new(context, value_context);

    if !map_aggregator.should_embed_value_in_contextual_json_map() {
        let result_ok = SerializableFields::new(
            traverser,
            tuple_metadata.field_names.into(),
            tuple_header.length,
        )
        .serialize(serializer, *context)?;
        consume_container_end_event::<S, E>(traverser)?;
        return Ok(result_ok);
    }
    map_aggregator.add_initial_details(ValueKind::Tuple, tuple_metadata.name);
    map_aggregator.add_field(
        "fields",
        SerializableType::SerializableFields(SerializableFields::new(
            traverser,
            tuple_metadata.field_names.into(),
            tuple_header.length,
        )),
    );
    let success = map_aggregator.into_map(serializer)?;
    consume_container_end_event::<S, E>(traverser)?;
    Ok(success)
}

fn serialize_enum_variant<'s, S: Serializer, E: SerializableCustomExtension>(
    serializer: S,
    traverser: &mut TypedTraverser<E>,
    context: &SerializationContext<'s, '_, E>,
    type_id: LocalTypeId,
    variant_header: EnumVariantHeader,
    value_context: &ValueContext,
) -> Result<S::Ok, S::Error> {
    let enum_metadata = context.schema.resolve_matching_enum_metadata(
        type_id,
        variant_header.variant,
        variant_header.length,
    );
    let mut map_aggregator = SerdeValueMapAggregator::new(context, value_context);

    map_aggregator.add_initial_details(ValueKind::Enum, enum_metadata.enum_name);
    map_aggregator.add_enum_variant_details(variant_header.variant, enum_metadata.variant_name);
    map_aggregator.add_field(
        "fields",
        SerializableType::SerializableFields(SerializableFields::new(
            traverser,
            enum_metadata.field_names,
            variant_header.length,
        )),
    );
    let success = map_aggregator.into_map(serializer)?;
    consume_container_end_event::<S, E>(traverser)?;
    Ok(success)
}

fn serialize_array<S: Serializer, E: SerializableCustomExtension>(
    serializer: S,
    traverser: &mut TypedTraverser<E>,
    context: &SerializationContext<'_, '_, E>,
    type_id: LocalTypeId,
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
    let array_metadata = context.schema.resolve_matching_array_metadata(type_id);
    if array_header.element_value_kind == ValueKind::U8 {
        map_aggregator
            .add_initial_details_with_custom_value_kind_name("Bytes", array_metadata.array_name);
    } else {
        map_aggregator.add_initial_details(ValueKind::Array, array_metadata.array_name);
    }
    map_aggregator
        .add_element_details(array_header.element_value_kind, array_metadata.element_name);

    match (array_header.element_value_kind, array_header.length) {
        (ValueKind::U8, 0) => {
            map_aggregator.add_field("hex", SerializableType::Str(""));
            consume_container_end_event::<S, E>(traverser)?;
        }
        (ValueKind::U8, _) => {
            let typed_event = traverser.next_event();
            match typed_event.event {
                TerminalValueBatch(_, TerminalValueBatchRef::U8(bytes)) => {
                    map_aggregator.add_field("hex", SerializableType::String(hex::encode(bytes)));
                }
                _ => Err(map_unexpected_event::<S, E>(
                    context,
                    "TerminalValueBatch",
                    typed_event,
                ))?,
            };
            consume_container_end_event::<S, E>(traverser)?;
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

fn serialize_map<S: Serializer, E: SerializableCustomExtension>(
    serializer: S,
    traverser: &mut TypedTraverser<E>,
    context: &SerializationContext<'_, '_, E>,
    type_id: LocalTypeId,
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
    let map_metadata = context.schema.resolve_matching_map_metadata(type_id);
    map_aggregator.add_initial_details(ValueKind::Map, map_metadata.map_name);
    map_aggregator.add_map_child_details(
        map_header.key_value_kind,
        map_header.value_value_kind,
        &map_metadata,
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

fn serialize_terminal_value<S: Serializer, E: SerializableCustomExtension>(
    serializer: S,
    context: &SerializationContext<'_, '_, E>,
    type_id: LocalTypeId,
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
            } = E::map_value_for_serialization(context, type_id, custom_value);
            (serialization, include_type_tag_in_simple_mode)
        }
    };
    let mut map_aggregator = if include_type_tag_in_simple_mode {
        SerdeValueMapAggregator::new_with_kind_tag(context, value_context)
    } else {
        SerdeValueMapAggregator::new(context, value_context)
    };
    if map_aggregator.should_embed_value_in_contextual_json_map() {
        let type_name = context.schema.resolve_type_name_from_metadata(type_id);
        map_aggregator.add_initial_details(value_kind, type_name);
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
                {
                    "key": 153,
                    "value": 62
                }
            ],
            [
                "hello",
                1234
            ]
        ]);

        let payload = basic_encode(&value).unwrap();
        assert_json_eq(
            BasicRawPayload::new_from_valid_slice_with_checks(&payload)
                .unwrap()
                .serializable(SerializationParameters::Schemaless {
                    mode: SerializationMode::Natural,
                    custom_context: (),
                    depth_limit: 64,
                }),
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

    #[derive(BasicSbor)]
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
        let (type_id, schema) =
            generate_full_schema_from_single_type::<MyComplexTupleStruct, NoCustomSchema>();
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

        let expected_programmatic = json!({
            "kind": "Tuple",
            "type_name": "MyComplexTupleStruct",
            "fields": [
                {
                    "kind": "Array",
                    "element_kind": "U16",
                    "elements": [
                        { "kind": "U16", "value": "1" },
                        { "kind": "U16", "value": "2" },
                        { "kind": "U16", "value": "3" },
                    ]
                },
                {
                    "kind": "Array",
                    "element_kind": "U16",
                    "elements": []
                },
                {
                    "kind": "Bytes",
                    "element_kind": "U8",
                    "hex": ""
                },
                {
                    "kind": "Bytes",
                    "element_kind": "U8",
                    "hex": "010203"
                },
                {
                    "kind": "Map",
                    "key_kind": "Enum",
                    "key_type_name": "TestEnum",
                    "value_kind": "Tuple",
                    "value_type_name": "MyFieldStruct",
                    "entries": [
                        {
                            "key": {
                                "kind": "Enum",
                                "type_name": "TestEnum",
                                "variant_id": "0",
                                "variant_name": "UnitVariant",
                                "fields": []
                            },
                            "value": {
                                "kind": "Tuple",
                                "type_name": "MyFieldStruct",
                                "fields": [
                                    {
                                        "kind": "U64",
                                        "field_name": "field1",
                                        "value": "1"
                                    },
                                    {
                                        "kind": "Array",
                                        "field_name": "field2",
                                        "element_kind": "String",
                                        "elements": [
                                            {
                                                "kind": "String",
                                                "value": "hello"
                                            },
                                        ]
                                    }
                                ]
                            }
                        },
                        {
                            "key": {
                                "kind": "Enum",
                                "type_name": "TestEnum",
                                "variant_id": "1",
                                "variant_name": "SingleFieldVariant",
                                "fields": [
                                    {
                                        "kind": "U8",
                                        "field_name": "field",
                                        "value": "1"
                                    }
                                ]
                            },
                            "value": {
                                "kind": "Tuple",
                                "type_name": "MyFieldStruct",
                                "fields": [
                                    {
                                        "kind": "U64",
                                        "field_name": "field1",
                                        "value": "2"
                                    },
                                    {
                                        "kind": "Array",
                                        "field_name": "field2",
                                        "element_kind": "String",
                                        "elements": [
                                            {
                                                "kind": "String",
                                                "value": "world"
                                            },
                                        ]
                                    }
                                ]
                            }
                        },
                        {
                            "key": {
                                "kind": "Enum",
                                "type_name": "TestEnum",
                                "variant_id": "2",
                                "variant_name": "DoubleStructVariant",
                                "fields": [
                                    {
                                        "kind": "U8",
                                        "field_name": "field1",
                                        "value": "1"
                                    },
                                    {
                                        "kind": "U8",
                                        "field_name": "field2",
                                        "value": "2"
                                    }
                                ]
                            },
                            "value": {
                                "kind": "Tuple",
                                "type_name": "MyFieldStruct",
                                "fields": [
                                    {
                                        "kind": "U64",
                                        "field_name": "field1",
                                        "value": "3"
                                    },
                                    {
                                        "kind": "Array",
                                        "field_name": "field2",
                                        "element_kind": "String",
                                        "elements": [
                                            {
                                                "kind": "String",
                                                "value": "!"
                                            },
                                        ]
                                    }
                                ]
                            }
                        }
                    ]
                },
                {
                    "kind": "Map",
                    "key_kind": "String",
                    "value_kind": "Tuple",
                    "value_type_name": "MyUnitStruct",
                    "entries": [
                        {
                            "key": {
                                "kind": "String",
                                "value": "hello"
                            },
                            "value": {
                                "kind": "Tuple",
                                "type_name": "MyUnitStruct",
                                "fields": []
                            },
                        },
                        {
                            "key": {
                                "kind": "String",
                                "value": "world"
                            },
                            "value": {
                                "kind": "Tuple",
                                "type_name": "MyUnitStruct",
                                "fields": []
                            },
                        }
                    ]
                },
                {
                    "kind": "Enum",
                    "type_name": "TestEnum",
                    "variant_id": "0",
                    "variant_name": "UnitVariant",
                    "fields": []
                },
                {
                    "kind": "Enum",
                    "type_name": "TestEnum",
                    "variant_id": "1",
                    "variant_name": "SingleFieldVariant",
                    "fields": [
                        {
                            "kind": "U8",
                            "field_name": "field",
                            "value": "1"
                        }
                    ]
                },
                {
                    "kind": "Enum",
                    "type_name": "TestEnum",
                    "variant_id": "2",
                    "variant_name": "DoubleStructVariant",
                    "fields": [
                        {
                            "kind": "U8",
                            "field_name": "field1",
                            "value": "3"
                        },
                        {
                            "kind": "U8",
                            "field_name": "field2",
                            "value": "5"
                        }
                    ]
                },
                {
                    "kind": "Tuple",
                    "type_name": "MyFieldStruct",
                    "fields": [
                        {
                            "kind": "U64",
                            "field_name": "field1",
                            "value": "21"
                        },
                        {
                            "kind": "Array",
                            "field_name": "field2",
                            "element_kind": "String",
                            "elements": [
                                {
                                    "kind": "String",
                                    "value": "hello"
                                },
                                {
                                    "kind": "String",
                                    "value": "world!"
                                },
                            ]
                        }
                    ]
                },
                {
                    "kind": "Array",
                    "element_kind": "Tuple",
                    "element_name": "MyUnitStruct",
                    "elements": [
                        {
                            "kind": "Tuple",
                            "type_name": "MyUnitStruct",
                            "fields": []
                        },
                        {
                            "kind": "Tuple",
                            "type_name": "MyUnitStruct",
                            "fields": []
                        },
                    ]
                },
                {
                    "kind": "Tuple",
                    "fields": [
                        {
                            "kind": "Enum",
                            "variant_id": "32",
                            "fields": []
                        },
                        {
                            "kind": "Enum",
                            "variant_id": "21",
                            "fields": [
                                {
                                    "kind": "I32",
                                    "value": "-3"
                                }
                            ]
                        }
                    ]
                }
            ]
        });

        assert_json_eq(
            BasicRawPayload::new_from_valid_slice_with_checks(&payload)
                .unwrap()
                .serializable(SerializationParameters::WithSchema {
                    mode: SerializationMode::Programmatic,
                    schema: schema.v1(),
                    custom_context: (),
                    type_id,
                    depth_limit: 64,
                }),
            expected_programmatic,
        );

        let expected_natural = json!([
            [1, 2, 3],
            [],
            { "hex": "" },
            { "hex": "010203" },
            // IndexMap<TestEnum, MyFieldStruct>
            [
                {
                    "key": { "variant_id": 0, "variant_name": "UnitVariant", "fields": [] },
                    "value": { "field1": "1", "field2": ["hello"] }
                },
                {
                    "key": { "variant_id": 1, "variant_name": "SingleFieldVariant", "fields": { "field": 1 } },
                    "value": { "field1": "2", "field2": ["world"] }
                },
                {
                    "key": { "variant_id": 2, "variant_name": "DoubleStructVariant", "fields": { "field1": 1, "field2": 2 } },
                    "value": { "field1": "3", "field2": ["!"] }
                }
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
            BasicRawPayload::new_from_valid_slice_with_checks(&payload)
                .unwrap()
                .serializable(SerializationParameters::WithSchema {
                    mode: SerializationMode::Natural,
                    schema: schema.v1(),
                    custom_context: (),
                    type_id,
                    depth_limit: 64,
                }),
            expected_natural,
        );

        let expected_model = json!({
            "kind": "Tuple",
            "type_name": "MyComplexTupleStruct",
            "fields": [
                {
                    "kind": "Array",
                    "element_kind": "U16",
                    "elements": [
                        1,
                        2,
                        3
                    ]
                },
                {
                    "kind": "Array",
                    "element_kind": "U16",
                    "elements": []
                },
                {
                    "kind": "Bytes",
                    "element_kind": "U8",
                    "hex": ""
                },
                {
                    "kind": "Bytes",
                    "element_kind": "U8",
                    "hex": "010203"
                },
                {
                    "kind": "Map",
                    "key_kind": "Enum",
                    "key_type_name": "TestEnum",
                    "value_kind": "Tuple",
                    "value_type_name": "MyFieldStruct",
                    "entries": [
                        {
                            "key": {
                                "variant_id": 0,
                                "variant_name": "UnitVariant",
                                "fields": []
                            },
                            "value": [
                                {
                                    "kind": "U64",
                                    "field_name": "field1",
                                    "value": "1"
                                },
                                {
                                    "kind": "Array",
                                    "field_name": "field2",
                                    "element_kind": "String",
                                    "elements": [
                                        "hello"
                                    ]
                                }
                            ]
                        },
                        {
                            "key": {
                                "variant_id": 1,
                                "variant_name": "SingleFieldVariant",
                                "fields": [
                                    {
                                        "kind": "U8",
                                        "field_name": "field",
                                        "value": 1
                                    }
                                ]
                            },
                            "value": [
                                {
                                    "kind": "U64",
                                    "field_name": "field1",
                                    "value": "2"
                                },
                                {
                                    "kind": "Array",
                                    "field_name": "field2",
                                    "element_kind": "String",
                                    "elements": [
                                        "world"
                                    ]
                                }
                            ]
                        },
                        {
                            "key": {
                                "variant_id": 2,
                                "variant_name": "DoubleStructVariant",
                                "fields": [
                                    {
                                        "kind": "U8",
                                        "field_name": "field1",
                                        "value": 1
                                    },
                                    {
                                        "kind": "U8",
                                        "field_name": "field2",
                                        "value": 2
                                    }
                                ]
                            },
                            "value": [
                                {
                                    "kind": "U64",
                                    "field_name": "field1",
                                    "value": "3"
                                },
                                {
                                    "kind": "Array",
                                    "field_name": "field2",
                                    "element_kind": "String",
                                    "elements": [
                                        "!"
                                    ]
                                }
                            ]
                        }
                    ]
                },
                {
                    "kind": "Map",
                    "key_kind": "String",
                    "value_kind": "Tuple",
                    "value_type_name": "MyUnitStruct",
                    "entries": [
                        {
                            "key": "hello",
                            "value": []
                        },
                        {
                            "key": "world",
                            "value": []
                        }
                    ]
                },
                {
                    "kind": "Enum",
                    "type_name": "TestEnum",
                    "variant_id": 0,
                    "variant_name": "UnitVariant",
                    "fields": []
                },
                {
                    "kind": "Enum",
                    "type_name": "TestEnum",
                    "variant_id": 1,
                    "variant_name": "SingleFieldVariant",
                    "fields": [
                        {
                            "kind": "U8",
                            "field_name": "field",
                            "value": 1
                        }
                    ]
                },
                {
                    "kind": "Enum",
                    "type_name": "TestEnum",
                    "variant_id": 2,
                    "variant_name": "DoubleStructVariant",
                    "fields": [
                        {
                            "kind": "U8",
                            "field_name": "field1",
                            "value": 3
                        },
                        {
                            "kind": "U8",
                            "field_name": "field2",
                            "value": 5
                        }
                    ]
                },
                {
                    "kind": "Tuple",
                    "type_name": "MyFieldStruct",
                    "fields": [
                        {
                            "kind": "U64",
                            "field_name": "field1",
                            "value": "21"
                        },
                        {
                            "kind": "Array",
                            "field_name": "field2",
                            "element_kind": "String",
                            "elements": [
                                "hello",
                                "world!"
                            ]
                        }
                    ]
                },
                {
                    "kind": "Array",
                    "element_kind": "Tuple",
                    "element_name": "MyUnitStruct",
                    "elements": [
                        [],
                        []
                    ]
                },
                {
                    "kind": "Tuple",
                    "fields": [
                        {
                            "kind": "Enum",
                            "variant_id": 32,
                            "fields": []
                        },
                        {
                            "kind": "Enum",
                            "variant_id": 21,
                            "fields": [
                                {
                                    "kind": "I32",
                                    "value": -3
                                }
                            ]
                        }
                    ]
                }
            ]
        });

        assert_json_eq(
            BasicRawPayload::new_from_valid_slice_with_checks(&payload)
                .unwrap()
                .serializable(SerializationParameters::WithSchema {
                    mode: SerializationMode::Model,
                    schema: schema.v1(),
                    custom_context: (),
                    type_id,
                    depth_limit: 64,
                }),
            expected_model,
        );
    }
}
