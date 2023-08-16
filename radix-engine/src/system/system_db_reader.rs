use radix_engine_common::data::scrypto::ScryptoDecode;
use radix_engine_common::prelude::{scrypto_decode, scrypto_encode, Hash, ScryptoSchema};
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::package::{BlueprintDefinition, BlueprintPartitionType, BlueprintPayloadDef, BlueprintPayloadIdentifier, BlueprintVersionKey, PartitionDescription, PACKAGE_BLUEPRINTS_PARTITION_OFFSET, KeyOrValue};
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use radix_engine_store_interface::db_key_mapper::SubstateKeyContent;
use radix_engine_store_interface::interface::ListableSubstateDatabase;
use radix_engine_store_interface::{
    db_key_mapper::{DatabaseKeyMapper, MappedSubstateDatabase, SpreadPrefixKeyMapper},
    interface::SubstateDatabase,
};
use sbor::rust::prelude::*;
use sbor::LocalTypeIndex;

use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::payload_validation::SchemaOrigin;
use crate::system::system::KeyValueEntrySubstate;
use crate::system::system_type_checker::{BlueprintTypeTarget, KVStoreTypeTarget, SchemaValidationMeta};
use crate::track::TrackedNode;
use crate::types::BlueprintCollectionSchema;

pub enum SystemPartitionDescription {
    TypeInfo,
    Schema,
    Module(ObjectModuleId, PartitionOffset),
}

pub enum ObjectPartitionDescriptor {
    Field,
    KeyValueCollection(u8),
    IndexCollection(u8),
    SortedIndexCollection(u8),
}

pub enum SystemPartitionDescriptor {
    TypeInfo,
    Schema,
    KeyValueStore,
    Object(ObjectModuleId, ObjectPartitionDescriptor),
}

pub struct ResolvedPayloadSchema {
    pub schema: ScryptoSchema,
    pub type_index: LocalTypeIndex,
    pub allow_ownership: bool,
    pub allow_non_global_refs: bool,
    pub schema_origin: SchemaOrigin,
}

/// A System Layer (Layer 2) abstraction over an underlying substate database
pub struct SystemDatabaseReader<'a, S: SubstateDatabase> {
    substate_db: &'a S,
    tracked: Option<&'a IndexMap<NodeId, TrackedNode>>,
}

impl<'a, S: SubstateDatabase> SystemDatabaseReader<'a, S> {
    pub fn new_with_overlay(
        substate_db: &'a S,
        tracked: &'a IndexMap<NodeId, TrackedNode>,
    ) -> Self {
        Self {
            substate_db,
            tracked: Some(tracked),
        }
    }

    pub fn new(substate_db: &'a S) -> Self {
        Self {
            substate_db,
            tracked: None,
        }
    }

    pub fn get_type_info(&self, node_id: &NodeId) -> Option<TypeInfoSubstate> {
        self.fetch_substate::<SpreadPrefixKeyMapper, TypeInfoSubstate>(
            node_id,
            TYPE_INFO_FIELD_PARTITION,
            &TypeInfoField::TypeInfo.into(),
        )
    }

    pub fn get_package_definition(
        &self,
        package_address: PackageAddress,
    ) -> BTreeMap<BlueprintVersionKey, BlueprintDefinition> {
        let entries = self.substate_db
            .list_mapped::<SpreadPrefixKeyMapper, KeyValueEntrySubstate<BlueprintDefinition>, MapKey>(
                package_address.as_node_id(),
                MAIN_BASE_PARTITION
                    .at_offset(PACKAGE_BLUEPRINTS_PARTITION_OFFSET)
                    .unwrap(),
            );

        let mut blueprints = BTreeMap::new();
        for (key, blueprint_definition) in entries {
            let bp_version_key: BlueprintVersionKey = match key {
                SubstateKey::Map(v) => scrypto_decode(&v).unwrap(),
                _ => panic!("Unexpected"),
            };

            blueprints.insert(bp_version_key, blueprint_definition.value.unwrap());
        }

        blueprints
    }

