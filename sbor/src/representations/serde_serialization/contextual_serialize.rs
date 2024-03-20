use super::*;
use crate::rust::prelude::*;
use crate::traversal::*;
use crate::*;
use radix_rust::*;
use serde::Serializer;

pub enum SerializationParameters<'s, 'a, E: SerializableCustomExtension> {
    Schemaless {
        mode: SerializationMode,
        custom_context: E::CustomDisplayContext<'a>,
        depth_limit: usize,
    },
    WithSchema {
        mode: SerializationMode,
        custom_context: E::CustomDisplayContext<'a>,
        schema: &'s Schema<E::CustomSchema>,
        type_id: LocalTypeId,
        depth_limit: usize,
    },
}

impl<'s, 'a, E: SerializableCustomExtension> SerializationParameters<'s, 'a, E> {
    pub fn get_context_params(&self) -> (SerializationContext<'s, 'a, E>, LocalTypeId, usize) {
        match self {
            SerializationParameters::Schemaless {
                mode,
                custom_context,
                depth_limit,
            } => (
                SerializationContext {
                    schema: E::CustomSchema::empty_schema(),
                    mode: *mode,
                    custom_context: *custom_context,
                },
                LocalTypeId::any(),
                *depth_limit,
            ),
            SerializationParameters::WithSchema {
                mode,
                custom_context,
                schema,
                type_id,
                depth_limit,
            } => (
                SerializationContext {
                    schema: *schema,
                    mode: *mode,
                    custom_context: *custom_context,
                },
                *type_id,
                *depth_limit,
            ),
        }
    }
}

impl<'s, 'a, 'b, E: SerializableCustomExtension>
    ContextualSerialize<SerializationParameters<'s, 'a, E>> for RawPayload<'b, E>
{
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &SerializationParameters<'s, 'a, E>,
    ) -> Result<S::Ok, S::Error> {
        let (context, type_id, depth_limit) = context.get_context_params();
        serialize_payload(
            serializer,
            self.payload_bytes(),
            &context,
            type_id,
            depth_limit,
        )
    }
}

impl<'s, 'a, 'b, E: SerializableCustomExtension>
    ContextualSerialize<SerializationParameters<'s, 'a, E>> for RawValue<'b, E>
{
    fn contextual_serialize<S: Serializer>(
        &self,
        serializer: S,
        context: &SerializationParameters<'s, 'a, E>,
    ) -> Result<S::Ok, S::Error> {
        let (context, type_id, depth_limit) = context.get_context_params();
        serialize_partial_payload(
            serializer,
            self.value_body_bytes(),
            ExpectedStart::ValueBody(self.value_kind()),
            true,
            0,
            &context,
            type_id,
            depth_limit,
        )
    }
}
