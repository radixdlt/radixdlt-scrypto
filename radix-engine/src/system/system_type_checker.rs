use super::payload_validation::*;
use crate::errors::{RuntimeError, SystemError};
use crate::kernel::kernel_api::KernelApi;
use crate::system::system::{KeyValueEntrySubstate, SystemService};
use crate::system::system_callback::{SystemConfig, SystemLockData};
use crate::system::system_callback_api::SystemCallbackObject;
use crate::types::*;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::CollectionIndex;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::schema::KeyValueStoreGenericSubstitutions;
use sbor::rust::vec::Vec;

/// Metadata for schema validation to help with location of certain schemas
/// since location of schemas are somewhat scattered
#[derive(Debug, Clone)]
pub enum SchemaValidationMeta {
    ExistingObject {
        additional_schemas: NodeId,
    },
    NewObject {
        additional_schemas: NonIterMap<SchemaHash, ScryptoSchema>,
    },
    Blueprint,
}

/// The blueprint type to check against along with any additional metadata
/// required to perform validation
#[derive(Debug, Clone)]
pub struct BlueprintTypeTarget {
    pub blueprint_info: BlueprintInfo,
    pub meta: SchemaValidationMeta,
}

/// The key value store to check against along with any additional metadata
/// required to perform validation
#[derive(Debug, Clone)]
pub struct KVStoreValidationTarget {
    pub kv_store_type: KeyValueStoreGenericSubstitutions,
    pub meta: NodeId,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum TypeCheckError {
    InvalidNumberOfGenericArgs { expected: usize, actual: usize },
    InvalidLocalTypeIndex(LocalTypeIndex),
    InvalidCollectionIndex(Box<BlueprintInfo>, CollectionIndex),
    BlueprintPayloadDoesNotExist(Box<BlueprintInfo>, BlueprintPayloadIdentifier),
    BlueprintPayloadValidationError(Box<BlueprintInfo>, BlueprintPayloadIdentifier, String),
    KeyValueStorePayloadValidationError(KeyOrValue, String),
    InstanceSchemaNotFound,
    MissingSchema,
}

impl<'a, Y, V> SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    /// Validate that the type substitutions match the generic definition of a given blueprint
    pub fn validate_bp_generic_args(
        &mut self,
        blueprint_interface: &BlueprintInterface,
        schemas: &IndexMap<SchemaHash, ScryptoSchema>,
        generic_substitutions: &Vec<GenericSubstitution>,
    ) -> Result<(), TypeCheckError> {
        let generics = &blueprint_interface.generics;

        if !generics.len().eq(&generic_substitutions.len()) {
            return Err(TypeCheckError::InvalidNumberOfGenericArgs {
                expected: generics.len(),
                actual: generic_substitutions.len(),
            });
        }

        for generic_substitution in generic_substitutions {
            Self::validate_generic_arg(schemas, generic_substitution)?;
        }

        Ok(())
    }

    /// Validate that the type substitutions for a kv store exist in a given schema
    pub fn validate_kv_store_generic_args(
        &mut self,
        schemas: &IndexMap<SchemaHash, ScryptoSchema>,
        key: &GenericSubstitution,
        value: &GenericSubstitution,
    ) -> Result<(), TypeCheckError> {
        Self::validate_generic_arg(schemas, key)?;
        Self::validate_generic_arg(schemas, value)?;

        Ok(())
    }

    fn validate_generic_arg(
        schemas: &IndexMap<SchemaHash, ScryptoSchema>,
        substitution: &GenericSubstitution,
    ) -> Result<(), TypeCheckError> {
        match substitution {
            GenericSubstitution::Local(type_id) => {
                let schema = schemas
                    .get(&type_id.0)
                    .ok_or_else(|| TypeCheckError::MissingSchema)?;

                if schema.resolve_type_kind(type_id.1).is_none() {
                    return Err(TypeCheckError::InvalidLocalTypeIndex(type_id.1));
                }
            }
        }

        Ok(())
    }