    pub fn get_object_info<A: Into<GlobalAddress>>(&self, address: A) -> Option<ObjectInfo> {
        let type_info = self.fetch_substate::<SpreadPrefixKeyMapper, TypeInfoSubstate>(
            address.into().as_node_id(),
            TYPE_INFO_FIELD_PARTITION,
            &TypeInfoField::TypeInfo.into(),
        )?;

        match type_info {
            TypeInfoSubstate::Object(object_info) => Some(object_info),
            i @ _ => panic!(
                "Inconsistent Substate Database, found invalid type_info: {:?}",
                i
            ),
        }
    }

    pub fn get_blueprint_id(
        &self,
        node_id: &NodeId,
        module_id: ObjectModuleId,
    ) -> Option<BlueprintId> {
        let type_info = self.fetch_substate::<SpreadPrefixKeyMapper, TypeInfoSubstate>(
            node_id,
            TYPE_INFO_FIELD_PARTITION,
            &TypeInfoField::TypeInfo.into(),
        )?;

        let object_info = match type_info {
            TypeInfoSubstate::Object(object_info) => object_info,
            i @ _ => panic!(
                "Inconsistent Substate Database, found invalid type_info: {:?}",
                i
            ),
        };

        if let Some(version) = object_info.module_versions.get(&module_id) {
            match module_id {
                ObjectModuleId::Main => Some(object_info.blueprint_info.blueprint_id),
                _ => Some(module_id.static_blueprint().unwrap()),
            }
        } else {
            None
        }
    }

    pub fn get_blueprint_definition(
        &self,
        blueprint_id: &BlueprintId,
    ) -> Option<BlueprintDefinition> {
        let bp_version_key = BlueprintVersionKey::new_default(blueprint_id.blueprint_name.clone());
        let definition = self
            .fetch_substate::<SpreadPrefixKeyMapper, KeyValueEntrySubstate<BlueprintDefinition>>(
                blueprint_id.package_address.as_node_id(),
                MAIN_BASE_PARTITION
                    .at_offset(PACKAGE_BLUEPRINTS_PARTITION_OFFSET)
                    .unwrap(),
                &SubstateKey::Map(scrypto_encode(&bp_version_key).unwrap()),
            )?;

        definition.value
    }

    pub fn get_kv_store_type_target(
        &self,
        node_id: &NodeId,
    ) -> Option<KVStoreTypeTarget> {
        let type_info = self.fetch_substate::<SpreadPrefixKeyMapper, TypeInfoSubstate>(
            node_id,
            TYPE_INFO_FIELD_PARTITION,
            &TypeInfoField::TypeInfo.into(),
        )?;

        let kv_store_info = match type_info {
            TypeInfoSubstate::KeyValueStore(kv_store_info) => kv_store_info,
            _ => return None,
        };

        Some(KVStoreTypeTarget {
            kv_store_type: kv_store_info.generic_substitutions,
            meta: *node_id,
        })
    }

    pub fn get_blueprint_type_target(
        &self,
        node_id: &NodeId,
        module_id: ObjectModuleId,
    ) -> Option<BlueprintTypeTarget> {
        let type_info = self.fetch_substate::<SpreadPrefixKeyMapper, TypeInfoSubstate>(
            node_id,
            TYPE_INFO_FIELD_PARTITION,
            &TypeInfoField::TypeInfo.into(),
        )?;

        let object_info = match type_info {
            TypeInfoSubstate::Object(object_info) => object_info,
            i @ _ => return None,
        };

        if let Some(_version) = object_info.module_versions.get(&module_id) {
            match module_id {
                ObjectModuleId::Main => {
                    let target = BlueprintTypeTarget {
                        blueprint_info: object_info.blueprint_info,
                        meta: SchemaValidationMeta::ExistingObject {
                            additional_schemas: *node_id,
                        },
                    };
                    Some(target)
                }
                _ => {
                    let blueprint_id = module_id.static_blueprint().unwrap();
                    let target = BlueprintTypeTarget {
                        blueprint_info: BlueprintInfo {
                            blueprint_id,
                            outer_obj_info: OuterObjectInfo::None,
                            features: Default::default(),
                            generic_substitutions: Default::default(),
                        },
                        meta: SchemaValidationMeta::ExistingObject {
                            additional_schemas: *node_id,
                        },
                    };
                    Some(target)
                }
            }
        } else {
            None
        }
    }

