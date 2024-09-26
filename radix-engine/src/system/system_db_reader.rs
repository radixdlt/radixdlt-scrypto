use crate::internal_prelude::*;
use radix_engine_interface::api::{AttachedModuleId, CollectionIndex, ModuleId};
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::types::*;
use radix_substate_store_interface::interface::*;
use sbor::{validate_payload_against_schema, LocalTypeId, LocatedValidationError};

use crate::blueprints::package::PackageBlueprintVersionDefinitionEntrySubstate;
use crate::system::payload_validation::{SchemaOrigin, TypeInfoForValidation, ValidationContext};
use crate::system::system_substates::FieldSubstate;
use crate::system::system_substates::KeyValueEntrySubstate;
use crate::system::system_substates::LockStatus;
use crate::system::system_type_checker::{
    BlueprintTypeTarget, KVStoreTypeTarget, SchemaValidationMeta,
};
use crate::system::type_info::TypeInfoSubstate;
use crate::transaction::{
    ObjectInstanceTypeReference, ObjectSubstateTypeReference, PackageTypeReference,
};
use radix_blueprint_schema_init::BlueprintCollectionSchema;

#[derive(Clone, Debug)]
pub enum SystemPartitionDescription {
    TypeInfo,
    Schema,
    Module(ModuleId, PartitionOffset),
}

#[derive(Clone, Debug)]
pub enum ObjectPartitionDescriptor {
    Fields,
    KeyValueCollection(u8),
    IndexCollection(u8),
    SortedIndexCollection(u8),
}

#[derive(Clone, Debug)]
pub enum SystemPartitionDescriptor {
    BootLoader,
    ProtocolUpdateStatus,
    TypeInfo,
    Schema,
    KeyValueStore,
    Object(ModuleId, ObjectPartitionDescriptor),
}

