use super::payload_validation::*;
use crate::errors::{PayloadValidationAgainstSchemaError, RuntimeError, SystemError};
use crate::kernel::actor::Actor;
use crate::system::system::{KeyValueEntrySubstate, SystemService};
use crate::system::system_callback::{SystemConfig, SystemLockData};
use crate::system::system_callback_api::SystemCallbackObject;
use crate::types::*;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::schema::KeyValueStoreTypeSubstitutions;
use sbor::rust::vec::Vec;
use crate::kernel::kernel_api::KernelApi;

#[derive(Debug, Clone)]
pub struct KVStoreValidationTarget {
    pub kv_store_type: KeyValueStoreTypeSubstitutions,
    pub meta: NodeId,
}

#[derive(Debug, Clone)]
pub enum SchemaValidationMeta {
    ExistingObject {
        additional_schemas: NodeId,
    },
    NewObject {
        additional_schemas: NonIterMap<Hash, ScryptoSchema>,
    },
    Blueprint,
}

#[derive(Debug, Clone)]
pub struct BlueprintTypeTarget {
    pub blueprint_info: BlueprintInfo,
    pub meta: SchemaValidationMeta,
}

impl<'a, Y, V> SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    pub fn validate_bp_generic_args(
        &mut self,
        blueprint_id: &BlueprintId,
        schemas: &IndexMap<Hash, ScryptoSchema>,
        type_substitution_refs: &Vec<TypeSubstitutionRef>,
    ) -> Result<(), RuntimeError> {
        let generics = self
            .get_blueprint_default_interface(blueprint_id.clone())?
            .generics;

        if !generics.len().eq(&type_substitution_refs.len()) {
            return Err(RuntimeError::SystemError(SystemError::InvalidGenericArgs));
        }

        for type_substitution_ref in type_substitution_refs {
            match type_substitution_ref {
                TypeSubstitutionRef::Local(type_id) => {
                    let _schema = schemas.get(&type_id.0).ok_or_else(|| {
                        RuntimeError::SystemError(SystemError::InvalidGenericArgs)
                    })?;
                }
            }
        }

        Ok(())
    }

    pub fn validate_kv_store_generic_args(
        &mut self,
        schemas: &IndexMap<Hash, ScryptoSchema>,
        key: &TypeSubstitutionRef,
        value: &TypeSubstitutionRef,
    ) -> Result<(), RuntimeError> {
        match key {
            TypeSubstitutionRef::Local(type_id) => {
                let _schema = schemas
                    .get(&type_id.0)
                    .ok_or_else(|| RuntimeError::SystemError(SystemError::InvalidGenericArgs))?;
            }
        }

        match value {
            TypeSubstitutionRef::Local(type_id) => {
                let _schema = schemas
                    .get(&type_id.0)
                    .ok_or_else(|| RuntimeError::SystemError(SystemError::InvalidGenericArgs))?;
            }
        }

        Ok(())
    }

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
                RuntimeError::SystemError(SystemError::PayloadValidationAgainstSchemaError(
                    PayloadValidationAgainstSchemaError::PayloadDoesNotExist(
                        Box::new(target.blueprint_info.clone()),
                        payload_identifier,
                    ),
                ))
            })?;

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
                let type_substitution_ref = target
                    .blueprint_info
                    .type_substitutions_refs
                    .get(instance_index as usize)
                    .ok_or_else(|| {
                        RuntimeError::SystemError(SystemError::PayloadValidationAgainstSchemaError(
                            PayloadValidationAgainstSchemaError::InstanceSchemaDoesNotExist,
                        ))
                    })?;

                match type_substitution_ref {
                    TypeSubstitutionRef::Local(type_id) => {
                        let schema = match &target.meta {
                            SchemaValidationMeta::ExistingObject { additional_schemas } => {
                                self.get_schema(additional_schemas, &type_id.0)?
                            }
                            SchemaValidationMeta::NewObject { additional_schemas } => {
                                additional_schemas.get(&type_id.0).ok_or_else(|| {
                                    RuntimeError::SystemError(SystemError::PayloadValidationAgainstSchemaError(
                                        PayloadValidationAgainstSchemaError::InstanceSchemaDoesNotExist,
                                    ))
                                })?.clone() // TODO: Remove clone
                            }
                            SchemaValidationMeta::Blueprint => {
                                return Err(RuntimeError::SystemError(
                                    SystemError::PayloadValidationAgainstSchemaError(
                                        PayloadValidationAgainstSchemaError::InstanceSchemaDoesNotExist,
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
            RuntimeError::SystemError(SystemError::PayloadValidationAgainstSchemaError(
                PayloadValidationAgainstSchemaError::PayloadValidationError(
                    err.error_message(&schema),
                ),
            ))
        })?;

        Ok(())
    }

    pub fn validate_kv_store_payload(
        &mut self,
        target: &KVStoreValidationTarget,
        payload_identifier: KeyOrValue,
        payload: &[u8],
    ) -> Result<(), RuntimeError> {
        let type_substition_ref = match payload_identifier {
            KeyOrValue::Key => target.kv_store_type.key_type_substitution,
            KeyOrValue::Value => target.kv_store_type.value_type_substitution,
        };

        let allow_ownership = match payload_identifier {
            KeyOrValue::Key => false,
            KeyOrValue::Value => target.kv_store_type.can_own,
        };

        match type_substition_ref {
            TypeSubstitutionRef::Local(type_id) => {
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
                    RuntimeError::SystemError(SystemError::KeyValueStorePayloadValidationError(
                        payload_identifier,
                        err.error_message(&schema),
                    ))
                })?;
            }
        }

        Ok(())
    }

    pub fn get_actor_type_target(&mut self) -> Result<BlueprintTypeTarget, RuntimeError> {
        let actor = self.current_actor();
        match actor {
            Actor::Root => Err(RuntimeError::SystemError(SystemError::RootHasNoType)),
            Actor::BlueprintHook(actor) => Ok(BlueprintTypeTarget {
                blueprint_info: BlueprintInfo {
                    blueprint_id: actor.blueprint_id.clone(),
                    outer_obj_info: OuterObjectInfo::None,
                    features: btreeset!(),
                    type_substitutions_refs: vec![],
                },
                meta: SchemaValidationMeta::Blueprint,
            }),
            Actor::Function(actor) => Ok(BlueprintTypeTarget {
                blueprint_info: BlueprintInfo {
                    blueprint_id: actor.blueprint_id.clone(),
                    outer_obj_info: OuterObjectInfo::None,
                    features: btreeset!(),
                    type_substitutions_refs: vec![],
                },
                meta: SchemaValidationMeta::Blueprint,
            }),
            Actor::Method(actor) => {
                let blueprint_info = self.get_blueprint_info(&actor.node_id, actor.module_id)?;
                Ok(BlueprintTypeTarget {
                    blueprint_info,
                    meta: SchemaValidationMeta::ExistingObject {
                        additional_schemas: actor.node_id,
                    },
                })
            }
        }
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
        schema_hash: &Hash,
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