    pub fn get_kv_store_payload_schema(
        &self,
        target: &KVStoreTypeTarget,
        key_or_value: KeyOrValue,
    ) -> Option<ResolvedPayloadSchema> {
        let (substs, allow_ownership, allow_non_global_refs) = match key_or_value {
            KeyOrValue::Key => (&target.kv_store_type.key_generic_substitutions, false, false),
            KeyOrValue::Value => (&target.kv_store_type.value_generic_substitutions, target.kv_store_type.allow_ownership, false),
        };

        match substs {
            GenericSubstitution::Local(type_identifier) => {
                let schema = self.get_schema(&target.meta, &type_identifier.0)?;

                Some(ResolvedPayloadSchema {
                    schema,
                    type_index: type_identifier.1,
                    allow_ownership,
                    allow_non_global_refs,
                    schema_origin: SchemaOrigin::KeyValueStore,
                })
            }
        }
    }

    // TODO: The logic here is currently copied from system_type_checker.rs get_payload_schema().
    // It would be nice to use the same underlying code but currently too many refactors are required
    // to make that happen.
    pub fn get_blueprint_payload_schema(
        &self,
        target: &BlueprintTypeTarget,
        payload_identifier: &BlueprintPayloadIdentifier,
    ) -> Option<ResolvedPayloadSchema> {
        let blueprint_interface = self
            .get_blueprint_definition(&target.blueprint_info.blueprint_id)?
            .interface;

        let (payload_def, allow_ownership, allow_non_global_refs) =
            blueprint_interface.get_payload_def(payload_identifier)?;

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
                    .get(instance_index as usize)?;

                match generic_substitution {
                    GenericSubstitution::Local(type_id) => {
                        let schema = match &target.meta {
                            SchemaValidationMeta::ExistingObject { additional_schemas } => {
                                self.get_schema(additional_schemas, &type_id.0)?
                            }
                            SchemaValidationMeta::NewObject { additional_schemas } => {
                                additional_schemas.get(&type_id.0)?.clone()
                            }
                            SchemaValidationMeta::Blueprint => {
                                return None;
                            }
                        };

                        (schema, type_id.1, SchemaOrigin::Instance)
                    }
                }
            }
        };

        Some(ResolvedPayloadSchema {
            schema,
            type_index: index,
            allow_ownership,
            allow_non_global_refs,
            schema_origin,
        })
    }

    fn get_schema(&self, node_id: &NodeId, schema_hash: &Hash) -> Option<ScryptoSchema> {
        let schema = self
            .fetch_substate::<SpreadPrefixKeyMapper, KeyValueEntrySubstate<ScryptoSchema>>(
                node_id,
                SCHEMAS_PARTITION,
                &SubstateKey::Map(scrypto_encode(schema_hash).unwrap()),
            )?;

        Some(
            schema
                .value
                .expect("Schema should exist if substate exists"),
        )
    }

    pub fn get_blueprint_payload_def(
        &self,
        blueprint_id: &BlueprintId,
    ) -> Option<BlueprintDefinition> {
        let bp_version_key = BlueprintVersionKey::new_default(blueprint_id.blueprint_name.clone());
        let definition = self
            .fetch_substate::<SpreadPrefixKeyMapper, KeyValueEntrySubstate<BlueprintDefinition>>(
                blueprint_id.package_address.as_node_id(),
                MAIN_BASE_PARTITION
                    .at_offset(PACKAGE_BLUEPRINTS_PARTITION_OFFSET)
                    .unwrap(),
                &SubstateKey::Map(scrypto_encode(&bp_version_key).unwrap()),
            )?;

        definition.value
    }

    pub fn fetch_substate<M: DatabaseKeyMapper, D: ScryptoDecode>(
        &self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        key: &SubstateKey,
    ) -> Option<D> {
        self.fetch_substate_from_state_updates::<M, D>(node_id, partition_num, key)
            .or_else(|| self.fetch_substate_from_database::<M, D>(node_id, partition_num, key))
    }

    pub fn fetch_substate_from_database<M: DatabaseKeyMapper, D: ScryptoDecode>(
        &self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        key: &SubstateKey,
    ) -> Option<D> {
        self.substate_db
            .get_mapped::<M, D>(node_id, partition_num, key)
    }

    pub fn fetch_substate_from_state_updates<M: DatabaseKeyMapper, D: ScryptoDecode>(
        &self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        key: &SubstateKey,
    ) -> Option<D> {
        if let Some(tracked) = self.tracked {
            tracked
                .get(node_id)
                .and_then(|tracked_node| tracked_node.tracked_partitions.get(&partition_num))
                .and_then(|tracked_module| tracked_module.substates.get(&M::to_db_sort_key(key)))
                .and_then(|tracked_key| {
                    tracked_key
                        .substate_value
                        .get()
                        .map(|e| e.as_typed().unwrap())
                })
        } else {
            None
        }
    }
}

