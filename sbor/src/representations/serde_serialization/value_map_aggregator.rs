use super::*;
use crate::rust::prelude::*;
use crate::*;
use radix_rust::*;
use serde::ser::*;

// See https://www.possiblerust.com/pattern/3-things-to-try-when-you-can-t-make-a-trait-object
pub enum SerializableType<'a, 't, 'de, 's1, 's2, E: SerializableCustomExtension> {
    String(String),
    Str(&'a str),
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    U8(u8),
    U16(u16),
    U32(u32),
    SerializableFields(SerializableFields<'t, 'de, 's1, 's2, E>),
    SerializableArrayElements(SerializableArrayElements<'t, 'de, 's1, E>),
    SerializableMapElements(SerializableMapElements<'t, 'de, 's1, E>),
}

impl<'a, 'a2, 't, 'de, 's1, 's2, E: SerializableCustomExtension>
    ContextualSerialize<SerializationContext<'s2, 'a2, E>>
    for SerializableType<'a, 't, 'de, 's1, 's2, E>
{
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &SerializationContext<'s2, 'a2, E>,
    ) -> Result<S::Ok, S::Error> {
        match context.mode {
            SerializationMode::Programmatic => match self {
                Self::String(s) => serializer.serialize_str(s),
                Self::Str(s) => serializer.serialize_str(s),
                Self::Bool(b) => serializer.serialize_bool(*b),
                Self::I8(i) => serializer.serialize_str(&i.to_string()),
                Self::I16(i) => serializer.serialize_str(&i.to_string()),
                Self::I32(i) => serializer.serialize_str(&i.to_string()),
                Self::U8(u) => serializer.serialize_str(&u.to_string()),
                Self::U16(u) => serializer.serialize_str(&u.to_string()),
                Self::U32(u) => serializer.serialize_str(&u.to_string()),
                Self::SerializableFields(s) => s.contextual_serialize(serializer, context),
                Self::SerializableArrayElements(s) => s.contextual_serialize(serializer, context),
                Self::SerializableMapElements(s) => s.contextual_serialize(serializer, context),
            },
            SerializationMode::Model | SerializationMode::Natural => match self {
                Self::String(s) => serializer.serialize_str(s),
                Self::Str(s) => serializer.serialize_str(s),
                Self::Bool(b) => serializer.serialize_bool(*b),
                Self::I8(i) => serializer.serialize_i8(*i),
                Self::I16(i) => serializer.serialize_i16(*i),
                Self::I32(i) => serializer.serialize_i32(*i),
                Self::U8(u) => serializer.serialize_u8(*u),
                Self::U16(u) => serializer.serialize_u16(*u),
                Self::U32(u) => serializer.serialize_u32(*u),
                Self::SerializableFields(s) => s.contextual_serialize(serializer, context),
                Self::SerializableArrayElements(s) => s.contextual_serialize(serializer, context),
                Self::SerializableMapElements(s) => s.contextual_serialize(serializer, context),
            },
        }
    }
}

pub struct SerdeValueMapAggregator<'a, 'a2, 't, 'de, 's, 's1, 's2, E: SerializableCustomExtension> {
    context: &'a SerializationContext<'s, 'a2, E>,
    opt_into_kind_tag_in_simple_mode: bool,
    value_context: &'a ValueContext,
    fields: Vec<(&'a str, SerializableType<'a, 't, 'de, 's1, 's2, E>)>,
}