    /// Validate that a blueprint payload matches the blueprint's definition of that payload
    pub fn validate_blueprint_payload(
        &mut self,
        target: &BlueprintTypeTarget,
        payload_identifier: BlueprintPayloadIdentifier,
        payload: &[u8],
    ) -> Result<(), RuntimeError> {
        let blueprint_interface =
            self.get_blueprint_default_interface(target.blueprint_info.blueprint_id.clone())?;

        let (payload_def, allow_ownership, allow_non_global_ref) = blueprint_interface
            .get_payload_def(&payload_identifier)
            .ok_or_else(|| {
                RuntimeError::SystemError(SystemError::TypeCheckError(
                    TypeCheckError::BlueprintPayloadDoesNotExist(
                        Box::new(target.blueprint_info.clone()),
                        payload_identifier.clone(),
                    ),
                ))
            })?;

        // Given the payload definition, retrieve the info to be able to do schema validation on a payload
        let (schema, index, schema_origin) = match payload_def {
            BlueprintPayloadDef::Static(type_identifier) => {
                let schema = self.get_schema(
                    target
                        .blueprint_info
                        .blueprint_id
                        .package_address
                        .as_node_id(),
                    &type_identifier.0,
                )?;
                (
                    schema,
                    type_identifier.1,
                    SchemaOrigin::Blueprint(target.blueprint_info.blueprint_id.clone()),
                )
            }
            BlueprintPayloadDef::Generic(instance_index) => {
                let generic_substitution = target
                    .blueprint_info
                    .generic_substitutions
                    .get(instance_index as usize)
                    .ok_or_else(|| {
                        RuntimeError::SystemError(SystemError::TypeCheckError(
                            TypeCheckError::InstanceSchemaNotFound,
                        ))
                    })?;

                match generic_substitution {
                    GenericSubstitution::Local(type_id) => {
                        let schema = match &target.meta {
                            SchemaValidationMeta::ExistingObject { additional_schemas } => {
                                self.get_schema(additional_schemas, &type_id.0)?
                            }
                            SchemaValidationMeta::NewObject { additional_schemas } => {
                                additional_schemas
                                    .get(&type_id.0)
                                    .ok_or_else(|| {
                                        RuntimeError::SystemError(SystemError::TypeCheckError(
                                            TypeCheckError::InstanceSchemaNotFound,
                                        ))
                                    })?
                                    .clone()
                            }
                            SchemaValidationMeta::Blueprint => {
                                return Err(RuntimeError::SystemError(
                                    SystemError::TypeCheckError(
                                        TypeCheckError::InstanceSchemaNotFound,
                                    ),
                                ));
                            }
                        };

                        (schema, type_id.1, SchemaOrigin::Instance)
                    }
                }
            }
        };

        self.validate_payload(
            payload,
            &schema,
            index,
            schema_origin,
            allow_ownership,
            allow_non_global_ref,
        )
        .map_err(|err| {
            RuntimeError::SystemError(SystemError::TypeCheckError(
                TypeCheckError::BlueprintPayloadValidationError(
                    Box::new(target.blueprint_info.clone()),
                    payload_identifier,
                    err.error_message(&schema),
                ),
            ))
        })?;

        Ok(())
    }