pub struct ResolvedPayloadSchema {
    pub schema: Rc<VersionedScryptoSchema>,
    pub type_id: LocalTypeId,
    pub allow_ownership: bool,
    pub allow_non_global_refs: bool,
    pub schema_origin: SchemaOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectCollectionKey<'a, K: ScryptoEncode> {
    KeyValue(u8, &'a K),
    Index(u8, &'a K),
    SortedIndex(u8, u16, &'a K),
}

impl<'a, K: ScryptoEncode> ObjectCollectionKey<'a, K> {
    fn collection_index(&self) -> u8 {
        match self {
            ObjectCollectionKey::KeyValue(index, ..)
            | ObjectCollectionKey::Index(index, ..)
            | ObjectCollectionKey::SortedIndex(index, ..) => *index,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemReaderError {
    FieldDoesNotExist,
    CollectionDoesNotExist,
    NodeIdDoesNotExist,
    PayloadDoesNotExist,
    BlueprintDoesNotExist,
    ModuleDoesNotExist,
    NotAKeyValueStore,
    NotAnObject,
    SchemaDoesNotExist,
    TargetNotSupported,
    BlueprintTypeNotFound(String),
}

/// A System Layer (Layer 2) abstraction over an underlying substate database
pub struct SystemDatabaseReader<'a, S: SubstateDatabase + ?Sized> {
    substate_db: &'a S,
    state_updates: Option<&'a StateUpdates>,

    blueprint_cache: RefCell<NonIterMap<CanonicalBlueprintId, Rc<BlueprintDefinition>>>,
    schema_cache: RefCell<NonIterMap<SchemaHash, Rc<VersionedScryptoSchema>>>,
}

impl<'a, S: SubstateDatabase + ?Sized> SystemDatabaseReader<'a, S> {
    pub fn new_with_overlay(substate_db: &'a S, state_updates: &'a StateUpdates) -> Self {
        Self {
            substate_db,
            state_updates: Some(state_updates),
            blueprint_cache: RefCell::new(NonIterMap::new()),
            schema_cache: RefCell::new(NonIterMap::new()),
        }
    }

    pub fn new(substate_db: &'a S) -> Self {
        Self {
            substate_db,
            state_updates: None,
            blueprint_cache: RefCell::new(NonIterMap::new()),
            schema_cache: RefCell::new(NonIterMap::new()),
        }
    }

    pub fn get_type_info(&self, node_id: &NodeId) -> Result<TypeInfoSubstate, SystemReaderError> {
        self.fetch_substate::<TypeInfoSubstate>(
            node_id,
            TYPE_INFO_FIELD_PARTITION,
            &TypeInfoField::TypeInfo.into(),
        )
        .ok_or_else(|| SystemReaderError::NodeIdDoesNotExist)
    }

    pub fn get_package_definition(
        &self,
        package_address: PackageAddress,
    ) -> BTreeMap<BlueprintVersionKey, BlueprintDefinition> {
        let entries = self
            .substate_db
            .list_map_values::<PackageBlueprintVersionDefinitionEntrySubstate>(
                package_address.as_node_id(),
                MAIN_BASE_PARTITION
                    .at_offset(PACKAGE_BLUEPRINTS_PARTITION_OFFSET)
                    .unwrap(),
                None::<SubstateKey>,
            );

        let mut blueprints = BTreeMap::new();
        for (key, blueprint_definition) in entries {
            let bp_version_key: BlueprintVersionKey = scrypto_decode(&key).unwrap();

            blueprints.insert(
                bp_version_key,
                blueprint_definition
                    .into_value()
                    .unwrap()
                    .fully_update_and_into_latest_version(),
            );
        }

        blueprints
    }

    pub fn read_object_field(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        field_index: u8,
    ) -> Result<IndexedScryptoValue, SystemReaderError> {
        self.read_object_field_advanced(node_id, module_id, field_index)
            .map(|x| x.0)
    }

    pub fn read_object_field_advanced(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        field_index: u8,
    ) -> Result<(IndexedScryptoValue, PartitionNumber), SystemReaderError> {
        let blueprint_id = self.get_blueprint_id(node_id, module_id)?;
        let definition = self.get_blueprint_definition(&blueprint_id)?;
        let partition_description = &definition
            .interface
            .state
            .fields
            .as_ref()
            .ok_or_else(|| SystemReaderError::FieldDoesNotExist)?
            .0;
        let partition_number = match partition_description {
            PartitionDescription::Logical(offset) => {
                let base_partition = match module_id {
                    ModuleId::Main => MAIN_BASE_PARTITION,
                    ModuleId::Metadata => METADATA_BASE_PARTITION,
                    ModuleId::Royalty => ROYALTY_BASE_PARTITION,
                    ModuleId::RoleAssignment => ROLE_ASSIGNMENT_BASE_PARTITION,
                };
                base_partition.at_offset(*offset).unwrap()
            }
            PartitionDescription::Physical(partition_number) => *partition_number,
        };

        let substate: FieldSubstate<ScryptoValue> = self
            .substate_db
            .get_substate(node_id, partition_number, SubstateKey::Field(field_index))
            .ok_or_else(|| SystemReaderError::FieldDoesNotExist)?;

        Ok((
            IndexedScryptoValue::from_scrypto_value(substate.into_payload()),
            partition_number,
        ))
    }

    pub fn read_typed_kv_entry<K: ScryptoEncode, V: ScryptoDecode>(
        &self,
        node_id: &NodeId,
        key: &K,
    ) -> Option<V> {
        self.substate_db
            .get_substate::<KeyValueEntrySubstate<V>>(
                node_id,
                MAIN_BASE_PARTITION,
                SubstateKey::Map(scrypto_encode(key).unwrap()),
            )
            .and_then(|v| v.into_value())
    }

    pub fn read_typed_object_field<V: ScryptoDecode>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        field_index: u8,
    ) -> Result<V, SystemReaderError> {
        let blueprint_id = self.get_blueprint_id(node_id, module_id)?;
        let definition = self.get_blueprint_definition(&blueprint_id)?;
        let partition_description = &definition
            .interface
            .state
            .fields
            .as_ref()
            .ok_or_else(|| SystemReaderError::FieldDoesNotExist)?
            .0;
        let partition_number = match partition_description {
            PartitionDescription::Logical(offset) => {
                let base_partition = match module_id {
                    ModuleId::Main => MAIN_BASE_PARTITION,
                    ModuleId::Metadata => METADATA_BASE_PARTITION,
                    ModuleId::Royalty => ROYALTY_BASE_PARTITION,
                    ModuleId::RoleAssignment => ROLE_ASSIGNMENT_BASE_PARTITION,
                };
                base_partition.at_offset(*offset).unwrap()
            }
            PartitionDescription::Physical(partition_number) => *partition_number,
        };

        let substate: FieldSubstate<V> = self
            .substate_db
            .get_substate(node_id, partition_number, SubstateKey::Field(field_index))
            .ok_or_else(|| SystemReaderError::FieldDoesNotExist)?;

        Ok(substate.into_payload())
    }

    pub fn get_partition_of_collection(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        collection_index: CollectionIndex,
    ) -> Result<PartitionNumber, SystemReaderError> {
        let blueprint_id = self.get_blueprint_id(node_id, module_id)?;
        let definition = self.get_blueprint_definition(&blueprint_id)?;

        let (partition_description, ..) = definition
            .interface
            .state
            .collections
            .get(collection_index as usize)
            .ok_or_else(|| SystemReaderError::CollectionDoesNotExist)?;

        let partition_number = match partition_description {
            PartitionDescription::Logical(offset) => {
                module_id.base_partition_num().at_offset(*offset).unwrap()
            }
            PartitionDescription::Physical(partition_number) => *partition_number,
        };

        Ok(partition_number)
    }

    pub fn read_object_collection_entry<K: ScryptoEncode, V: ScryptoDecode>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        collection_key: ObjectCollectionKey<K>,
    ) -> Result<Option<V>, SystemReaderError> {
        let partition_number = self.get_partition_of_collection(
            node_id,
            module_id,
            collection_key.collection_index(),
        )?;

        let entry = match collection_key {
            ObjectCollectionKey::KeyValue(_, key) => self
                .substate_db
                .get_substate::<KeyValueEntrySubstate<V>>(
                    node_id,
                    partition_number,
                    SubstateKey::Map(scrypto_encode(key).unwrap()),
                )
                .and_then(|value| value.into_value()),
            ObjectCollectionKey::Index(_, key) => self
                .substate_db
                .get_substate::<IndexEntrySubstate<V>>(
                    node_id,
                    partition_number,
                    SubstateKey::Map(scrypto_encode(key).unwrap()),
                )
                .map(|value| value.into_value()),
            ObjectCollectionKey::SortedIndex(_, sort, key) => self
                .substate_db
                .get_substate::<SortedIndexEntrySubstate<V>>(
                    node_id,
                    partition_number,
                    SubstateKey::Sorted((sort.to_be_bytes(), scrypto_encode(key).unwrap())),
                )
                .map(|value| value.into_value()),
        };

        Ok(entry)
    }

    pub fn key_value_store_iter(
        &self,
        node_id: &NodeId,
        from_key: Option<&MapKey>,
    ) -> Result<Box<dyn Iterator<Item = (MapKey, Vec<u8>)> + '_>, SystemReaderError> {
        if self.state_updates.is_some() {
            panic!("key_value_store_iter with overlay not supported.");
        }

        match self.get_type_info(node_id)? {
            TypeInfoSubstate::KeyValueStore(..) => {}
            _ => return Err(SystemReaderError::NotAKeyValueStore),
        }

        let iterable = self
            .substate_db
            .list_map_values::<KeyValueEntrySubstate<ScryptoRawValue>>(
                node_id,
                MAIN_BASE_PARTITION,
                from_key,
            )
            .filter_map(move |(map_key, substate)| {
                let value = substate.into_value()?;
                let value = scrypto_encode(&value).unwrap();

                Some((map_key, value))
            });

        Ok(Box::new(iterable))
    }

    pub fn collection_iter(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        collection_index: CollectionIndex,
    ) -> Result<Box<dyn Iterator<Item = (SubstateKey, Vec<u8>)> + '_>, SystemReaderError> {
        self.collection_iter_advanced(node_id, module_id, collection_index, None)
            .map(|x| x.0)
    }

    pub fn collection_iter_advanced<'s, 'x>(
        &'s self,
        node_id: &'x NodeId,
        module_id: ModuleId,
        collection_index: CollectionIndex,
        from_substate_key: Option<&'x SubstateKey>,
    ) -> Result<
        (
            Box<dyn Iterator<Item = (SubstateKey, Vec<u8>)> + 's>,
            PartitionNumber,
        ),
        SystemReaderError,
    > {
        if self.state_updates.is_some() {
            panic!("collection_iter_advanced with overlay not supported.");
        }

        let blueprint_id = self.get_blueprint_id(node_id, module_id)?;
        let definition = self.get_blueprint_definition(&blueprint_id)?;

        let (partition_description, schema) = definition
            .interface
            .state
            .collections
            .get(collection_index as usize)
            .ok_or_else(|| SystemReaderError::CollectionDoesNotExist)?
            .clone();

        let partition_number = match partition_description {
            PartitionDescription::Physical(partition_num) => partition_num,
            PartitionDescription::Logical(offset) => {
                module_id.base_partition_num().at_offset(offset).unwrap()
            }
        };

        let iterable: Box<dyn Iterator<Item = (SubstateKey, Vec<u8>)> + 's> = match schema {
            BlueprintCollectionSchema::KeyValueStore(..) => {
                let iterable = self
                    .substate_db
                    .list_map_values::<KeyValueEntrySubstate<ScryptoRawValue>>(
                        node_id,
                        partition_number,
                        from_substate_key,
                    )
                    .filter_map(|(map_key, substate)| {
                        Some((
                            SubstateKey::Map(map_key),
                            scrypto_encode(&substate.into_value()?).unwrap(),
                        ))
                    });
                Box::new(iterable)
            }
            BlueprintCollectionSchema::Index(..) => {
                let iterable = self
                    .substate_db
                    .list_map_values::<IndexEntrySubstate<ScryptoRawValue>>(
                        node_id,
                        partition_number,
                        from_substate_key,
                    )
                    .map(|(map_key, substate)| {
                        (
                            SubstateKey::Map(map_key),
                            scrypto_encode(&substate.into_value()).unwrap(),
                        )
                    });
                Box::new(iterable)
            }
            BlueprintCollectionSchema::SortedIndex(..) => {
                let iterable = self
                    .substate_db
                    .list_sorted_values::<SortedIndexEntrySubstate<ScryptoRawValue>>(
                        node_id,
                        partition_number,
                        from_substate_key,
                    )
                    .map(|(key, substate)| {
                        (
                            SubstateKey::Sorted(key),
                            scrypto_encode(&substate.into_value()).unwrap(),
                        )
                    });
                Box::new(iterable)
            }
        };

        Ok((iterable, partition_number))
    }

