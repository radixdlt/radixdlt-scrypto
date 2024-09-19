use super::payload_validation::*;
use crate::errors::{RuntimeError, SystemError};
use crate::internal_prelude::*;
use crate::system::system::SystemService;
use crate::system::system_callback::*;
use crate::system::system_substates::{FieldSubstate, KeyValueEntrySubstate, LockStatus};
use crate::track::interface::NodeSubstates;
use radix_blueprint_schema_init::KeyValueStoreGenericSubstitutions;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::{CollectionIndex, FieldValue, KVEntry};
use radix_engine_interface::blueprints::package::*;
use sbor::rust::vec::Vec;

/// Metadata for schema validation to help with location of certain schemas
/// since location of schemas are somewhat scattered
#[derive(Debug, Clone)]
pub enum SchemaValidationMeta {
    ExistingObject {
        additional_schemas: NodeId,
    },
    NewObject {
        additional_schemas: NonIterMap<SchemaHash, VersionedScryptoSchema>,
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
pub struct KVStoreTypeTarget {
    pub kv_store_type: KeyValueStoreGenericSubstitutions,
    pub meta: NodeId,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum TypeCheckError {
    InvalidNumberOfGenericArgs { expected: usize, actual: usize },
    InvalidLocalTypeId(LocalTypeId),
    InvalidBlueprintTypeIdentifier(BlueprintTypeIdentifier),
    InvalidCollectionIndex(Box<BlueprintInfo>, CollectionIndex),
    BlueprintPayloadDoesNotExist(Box<BlueprintInfo>, BlueprintPayloadIdentifier),
    BlueprintPayloadValidationError(Box<BlueprintInfo>, BlueprintPayloadIdentifier, String),
    KeyValueStorePayloadValidationError(KeyOrValue, String),
    InstanceSchemaNotFound,
    MissingSchema,
}

impl<'a, Y: SystemBasedKernelApi> SystemService<'a, Y> {
    /// Validate that the type substitutions match the generic definition of a given blueprint
    pub fn validate_bp_generic_args(
        &mut self,
        blueprint_interface: &BlueprintInterface,
        schemas: &IndexMap<SchemaHash, VersionedScryptoSchema>,
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
            Self::validate_generic_substitution(self, schemas, generic_substitution)?;
        }

        Ok(())
    }

    /// Validate that the type substitutions for a kv store exist in a given schema
    pub fn validate_kv_store_generic_args(
        &mut self,
        schemas: &IndexMap<SchemaHash, VersionedScryptoSchema>,
        key: &GenericSubstitution,
        value: &GenericSubstitution,
    ) -> Result<(), TypeCheckError> {
        Self::validate_generic_substitution(self, schemas, key)?;
        Self::validate_generic_substitution(self, schemas, value)?;

        Ok(())
    }

    fn validate_generic_substitution(
        &mut self,
        schemas: &IndexMap<SchemaHash, VersionedScryptoSchema>,
        substitution: &GenericSubstitution,
    ) -> Result<(), TypeCheckError> {
        match substitution {
            GenericSubstitution::Local(type_id) => {
                let schema = schemas
                    .get(&type_id.0)
                    .ok_or_else(|| TypeCheckError::MissingSchema)?;

                if schema.v1().resolve_type_kind(type_id.1).is_none() {
                    Err(TypeCheckError::InvalidLocalTypeId(type_id.1))
                } else {
                    Ok(())
                }
            }
            GenericSubstitution::Remote(type_id) => self
                .get_blueprint_type_schema(type_id)
                .map(|_| ())
                .map_err(|_| TypeCheckError::InvalidBlueprintTypeIdentifier(type_id.clone())),
        }
    }

    pub fn get_payload_schema(
        &mut self,
        target: &BlueprintTypeTarget,
        payload_identifier: &BlueprintPayloadIdentifier,
    ) -> Result<
        (
            Rc<VersionedScryptoSchema>,
            LocalTypeId,
            bool,
            bool,
            SchemaOrigin,
        ),
        RuntimeError,
    > {
        let blueprint_definition =
            self.get_blueprint_default_definition(target.blueprint_info.blueprint_id.clone())?;

        let (payload_def, allow_ownership, allow_non_global_ref) = blueprint_definition
            .interface
            .get_payload_def(payload_identifier)
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
                            SchemaValidationMeta::NewObject { additional_schemas } => Rc::new(
                                additional_schemas
                                    .get(&type_id.0)
                                    .ok_or_else(|| {
                                        RuntimeError::SystemError(SystemError::TypeCheckError(
                                            TypeCheckError::InstanceSchemaNotFound,
                                        ))
                                    })?
                                    .clone(),
                            ),
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
                    GenericSubstitution::Remote(type_id) => {
                        let (schema, scoped_type_id) = self.get_blueprint_type_schema(&type_id)?;

                        (
                            schema,
                            scoped_type_id.1,
                            SchemaOrigin::Blueprint(BlueprintId::new(
                                &type_id.package_address,
                                type_id.blueprint_name.clone(),
                            )),
                        )
                    }
                }
            }
        };