    /// Validate that a blueprint kv collection payloads match the blueprint's definition
    pub fn validate_blueprint_kv_collection(
        &mut self,
        target: &BlueprintTypeTarget,
        collection_index: CollectionIndex,
        payloads: &[(&Vec<u8>, &Vec<u8>)],
    ) -> Result<PartitionDescription, RuntimeError> {
        let blueprint_interface =
            self.get_blueprint_default_interface(target.blueprint_info.blueprint_id.clone())?;

        let partition_description = blueprint_interface
            .state
            .collections
            .get(collection_index as usize)
            .ok_or_else(|| {
                RuntimeError::SystemError(SystemError::TypeCheckError(
                    TypeCheckError::InvalidCollectionIndex(
                        Box::new(target.blueprint_info.clone()),
                        collection_index,
                    ),
                ))
            })?
            .0;

        for (key, value) in payloads {
            self.validate_blueprint_payload(
                &target,
                BlueprintPayloadIdentifier::KeyValueEntry(collection_index, KeyOrValue::Key),
                key,
            )?;

            self.validate_blueprint_payload(
                &target,
                BlueprintPayloadIdentifier::KeyValueEntry(collection_index, KeyOrValue::Value),
                value,
            )?;
        }

        Ok(partition_description)
    }

    /// Validate that a key value payload matches the key value store's definition of that payload
    pub fn validate_kv_store_payload(
        &mut self,
        target: &KVStoreValidationTarget,
        payload_identifier: KeyOrValue,
        payload: &[u8],
    ) -> Result<(), RuntimeError> {
        let type_substition_ref = match payload_identifier {
            KeyOrValue::Key => target.kv_store_type.key_generic_substitutions,
            KeyOrValue::Value => target.kv_store_type.value_generic_substitutions,
        };

        let allow_ownership = match payload_identifier {
            KeyOrValue::Key => false,
            KeyOrValue::Value => target.kv_store_type.allow_ownership,
        };

        match type_substition_ref {
            GenericSubstitution::Local(type_id) => {
                let schema = self.get_schema(&target.meta, &type_id.0)?;

                self.validate_payload(
                    payload,
                    &schema,
                    type_id.1,
                    SchemaOrigin::KeyValueStore,
                    allow_ownership,
                    false,
                )
                .map_err(|err| {
                    RuntimeError::SystemError(SystemError::TypeCheckError(
                        TypeCheckError::KeyValueStorePayloadValidationError(
                            payload_identifier,
                            err.error_message(&schema),
                        ),
                    ))
                })?;
            }
        }

        Ok(())
    }

    fn validate_payload<'s>(
        &mut self,
        payload: &[u8],
        schema: &'s ScryptoSchema,
        type_index: LocalTypeIndex,
        schema_origin: SchemaOrigin,
        allow_ownership: bool,
        allow_non_global_ref: bool,
    ) -> Result<(), LocatedValidationError<'s, ScryptoCustomExtension>> {
        let validation_context: Box<dyn ValidationContext> =
            Box::new(SystemServiceTypeInfoLookup::new(
                self,
                schema_origin,
                allow_ownership,
                allow_non_global_ref,
            ));
        validate_payload_against_schema::<ScryptoCustomExtension, _>(
            payload,
            schema,
            type_index,
            &validation_context,
        )
    }

    fn get_schema(
        &mut self,
        node_id: &NodeId,
        schema_hash: &SchemaHash,
    ) -> Result<ScryptoSchema, RuntimeError> {
        let def = self
            .api
            .kernel_get_system_state()
            .system
            .schema_cache
            .get(schema_hash);
        if let Some(schema) = def {
            return Ok(schema.clone());
        }

        let handle = self.api.kernel_open_substate_with_default(
            node_id,
            SCHEMAS_PARTITION,
            &SubstateKey::Map(scrypto_encode(schema_hash).unwrap()),
            LockFlags::read_only(),
            Some(|| {
                let kv_entry = KeyValueEntrySubstate::<()>::default();
                IndexedScryptoValue::from_typed(&kv_entry)
            }),
            SystemLockData::default(),
        )?;

        let substate: KeyValueEntrySubstate<ScryptoSchema> =
            self.api.kernel_read_substate(handle)?.as_typed().unwrap();
        self.api.kernel_close_substate(handle)?;

        let schema = substate.value.unwrap();

        self.api
            .kernel_get_system_state()
            .system
            .schema_cache
            .insert(schema_hash.clone(), schema.clone());

        Ok(schema)
    }
}