    pub fn get_object_info<A: Into<NodeId>>(
        &self,
        node_id: A,
    ) -> Result<ObjectInfo, SystemReaderError> {
        let type_info = self
            .fetch_substate::<TypeInfoSubstate>(
                &node_id.into(),
                TYPE_INFO_FIELD_PARTITION,
                &TypeInfoField::TypeInfo.into(),
            )
            .ok_or_else(|| SystemReaderError::NodeIdDoesNotExist)?;

        match type_info {
            TypeInfoSubstate::Object(object_info) => Ok(object_info),
            _ => Err(SystemReaderError::NotAnObject),
        }
    }

    pub fn get_blueprint_id(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Result<BlueprintId, SystemReaderError> {
        let type_info = self
            .fetch_substate::<TypeInfoSubstate>(
                node_id,
                TYPE_INFO_FIELD_PARTITION,
                &TypeInfoField::TypeInfo.into(),
            )
            .ok_or_else(|| SystemReaderError::NodeIdDoesNotExist)?;

        let object_info = match type_info {
            TypeInfoSubstate::Object(object_info) => object_info,
            _ => {
                return Err(SystemReaderError::NotAnObject);
            }
        };

        let module_id: Option<AttachedModuleId> = module_id.into();
        if let Some(module_id) = module_id {
            match object_info.object_type {
                ObjectType::Global { modules } => {
                    if !modules.contains_key(&module_id) {
                        return Err(SystemReaderError::ModuleDoesNotExist);
                    }
                }
                ObjectType::Owned => return Err(SystemReaderError::ModuleDoesNotExist),
            }

            Ok(module_id.static_blueprint())
        } else {
            Ok(object_info.blueprint_info.blueprint_id)
        }
    }

    pub fn get_blueprint_definition(
        &self,
        blueprint_id: &BlueprintId,
    ) -> Result<Rc<BlueprintDefinition>, SystemReaderError> {
        let canonical_key = CanonicalBlueprintId {
            address: blueprint_id.package_address,
            blueprint: blueprint_id.blueprint_name.clone(),
            version: BlueprintVersion::default(),
        };
        {
            if let Some(cache) = self.blueprint_cache.borrow().get(&canonical_key) {
                return Ok(cache.clone());
            }
        }

        let bp_version_key = BlueprintVersionKey::new_default(blueprint_id.blueprint_name.clone());
        let definition = Rc::new(
            self.fetch_substate::<PackageBlueprintVersionDefinitionEntrySubstate>(
                blueprint_id.package_address.as_node_id(),
                MAIN_BASE_PARTITION
                    .at_offset(PACKAGE_BLUEPRINTS_PARTITION_OFFSET)
                    .unwrap(),
                &SubstateKey::Map(scrypto_encode(&bp_version_key).unwrap()),
            )
            .ok_or_else(|| SystemReaderError::BlueprintDoesNotExist)?
            .into_value()
            .unwrap()
            .fully_update_and_into_latest_version(),
        );

        self.blueprint_cache
            .borrow_mut()
            .insert(canonical_key, definition.clone());

        Ok(definition)
    }

    pub fn get_kv_store_type_target(
        &self,
        node_id: &NodeId,
    ) -> Result<KVStoreTypeTarget, SystemReaderError> {
        let type_info = self
            .fetch_substate::<TypeInfoSubstate>(
                node_id,
                TYPE_INFO_FIELD_PARTITION,
                &TypeInfoField::TypeInfo.into(),
            )
            .ok_or_else(|| SystemReaderError::NodeIdDoesNotExist)?;

        let kv_store_info = match type_info {
            TypeInfoSubstate::KeyValueStore(kv_store_info) => kv_store_info,
            _ => return Err(SystemReaderError::NotAKeyValueStore),
        };

        Ok(KVStoreTypeTarget {
            kv_store_type: kv_store_info.generic_substitutions,
            meta: *node_id,
        })
    }

    pub fn get_blueprint_type_target(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Result<BlueprintTypeTarget, SystemReaderError> {
        let type_info = self
            .fetch_substate::<TypeInfoSubstate>(
                node_id,
                TYPE_INFO_FIELD_PARTITION,
                &TypeInfoField::TypeInfo.into(),
            )
            .ok_or_else(|| SystemReaderError::NodeIdDoesNotExist)?;

        let object_info = match type_info {
            TypeInfoSubstate::Object(object_info) => object_info,
            _ => return Err(SystemReaderError::NotAnObject),
        };

        let module_id: Option<AttachedModuleId> = module_id.into();
        let target = if let Some(module_id) = module_id {
            let blueprint_id = module_id.static_blueprint();
            match object_info.object_type {
                ObjectType::Global { modules } => {
                    if !modules.contains_key(&module_id) {
                        return Err(SystemReaderError::ModuleDoesNotExist);
                    }
                }
                ObjectType::Owned => return Err(SystemReaderError::ModuleDoesNotExist),
            }

            let target = BlueprintTypeTarget {
                blueprint_info: BlueprintInfo {
                    blueprint_id,
                    blueprint_version: Default::default(),
                    outer_obj_info: OuterObjectInfo::None,
                    features: Default::default(),
                    generic_substitutions: Default::default(),
                },
                meta: SchemaValidationMeta::ExistingObject {
                    additional_schemas: *node_id,
                },
            };
            target
        } else {
            BlueprintTypeTarget {
                blueprint_info: object_info.blueprint_info,
                meta: SchemaValidationMeta::ExistingObject {
                    additional_schemas: *node_id,
                },
            }
        };

        Ok(target)
    }

    pub fn get_kv_store_payload_schema(
        &self,
        target: &KVStoreTypeTarget,
        key_or_value: KeyOrValue,
    ) -> Result<ResolvedPayloadSchema, SystemReaderError> {
        let (substitution, allow_ownership, allow_non_global_refs) = match key_or_value {
            KeyOrValue::Key => (&target.kv_store_type.key_generic_substitution, false, false),
            KeyOrValue::Value => (
                &target.kv_store_type.value_generic_substitution,
                target.kv_store_type.allow_ownership,
                false,
            ),
        };

        match substitution {
            GenericSubstitution::Local(local_type_id) => {
                let schema = self.get_schema(&target.meta, &local_type_id.0)?;

                Ok(ResolvedPayloadSchema {
                    schema,
                    type_id: local_type_id.1,
                    allow_ownership,
                    allow_non_global_refs,
                    schema_origin: SchemaOrigin::KeyValueStore,
                })
            }
            GenericSubstitution::Remote(blueprint_type_id) => {
                let (schema, scoped_type_id) =
                    self.get_blueprint_type_schema(&blueprint_type_id)?;

                Ok(ResolvedPayloadSchema {
                    schema,
                    type_id: scoped_type_id.1,
                    allow_ownership,
                    allow_non_global_refs,
                    schema_origin: SchemaOrigin::KeyValueStore,
                })
            }
        }
    }

    pub fn get_blueprint_payload_schema_pointer(
        &self,
        target: &BlueprintTypeTarget,
        payload_identifier: &BlueprintPayloadIdentifier,
    ) -> Result<ObjectSubstateTypeReference, SystemReaderError> {
        let blueprint_interface = &self
            .get_blueprint_definition(&target.blueprint_info.blueprint_id)?
            .interface;

        let (payload_def, ..) = blueprint_interface
            .get_payload_def(payload_identifier)
            .ok_or_else(|| SystemReaderError::PayloadDoesNotExist)?;

        let obj_type_reference = match payload_def {
            BlueprintPayloadDef::Static(type_identifier) => {
                ObjectSubstateTypeReference::Package(PackageTypeReference {
                    full_type_id: type_identifier
                        .under_node(target.blueprint_info.blueprint_id.package_address),
                })
            }
            BlueprintPayloadDef::Generic(instance_index) => {
                let generic_substitution = target
                    .blueprint_info
                    .generic_substitutions
                    .get(instance_index as usize)
                    .expect("Missing generic");

                let entity_address = match target.meta {
                    SchemaValidationMeta::Blueprint | SchemaValidationMeta::NewObject { .. } => {
                        return Err(SystemReaderError::TargetNotSupported)
                    }
                    SchemaValidationMeta::ExistingObject { additional_schemas } => {
                        additional_schemas
                    }
                };

                match generic_substitution {
                    GenericSubstitution::Local(type_id) => {
                        ObjectSubstateTypeReference::ObjectInstance(ObjectInstanceTypeReference {
                            instance_type_id: instance_index,
                            resolved_full_type_id: type_id.under_node(entity_address),
                        })
                    }
                    GenericSubstitution::Remote(type_id) => {
                        let (_, scoped_type_id) = self.get_blueprint_type_schema(&type_id)?;
                        ObjectSubstateTypeReference::Package(PackageTypeReference {
                            full_type_id: scoped_type_id.under_node(type_id.package_address),
                        })
                    }
                }
            }
        };

        Ok(obj_type_reference)
    }

    // TODO: The logic here is currently copied from system_type_checker.rs get_payload_schema().
    // It would be nice to use the same underlying code but currently too many refactors are required
    // to make that happen.
    pub fn get_blueprint_payload_schema(
        &self,
        target: &BlueprintTypeTarget,
        payload_identifier: &BlueprintPayloadIdentifier,
    ) -> Result<ResolvedPayloadSchema, SystemReaderError> {
        let blueprint_interface = &self
            .get_blueprint_definition(&target.blueprint_info.blueprint_id)?
            .interface;

        let (payload_def, allow_ownership, allow_non_global_refs) = blueprint_interface
            .get_payload_def(payload_identifier)
            .ok_or_else(|| SystemReaderError::PayloadDoesNotExist)?;

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
                    .expect("Missing generic substitution");

                match generic_substitution {
                    GenericSubstitution::Local(type_id) => {
                        let schema = match &target.meta {
                            SchemaValidationMeta::ExistingObject { additional_schemas } => {
                                self.get_schema(additional_schemas, &type_id.0)?
                            }
                            SchemaValidationMeta::NewObject { .. }
                            | SchemaValidationMeta::Blueprint => {
                                return Err(SystemReaderError::TargetNotSupported);
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

        Ok(ResolvedPayloadSchema {
            schema,
            type_id: index,
            allow_ownership,
            allow_non_global_refs,
            schema_origin,
        })
    }

    pub fn get_schema(
        &self,
        node_id: &NodeId,
        schema_hash: &SchemaHash,
    ) -> Result<Rc<VersionedScryptoSchema>, SystemReaderError> {
        {
            if let Some(cache) = self.schema_cache.borrow().get(schema_hash) {
                return Ok(cache.clone());
            }
        }

        let schema = Rc::new(
            self.fetch_substate::<KeyValueEntrySubstate<VersionedScryptoSchema>>(
                node_id,
                SCHEMAS_PARTITION,
                &SubstateKey::Map(scrypto_encode(schema_hash).unwrap()),
            )
            .ok_or_else(|| SystemReaderError::SchemaDoesNotExist)?
            .into_value()
            .expect("Schema should exist if substate exists"),
        );

        self.schema_cache
            .borrow_mut()
            .insert(schema_hash.clone(), schema.clone());

        Ok(schema)
    }

    pub fn get_blueprint_type_schema(
        &self,
        type_id: &BlueprintTypeIdentifier,
    ) -> Result<(Rc<VersionedScryptoSchema>, ScopedTypeId), SystemReaderError> {
        let BlueprintTypeIdentifier {
            package_address,
            blueprint_name,
            type_name,
        } = type_id.clone();
        let definition = self.get_blueprint_payload_def(&BlueprintId {
            package_address,
            blueprint_name,
        })?;
        let scoped_type_id = definition
            .interface
            .types
            .get(&type_name)
            .ok_or(SystemReaderError::BlueprintTypeNotFound(type_name.clone()))?;
        Ok((
            self.get_schema(package_address.as_node_id(), &scoped_type_id.0)?,
            scoped_type_id.clone(),
        ))
    }

    pub fn get_blueprint_payload_def(
        &self,
        blueprint_id: &BlueprintId,
    ) -> Result<BlueprintDefinition, SystemReaderError> {
        let bp_version_key = BlueprintVersionKey::new_default(blueprint_id.blueprint_name.clone());
        let definition = self
            .fetch_substate::<PackageBlueprintVersionDefinitionEntrySubstate>(
                blueprint_id.package_address.as_node_id(),
                MAIN_BASE_PARTITION
                    .at_offset(PACKAGE_BLUEPRINTS_PARTITION_OFFSET)
                    .unwrap(),
                &SubstateKey::Map(scrypto_encode(&bp_version_key).unwrap()),
            )
            .ok_or_else(|| SystemReaderError::BlueprintDoesNotExist)?;

        Ok(definition
            .into_value()
            .unwrap()
            .fully_update_and_into_latest_version())
    }

    pub fn validate_payload<'b>(
        &'b self,
        payload: &[u8],
        payload_schema: &'b ResolvedPayloadSchema,
        depth_limit: usize,
    ) -> Result<(), LocatedValidationError<ScryptoCustomExtension>> {
        let validation_context: Box<dyn ValidationContext<Error = String>> =
            Box::new(ValidationPayloadCheckerContext {
                reader: self,
                schema_origin: payload_schema.schema_origin.clone(),
                allow_ownership: payload_schema.allow_ownership,
                allow_non_global_ref: payload_schema.allow_non_global_refs,
            });

        validate_payload_against_schema::<ScryptoCustomExtension, _>(
            payload,
            payload_schema.schema.v1(),
            payload_schema.type_id,
            &validation_context,
            depth_limit,
        )
    }

    pub fn fetch_substate<D: ScryptoDecode>(
        &self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        key: &SubstateKey,
    ) -> Option<D> {
        if let Some(result) =
            self.fetch_substate_from_state_updates::<D>(node_id, partition_num, key)
        {
            // If result can be determined from the state updates.
            result
        } else {
            // Otherwise, read from the substate database.
            self.fetch_substate_from_database::<D>(node_id, partition_num, key)
        }
    }

    pub fn fetch_substate_from_database<D: ScryptoDecode>(
        &self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        key: &SubstateKey,
    ) -> Option<D> {
        self.substate_db
            .get_substate::<D>(node_id, partition_num, key)
    }

    pub fn fetch_substate_from_state_updates<D: ScryptoDecode>(
        &self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Option<Option<D>> {
        if let Some(updates) = self.state_updates {
            updates
                .by_node
                .get(node_id)
                .and_then(|node_updates| match node_updates {
                    NodeStateUpdates::Delta { by_partition } => by_partition.get(&partition_num),
                })
                .and_then(|partition_updates| match partition_updates {
                    PartitionStateUpdates::Delta { by_substate } => {
                        match by_substate.get(substate_key) {
                            Some(e) => match e {
                                DatabaseUpdate::Set(value) => {
                                    Some(Some(scrypto_decode(value).unwrap()))
                                }
                                DatabaseUpdate::Delete => {
                                    // Return `Some(None)` if the substate is deleted.
                                    Some(None)
                                }
                            },
                            None => None,
                        }
                    }
                    PartitionStateUpdates::Batch(e) => match e {
                        BatchPartitionStateUpdate::Reset {
                            new_substate_values,
                        } => {
                            // Return `Some(None)` if the substate key isn't in the new value set.
                            Some(
                                new_substate_values
                                    .get(substate_key)
                                    .map(|value| scrypto_decode(value).unwrap()),
                            )
                        }
                    },
                })
        } else {
            None
        }
    }
}

// Reverse Mapping Functionality
impl<'a, S: SubstateDatabase> SystemDatabaseReader<'a, S> {
    pub fn get_partition_descriptors(
        &self,
        node_id: &NodeId,
        partition_num: &PartitionNumber,
    ) -> Result<Vec<SystemPartitionDescriptor>, SystemReaderError> {
        let mut descriptors = Vec::new();

        if partition_num.eq(&BOOT_LOADER_PARTITION) {
            descriptors.push(SystemPartitionDescriptor::BootLoader);
        }

        if partition_num.eq(&PROTOCOL_UPDATE_STATUS_PARTITION) {
            descriptors.push(SystemPartitionDescriptor::ProtocolUpdateStatus);
        }

        if partition_num.eq(&TYPE_INFO_FIELD_PARTITION) {
            descriptors.push(SystemPartitionDescriptor::TypeInfo);
        }

        if partition_num.eq(&SCHEMAS_PARTITION) {
            descriptors.push(SystemPartitionDescriptor::Schema);
        }

        let type_info = self.get_type_info(node_id)?;

        match type_info {
            TypeInfoSubstate::Object(object_info) => {
                let (module_id, partition_offset) = if partition_num.ge(&MAIN_BASE_PARTITION) {
                    let partition_offset = PartitionOffset(partition_num.0 - MAIN_BASE_PARTITION.0);
                    (ModuleId::Main, Some(partition_offset))
                } else {
                    match object_info.object_type {
                        ObjectType::Global { modules } => {
                            if partition_num.ge(&ROLE_ASSIGNMENT_BASE_PARTITION) {
                                if modules.contains_key(&AttachedModuleId::RoleAssignment) {
                                    let partition_offset = PartitionOffset(
                                        partition_num.0 - ROLE_ASSIGNMENT_BASE_PARTITION.0,
                                    );
                                    (ModuleId::RoleAssignment, Some(partition_offset))
                                } else {
                                    (ModuleId::Main, None)
                                }
                            } else if partition_num.ge(&ROYALTY_BASE_PARTITION) {
                                if modules.contains_key(&AttachedModuleId::Royalty) {
                                    let partition_offset =
                                        PartitionOffset(partition_num.0 - ROYALTY_BASE_PARTITION.0);
                                    (ModuleId::Royalty, Some(partition_offset))
                                } else {
                                    (ModuleId::Main, None)
                                }
                            } else if partition_num.ge(&METADATA_BASE_PARTITION) {
                                if modules.contains_key(&AttachedModuleId::Metadata) {
                                    let partition_offset = PartitionOffset(
                                        partition_num.0 - METADATA_BASE_PARTITION.0,
                                    );
                                    (ModuleId::Metadata, Some(partition_offset))
                                } else {
                                    (ModuleId::Main, None)
                                }
                            } else {
                                (ModuleId::Main, None)
                            }
                        }
                        ObjectType::Owned => (ModuleId::Main, None),
                    }
                };

                let blueprint_id = match module_id {
                    ModuleId::Main => object_info.blueprint_info.blueprint_id,
                    _ => module_id.static_blueprint().unwrap(),
                };

                let definition = self.get_blueprint_definition(&blueprint_id).unwrap();

                let state_schema = &definition.interface.state;
                match (&state_schema.fields, &partition_offset) {
                    (
                        Some((PartitionDescription::Logical(offset), _fields)),
                        Some(partition_offset),
                    ) => {
                        if offset.eq(partition_offset) {
                            descriptors.push(SystemPartitionDescriptor::Object(
                                module_id,
                                ObjectPartitionDescriptor::Fields,
                            ));
                        }
                    }
                    _ => {}
                }

                for (index, (partition_description, schema)) in
                    state_schema.collections.iter().enumerate()
                {
                    let partition_descriptor = match schema {
                        BlueprintCollectionSchema::KeyValueStore(..) => {
                            ObjectPartitionDescriptor::KeyValueCollection(index as u8)
                        }
                        BlueprintCollectionSchema::Index(..) => {
                            ObjectPartitionDescriptor::IndexCollection(index as u8)
                        }
                        BlueprintCollectionSchema::SortedIndex(..) => {
                            ObjectPartitionDescriptor::SortedIndexCollection(index as u8)
                        }
                    };

                    match (partition_description, &partition_offset) {
                        (PartitionDescription::Logical(offset), Some(partition_offset))
                            if offset.eq(partition_offset) =>
                        {
                            descriptors.push(SystemPartitionDescriptor::Object(
                                module_id,
                                partition_descriptor,
                            ))
                        }
                        (PartitionDescription::Physical(physical_partition), None)
                            if physical_partition.eq(&partition_num) =>
                        {
                            descriptors.push(SystemPartitionDescriptor::Object(
                                module_id,
                                partition_descriptor,
                            ))
                        }
                        _ => {}
                    }
                }
            }
            TypeInfoSubstate::KeyValueStore(..) => {
                if partition_num.eq(&MAIN_BASE_PARTITION) {
                    descriptors.push(SystemPartitionDescriptor::KeyValueStore);
                }
            }
            _ => {}
        }

        Ok(descriptors)
    }

    pub fn field_iter(
        &self,
        node_id: &NodeId,
        partition_number: PartitionNumber,
    ) -> Box<dyn Iterator<Item = (FieldKey, Vec<u8>)> + '_> {
        if self.state_updates.is_some() {
            panic!("fields_iter with overlay not supported.");
        }
        self.substate_db
            .list_field_raw_values(node_id, partition_number, None::<SubstateKey>)
    }

    pub fn map_iter(
        &self,
        node_id: &NodeId,
        partition_number: PartitionNumber,
    ) -> Box<dyn Iterator<Item = (MapKey, Vec<u8>)> + '_> {
        if self.state_updates.is_some() {
            panic!("map_iter with overlay not supported.");
        }
        self.substate_db
            .list_map_raw_values(node_id, partition_number, None::<SubstateKey>)
    }

    pub fn sorted_iter(
        &self,
        node_id: &NodeId,
        partition_number: PartitionNumber,
    ) -> Box<dyn Iterator<Item = (SortedKey, Vec<u8>)> + '_> {
        if self.state_updates.is_some() {
            panic!("sorted_iter with overlay not supported.");
        }
        self.substate_db
            .list_sorted_raw_values(node_id, partition_number, None::<SubstateKey>)
    }
}

struct ValidationPayloadCheckerContext<'a, S: SubstateDatabase + ?Sized> {
    reader: &'a SystemDatabaseReader<'a, S>,
    schema_origin: SchemaOrigin,
    allow_non_global_ref: bool,
    allow_ownership: bool,
}

impl<'a, S: SubstateDatabase + ?Sized> ValidationContext
    for ValidationPayloadCheckerContext<'a, S>
{
    type Error = String;

    fn get_node_type_info(&self, node_id: &NodeId) -> Result<TypeInfoForValidation, String> {
        let type_info = self
            .reader
            .get_type_info(node_id)
            .map_err(|_| "Type Info missing".to_string())?;
        let type_info_for_validation = match type_info {
            TypeInfoSubstate::Object(object_info) => TypeInfoForValidation::Object {
                package: object_info.blueprint_info.blueprint_id.package_address,
                blueprint: object_info.blueprint_info.blueprint_id.blueprint_name,
            },
            TypeInfoSubstate::KeyValueStore(..) => TypeInfoForValidation::KeyValueStore,
            TypeInfoSubstate::GlobalAddressReservation(..) => {
                TypeInfoForValidation::GlobalAddressReservation
            }
            TypeInfoSubstate::GlobalAddressPhantom(..) => {
                return Err("Found invalid stored address phantom".to_string())
            }
        };

        Ok(type_info_for_validation)
    }

    fn schema_origin(&self) -> &SchemaOrigin {
        &self.schema_origin
    }

    fn allow_ownership(&self) -> bool {
        self.allow_ownership
    }

    fn allow_non_global_ref(&self) -> bool {
        self.allow_non_global_ref
    }
}

impl<'a, S: SubstateDatabase + ListableSubstateDatabase> SystemDatabaseReader<'a, S> {
    pub fn partitions_iter(&self) -> Box<dyn Iterator<Item = (NodeId, PartitionNumber)> + '_> {
        if self.state_updates.is_some() {
            panic!("partitions_iter with overlay not supported.");
        }

        self.substate_db.read_partition_keys()
    }
}

pub struct SystemDatabaseWriter<'a, S: SubstateDatabase + CommittableSubstateDatabase> {
    substate_db: &'a mut S,
}

