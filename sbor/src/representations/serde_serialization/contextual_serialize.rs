use super::*;
use crate::rust::prelude::*;
use crate::traversal::*;
use crate::*;
use serde::Serializer;
use utils::*;

pub enum SerializationParameters<'s, 'a, 'c, E: SerializableCustomTypeExtension> {
    Schemaless {
        mode: SerializationMode,
        custom_display_context: E::CustomDisplayContext<'a>,
        custom_validation_context: &'c E::CustomValidationContext,
    },
    WithSchema {
        mode: SerializationMode,
        custom_display_context: E::CustomDisplayContext<'a>,
        custom_validation_context: &'c E::CustomValidationContext,
        schema: &'s Schema<E>,
        type_index: LocalTypeIndex,
    },
}

impl<'s, 'a, 'c, E: SerializableCustomTypeExtension> SerializationParameters<'s, 'a, 'c, E> {
    pub fn get_context_and_type_index(
        &self,
    ) -> (SerializationContext<'s, 'a, 'c, E>, LocalTypeIndex) {
        match self {
            SerializationParameters::Schemaless {
                mode,
                custom_display_context,
                custom_validation_context,
            } => (
                SerializationContext {
                    schema: E::empty_schema(),
                    mode: *mode,
                    custom_display_context: *custom_display_context,
                    custom_validation_context: *custom_validation_context,
                },
                LocalTypeIndex::any(),
            ),
            SerializationParameters::WithSchema {
                mode,
                custom_display_context,
                custom_validation_context,
                schema,
                type_index,
            } => (
                SerializationContext {
                    schema: *schema,
                    mode: *mode,
                    custom_display_context: *custom_display_context,
                    custom_validation_context: *custom_validation_context,
                },
                *type_index,
            ),
        }
    }
}

impl<'s, 'a, 'b, 'c, E: SerializableCustomTypeExtension>
    ContextualSerialize<SerializationParameters<'s, 'a, 'c, E>> for RawPayload<'b, E>
{
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &SerializationParameters<'s, 'a, 'c, E>,
    ) -> Result<S::Ok, S::Error> {
        let (context, type_index) = context.get_context_and_type_index();
        serialize_payload(serializer, self.payload_bytes(), &context, type_index)
    }
}

impl<'s, 'a, 'b, 'c, E: SerializableCustomTypeExtension>
    ContextualSerialize<SerializationParameters<'s, 'a, 'c, E>> for RawValue<'b, E>
{
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &SerializationParameters<'s, 'a, 'c, E>,
    ) -> Result<S::Ok, S::Error> {
        let (context, type_index) = context.get_context_and_type_index();
        serialize_partial_payload(
            serializer,
            self.value_body_bytes(),
            ExpectedStart::ValueBody(self.value_kind()),
            true,
            0,
            &context,
            type_index,
        )
    }
}