// Reverse Mapping Functionality
impl<'a, S: SubstateDatabase> SystemDatabaseReader<'a, S> {
    pub fn partition_description(
        &self,
        partition_num: &PartitionNumber,
    ) -> SystemPartitionDescription {
        if partition_num.ge(&MAIN_BASE_PARTITION) {
            let partition_offset = PartitionOffset(partition_num.0 - MAIN_BASE_PARTITION.0);
            SystemPartitionDescription::Module(ObjectModuleId::Main, partition_offset)
        } else if partition_num.ge(&ROLE_ASSIGNMENT_BASE_PARTITION) {
            let partition_offset =
                PartitionOffset(partition_num.0 - ROLE_ASSIGNMENT_BASE_PARTITION.0);
            SystemPartitionDescription::Module(ObjectModuleId::RoleAssignment, partition_offset)
        } else if partition_num.ge(&ROYALTY_BASE_PARTITION) {
            let partition_offset = PartitionOffset(partition_num.0 - ROYALTY_BASE_PARTITION.0);
            SystemPartitionDescription::Module(ObjectModuleId::Royalty, partition_offset)
        } else if partition_num.ge(&METADATA_BASE_PARTITION) {
            let partition_offset = PartitionOffset(partition_num.0 - METADATA_BASE_PARTITION.0);
            SystemPartitionDescription::Module(ObjectModuleId::Metadata, partition_offset)
        } else if partition_num.ge(&SCHEMAS_PARTITION) {
            SystemPartitionDescription::Schema
        } else if partition_num.eq(&TYPE_INFO_FIELD_PARTITION) {
            SystemPartitionDescription::TypeInfo
        } else {
            panic!("Should not get here")
        }
    }