        Ok((
            schema,
            index,
            allow_ownership,
            allow_non_global_ref,
            schema_origin,
        ))
    }

    /// Validate that a blueprint payload matches the blueprint's definition of that payload
    pub fn validate_blueprint_payload(
        &mut self,
        target: &BlueprintTypeTarget,
        payload_identifier: BlueprintPayloadIdentifier,
        payload: &[u8],
    ) -> Result<(), RuntimeError> {
        let (schema, index, allow_ownership, allow_non_global_ref, schema_origin) =
            self.get_payload_schema(target, &payload_identifier)?;

        self.validate_payload(
            payload,
            &schema,
            index,
            schema_origin,
            allow_ownership,
            allow_non_global_ref,
            BLUEPRINT_PAYLOAD_MAX_DEPTH,
        )
        .map_err(|err| {
            RuntimeError::SystemError(SystemError::TypeCheckError(
                TypeCheckError::BlueprintPayloadValidationError(
                    Box::new(target.blueprint_info.clone()),
                    payload_identifier,
                    err.error_message(schema.v1()),
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
        let blueprint_definition =
            self.get_blueprint_default_definition(target.blueprint_info.blueprint_id.clone())?;

        let partition_description = blueprint_definition
            .interface
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
        target: &KVStoreTypeTarget,
        payload_identifier: KeyOrValue,
        payload: &[u8],
    ) -> Result<(), RuntimeError> {
        let type_substitution = match payload_identifier {
            KeyOrValue::Key => target.kv_store_type.key_generic_substitution.clone(),
            KeyOrValue::Value => target.kv_store_type.value_generic_substitution.clone(),
        };

        let allow_ownership = match payload_identifier {
            KeyOrValue::Key => false,
            KeyOrValue::Value => target.kv_store_type.allow_ownership,
        };

        let (schema, local_type_id) = match type_substitution {
            GenericSubstitution::Local(type_id) => {
                (self.get_schema(&target.meta, &type_id.0)?, type_id.1)
            }
            GenericSubstitution::Remote(type_id) => self
                .get_blueprint_type_schema(&type_id)
                .map(|x| (x.0, x.1 .1))?,
        };

        self.validate_payload(
            payload,
            &schema,
            local_type_id,
            SchemaOrigin::KeyValueStore,
            allow_ownership,
            false,
            KEY_VALUE_STORE_PAYLOAD_MAX_DEPTH,
        )
        .map_err(|err| {
            RuntimeError::SystemError(SystemError::TypeCheckError(
                TypeCheckError::KeyValueStorePayloadValidationError(
                    payload_identifier,
                    err.error_message(schema.v1()),
                ),
            ))
        })?;

        Ok(())
    }

    fn validate_payload<'s>(
        &mut self,
        payload: &[u8],
        schema: &'s VersionedScryptoSchema,
        type_id: LocalTypeId,
        schema_origin: SchemaOrigin,
        allow_ownership: bool,
        allow_non_global_ref: bool,
        depth_limit: usize,
    ) -> Result<(), LocatedValidationError<'s, ScryptoCustomExtension>> {
        let validation_context: Box<dyn ValidationContext<Error = RuntimeError>> =
            Box::new(SystemServiceTypeInfoLookup::new(
                self,
                schema_origin,
                allow_ownership,
                allow_non_global_ref,
            ));
        validate_payload_against_schema::<ScryptoCustomExtension, _>(
            payload,
            schema.v1(),
            type_id,
            &validation_context,
            depth_limit,
        )
    }

    fn get_schema(
        &mut self,
        node_id: &NodeId,
        schema_hash: &SchemaHash,
    ) -> Result<Rc<VersionedScryptoSchema>, RuntimeError> {
        let def = self.system().schema_cache.get(schema_hash);
        if let Some(schema) = def {
            return Ok(schema.clone());
        }

        let handle = self.api().kernel_open_substate_with_default(
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

        let substate: KeyValueEntrySubstate<VersionedScryptoSchema> =
            self.api().kernel_read_substate(handle)?.as_typed().unwrap();
        self.api().kernel_close_substate(handle)?;

        let schema = Rc::new(substate.into_value().unwrap());

        self.system()
            .schema_cache
            .insert(schema_hash.clone(), schema.clone());

        Ok(schema)
    }

    pub fn get_blueprint_type_schema(
        &mut self,
        type_id: &BlueprintTypeIdentifier,
    ) -> Result<(Rc<VersionedScryptoSchema>, ScopedTypeId), RuntimeError> {
        let BlueprintTypeIdentifier {
            package_address,
            blueprint_name,
            type_name,
        } = type_id.clone();
        let blueprint_definition = self.get_blueprint_default_definition(BlueprintId {
            package_address,
            blueprint_name,
        })?;
        let scoped_type_id = blueprint_definition.interface.types.get(&type_name).ok_or(
            RuntimeError::SystemError(SystemError::BlueprintTypeNotFound(type_name.clone())),
        )?;
        Ok((
            self.get_schema(package_address.as_node_id(), &scoped_type_id.0)?,
            scoped_type_id.clone(),
        ))
    }
}

pub struct SystemMapper;

impl SystemMapper {
    pub fn system_struct_to_node_substates(
        schema: &IndexedStateSchema,
        system_struct: (
            IndexMap<u8, FieldValue>,
            IndexMap<u8, IndexMap<Vec<u8>, KVEntry>>,
        ),
        base_partition_num: PartitionNumber,
    ) -> NodeSubstates {
        let mut partitions: NodeSubstates = BTreeMap::new();

        if !system_struct.0.is_empty() {
            let partition_description = schema.fields_partition().unwrap();
            let partition_num = match partition_description {
                PartitionDescription::Physical(partition_num) => partition_num,
                PartitionDescription::Logical(offset) => {
                    base_partition_num.at_offset(offset).unwrap()
                }
            };

            let mut field_partition = BTreeMap::new();

            for (index, field) in system_struct.0.into_iter() {
                let (_, field_schema) = schema.field(index).unwrap();
                match field_schema.transience {
                    FieldTransience::TransientStatic { .. } => continue,
                    FieldTransience::NotTransient => {}
                }

                let value: ScryptoRawValue =
                    scrypto_decode(&field.value).expect("Checked by payload-schema validation");

                let lock_status = if field.locked {
                    LockStatus::Locked
                } else {
                    LockStatus::Unlocked
                };

                let substate = FieldSubstate::new_field(value, lock_status);

                let value = IndexedScryptoValue::from_typed(&substate);
                field_partition.insert(SubstateKey::Field(index), value);
            }

            partitions.insert(partition_num, field_partition);
        }

        for (collection_index, substates) in system_struct.1 {
            let (partition_description, _) = schema.get_partition(collection_index).unwrap();
            let partition_num = match partition_description {
                PartitionDescription::Physical(partition_num) => partition_num,
                PartitionDescription::Logical(offset) => {
                    base_partition_num.at_offset(offset).unwrap()
                }
            };

            let mut partition = BTreeMap::new();

            for (key, kv_entry) in substates {
                let kv_entry = if let Some(value) = kv_entry.value {
                    let value: ScryptoRawValue = scrypto_decode(&value).unwrap();
                    let kv_entry = if kv_entry.locked {
                        KeyValueEntrySubstate::locked_entry(value)
                    } else {
                        KeyValueEntrySubstate::unlocked_entry(value)
                    };
                    kv_entry
                } else {
                    if kv_entry.locked {
                        KeyValueEntrySubstate::locked_empty_entry()
                    } else {
                        continue;
                    }
                };

                let value = IndexedScryptoValue::from_typed(&kv_entry);
                partition.insert(SubstateKey::Map(key), value);
            }

            partitions.insert(partition_num, partition);
        }

        partitions
    }
}