impl<'a, 'a2, 't, 'de, 's, 's1, 's2, E: SerializableCustomExtension>
    SerdeValueMapAggregator<'a, 'a2, 't, 'de, 's, 's1, 's2, E>
{
    pub fn new(
        context: &'a SerializationContext<'s, 'a2, E>,
        value_context: &'a ValueContext,
    ) -> Self {
        Self {
            context,
            opt_into_kind_tag_in_simple_mode: false,
            value_context,
            fields: vec![],
        }
    }

    pub fn new_with_kind_tag(
        context: &'a SerializationContext<'s, 'a2, E>,
        value_context: &'a ValueContext,
    ) -> Self {
        Self {
            context,
            opt_into_kind_tag_in_simple_mode: true,
            value_context,
            fields: vec![],
        }
    }

    /// SBOR values can either be represented just as a JSON value, or in a contextual JSON object.
    /// This contextual object allows for adding extra information (eg type names, kind tags, etc).
    /// As a general rule, Natural uses mostly JSON values, and Programmatic/Model use mostly contextual objects.
    ///
    /// This method returns whether a wrapping object is needed.
    ///
    /// Note that some types _have to_ be embedded in a wrapper object, so
    pub fn should_embed_value_in_contextual_json_map(&self) -> bool {
        match (
            self.context.mode,
            self.opt_into_kind_tag_in_simple_mode,
            self.value_context,
        ) {
            // If we're in simple mode, and we're not a type which has explicitly opted into adding the kind flag, then we don't need to add any details
            (SerializationMode::Natural, false, _) => false,
            // If we're in Model mode, and we're the child of a parent Vec or Map,
            // then our details are already included in the parent, so we don't need the wrapper!
            (SerializationMode::Model, _, ValueContext::VecOrMapChild) => false,
            // Otherwise the wrapper object is needed
            _ => true,
        }
    }

    pub fn child_details_are_needed(&self) -> bool {
        match self.context.mode {
            SerializationMode::Model => true,
            SerializationMode::Programmatic => true,
            SerializationMode::Natural => false,
        }
    }

    pub fn add_initial_details(
        &mut self,
        value_kind: ValueKind<E::CustomValueKind>,
        type_name: Option<&'a str>,
    ) {
        if self.should_embed_value_in_contextual_json_map() {
            self.fields
                .push(("kind", SerializableType::String(value_kind.to_string())));
            type_name.map(|type_name| {
                self.fields
                    .push(("type_name", SerializableType::Str(type_name)))
            });
        }
        if let ValueContext::IncludeFieldName { field_name: key } = self.value_context {
            self.fields.push(("field_name", SerializableType::Str(key)));
        }
    }

    pub fn add_initial_details_with_custom_value_kind_name(
        &mut self,
        value_kind_name: &'a str,
        type_name: Option<&'a str>,
    ) {
        if self.should_embed_value_in_contextual_json_map() {
            self.fields.push((
                "kind",
                SerializableType::String(value_kind_name.to_string()),
            ));
            type_name.map(|type_name| {
                self.fields
                    .push(("type_name", SerializableType::Str(type_name)))
            });
        }
        if let ValueContext::IncludeFieldName { field_name: key } = self.value_context {
            self.fields.push(("field_name", SerializableType::Str(key)));
        }
    }

    pub fn add_element_details(
        &mut self,
        element_value_kind: ValueKind<E::CustomValueKind>,
        element_type_name: Option<&'a str>,
    ) {
        if self.child_details_are_needed() {
            self.fields.push((
                "element_kind",
                SerializableType::String(element_value_kind.to_string()),
            ));
            element_type_name.map(|element_name| {
                self.fields
                    .push(("element_name", SerializableType::Str(element_name)))
            });
        }
    }

    pub fn add_map_child_details(
        &mut self,
        key_value_kind: ValueKind<E::CustomValueKind>,
        value_value_kind: ValueKind<E::CustomValueKind>,
        map_data: &'a MapData,
    ) {
        if self.child_details_are_needed() {
            self.fields.push((
                "key_kind",
                SerializableType::String(key_value_kind.to_string()),
            ));
            map_data.key_name.map(|key_name| {
                self.fields
                    .push(("key_type_name", SerializableType::Str(key_name)))
            });
            self.fields.push((
                "value_kind",
                SerializableType::String(value_value_kind.to_string()),
            ));
            map_data.value_name.map(|value_name| {
                self.fields
                    .push(("value_type_name", SerializableType::Str(value_name)))
            });
        }
    }

    pub fn add_enum_variant_details(&mut self, variant_id: u8, variant_name: Option<&'a str>) {
        self.fields
            .push(("variant_id", SerializableType::U8(variant_id)));
        variant_name.map(|variant_name| {
            self.fields
                .push(("variant_name", SerializableType::Str(variant_name)))
        });
    }

    pub fn add_field(
        &mut self,
        field_name: &'static str,
        value: SerializableType<'a, 't, 'de, 's1, 's2, E>,
    ) {
        self.fields.push((field_name, value));
    }

    pub fn into_map<S: Serializer>(self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.fields.len()))?;
        for (key, value) in self.fields {
            map.serialize_entry(key, &value.serializable(*self.context))?;
        }
        map.end()
    }
}