    pub fn get_partition_descriptors(
        &self,
        node_id: &NodeId,
        partition_num: &PartitionNumber,
    ) -> Vec<SystemPartitionDescriptor> {
        let mut descriptors = Vec::new();

        if partition_num.eq(&TYPE_INFO_FIELD_PARTITION) {
            descriptors.push(SystemPartitionDescriptor::TypeInfo);
        }

        if partition_num.eq(&SCHEMAS_PARTITION) {
            descriptors.push(SystemPartitionDescriptor::Schema);
        }

        let type_info = match self.get_type_info(node_id) {
            Some(type_info) => type_info,
            _ => return vec![],
        };

        match type_info {
            TypeInfoSubstate::Object(object_info) => {
                let (module_id, partition_offset) = if partition_num.ge(&MAIN_BASE_PARTITION) {
                    let partition_offset = PartitionOffset(partition_num.0 - MAIN_BASE_PARTITION.0);
                    (ObjectModuleId::Main, Some(partition_offset))
                } else if partition_num.ge(&ROLE_ASSIGNMENT_BASE_PARTITION) {
                    if object_info
                        .module_versions
                        .contains_key(&ObjectModuleId::RoleAssignment)
                    {
                        let partition_offset =
                            PartitionOffset(partition_num.0 - ROLE_ASSIGNMENT_BASE_PARTITION.0);
                        (ObjectModuleId::RoleAssignment, Some(partition_offset))
                    } else {
                        (ObjectModuleId::Main, None)
                    }
                } else if partition_num.ge(&ROYALTY_BASE_PARTITION) {
                    if object_info
                        .module_versions
                        .contains_key(&ObjectModuleId::Royalty)
                    {
                        let partition_offset =
                            PartitionOffset(partition_num.0 - ROYALTY_BASE_PARTITION.0);
                        (ObjectModuleId::Royalty, Some(partition_offset))
                    } else {
                        (ObjectModuleId::Main, None)
                    }
                } else if partition_num.ge(&METADATA_BASE_PARTITION) {
                    if object_info
                        .module_versions
                        .contains_key(&ObjectModuleId::Metadata)
                    {
                        let partition_offset =
                            PartitionOffset(partition_num.0 - METADATA_BASE_PARTITION.0);
                        (ObjectModuleId::Metadata, Some(partition_offset))
                    } else {
                        (ObjectModuleId::Main, None)
                    }
                } else {
                    (ObjectModuleId::Main, None)
                };

                let blueprint_id = match module_id {
                    ObjectModuleId::Main => object_info.blueprint_info.blueprint_id,
                    _ => module_id.static_blueprint().unwrap(),
                };

                let definition = match self.get_blueprint_definition(&blueprint_id) {
                    Some(definition) => definition,
                    _ => return vec![],
                };

                let state_schema = definition.interface.state;

                match (&state_schema.fields, &partition_offset) {
                    (
                        Some((PartitionDescription::Logical(offset), _fields)),
                        Some(partition_offset),
                    ) => {
                        if offset.eq(partition_offset) {
                            descriptors.push(SystemPartitionDescriptor::Object(
                                module_id,
                                ObjectPartitionDescriptor::Field,
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

        descriptors
    }

    pub fn substates_iter<K: SubstateKeyContent>(
        &self,
        node_id: &NodeId,
        partition_number: PartitionNumber,
    ) -> Box<dyn Iterator<Item = (SubstateKey, Vec<u8>)> + '_> {
        if self.tracked.is_some() {
            panic!("substates_iter with overlay not supported.");
        }

        let partition_key = SpreadPrefixKeyMapper::to_db_partition_key(node_id, partition_number);
        let iter = self.substate_db.list_entries(&partition_key).map(|entry| {
            let substate_key = SpreadPrefixKeyMapper::from_db_sort_key::<K>(&entry.0);
            (substate_key, entry.1)
        });

        Box::new(iter)
    }
}

impl<'a, S: SubstateDatabase + ListableSubstateDatabase> SystemDatabaseReader<'a, S> {
    pub fn partitions_iter(&self) -> Box<dyn Iterator<Item = (NodeId, PartitionNumber)> + '_> {
        if self.tracked.is_some() {
            panic!("partitions_iter with overlay not supported.");
        }

        let iter = self.substate_db.list_partition_keys().map(|partition_key| {
            let canonical_partition = SpreadPrefixKeyMapper::from_db_partition_key(&partition_key);
            canonical_partition
        });
        Box::new(iter)
    }
}