impl<'a, S: SubstateDatabase + CommittableSubstateDatabase> SystemDatabaseWriter<'a, S> {
    pub fn new(substate_db: &'a mut S) -> Self {
        Self { substate_db }
    }

    pub fn write_typed_object_field<V: ScryptoEncode>(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        field_index: u8,
        value: V,
    ) -> Result<(), SystemReaderError> {
        let reader = SystemDatabaseReader::new(self.substate_db);
        let blueprint_id = reader.get_blueprint_id(node_id, module_id)?;
        let definition = reader.get_blueprint_definition(&blueprint_id)?;
        let partition_description = &definition
            .interface
            .state
            .fields
            .as_ref()
            .ok_or_else(|| SystemReaderError::FieldDoesNotExist)?
            .0;
        let partition_number = match partition_description {
            PartitionDescription::Logical(offset) => {
                let base_partition = match module_id {
                    ModuleId::Main => MAIN_BASE_PARTITION,
                    ModuleId::Metadata => METADATA_BASE_PARTITION,
                    ModuleId::Royalty => ROYALTY_BASE_PARTITION,
                    ModuleId::RoleAssignment => ROLE_ASSIGNMENT_BASE_PARTITION,
                };
                base_partition.at_offset(*offset).unwrap()
            }
            PartitionDescription::Physical(partition_number) => *partition_number,
        };

        self.substate_db.update_substate(
            node_id,
            partition_number,
            SubstateKey::Field(field_index),
            FieldSubstate::new_field(value, LockStatus::Unlocked),
        );

        Ok(())
    }
}
