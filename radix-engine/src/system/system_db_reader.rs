use radix_engine_common::data::scrypto::ScryptoDecode;
use radix_engine_common::prelude::{scrypto_decode, scrypto_encode};
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::package::{BlueprintDefinition, BlueprintPartitionType, BlueprintVersionKey, PACKAGE_BLUEPRINTS_PARTITION_OFFSET, PartitionDescription};
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use radix_engine_store_interface::{
    db_key_mapper::{DatabaseKeyMapper, MappedSubstateDatabase, SpreadPrefixKeyMapper},
    interface::SubstateDatabase,
};
use radix_engine_store_interface::db_key_mapper::SubstateKeyContent;
use radix_engine_store_interface::interface::{ListableSubstateDatabase};
use sbor::rust::prelude::*;

use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::system::KeyValueEntrySubstate;
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

    pub fn get_blueprint_id(&self, node_id: &NodeId, module_id: ObjectModuleId) -> Option<BlueprintId> {
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
                ObjectModuleId::Main => {
                    Some(object_info.blueprint_info.blueprint_id)
                }
                _ => Some(module_id.static_blueprint().unwrap())
            }
        } else { None }
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

                let (module_id, partition_offset) =
                    if partition_num.ge(&MAIN_BASE_PARTITION) {
                        let partition_offset = PartitionOffset(partition_num.0 - MAIN_BASE_PARTITION.0);
                        (ObjectModuleId::Main, Some(partition_offset))
                    } else if partition_num.ge(&ROLE_ASSIGNMENT_BASE_PARTITION) {
                        if object_info.module_versions.contains_key(&ObjectModuleId::RoleAssignment) {
                            let partition_offset =
                                PartitionOffset(partition_num.0 - ROLE_ASSIGNMENT_BASE_PARTITION.0);
                            (ObjectModuleId::RoleAssignment, Some(partition_offset))
                        } else {
                            (ObjectModuleId::Main, None)
                        }
                    } else if partition_num.ge(&ROYALTY_BASE_PARTITION) {
                        if object_info.module_versions.contains_key(&ObjectModuleId::Royalty) {
                            let partition_offset = PartitionOffset(partition_num.0 - ROYALTY_BASE_PARTITION.0);
                            (ObjectModuleId::Royalty, Some(partition_offset))
                        } else {
                            (ObjectModuleId::Main, None)
                        }
                    } else if partition_num.ge(&METADATA_BASE_PARTITION) {
                        if object_info.module_versions.contains_key(&ObjectModuleId::Metadata) {
                            let partition_offset = PartitionOffset(partition_num.0 - METADATA_BASE_PARTITION.0);
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
                    (Some((PartitionDescription::Logical(offset), _fields)), Some(partition_offset)) => if offset.eq(partition_offset) {
                        descriptors.push(SystemPartitionDescriptor::Object(module_id, ObjectPartitionDescriptor::Field));
                    }
                    _ => {}
                }

                for (index, (partition_description, schema)) in state_schema.collections.iter().enumerate() {
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
                        (PartitionDescription::Logical(offset), Some(partition_offset)) if offset.eq(partition_offset) => {
                            descriptors.push(SystemPartitionDescriptor::Object(module_id, partition_descriptor))
                        }
                        (PartitionDescription::Physical(physical_partition), None) if physical_partition.eq(&partition_num) => {
                            descriptors.push(SystemPartitionDescriptor::Object(module_id, partition_descriptor))
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

    pub fn substates_iter<K: SubstateKeyContent>(&self, node_id: &NodeId, partition_number: PartitionNumber) -> Box<dyn Iterator<Item = (SubstateKey, Vec<u8>)> + '_> {
        if self.tracked.is_some() {
            panic!("substates_iter with overlay not supported.");
        }

        let partition_key = SpreadPrefixKeyMapper::to_db_partition_key(node_id, partition_number);
        let iter = self.substate_db.list_entries(&partition_key)
            .map(|entry| {
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

        let iter = self.substate_db.list_partition_keys()
            .map(|partition_key| {
                let canonical_partition = SpreadPrefixKeyMapper::from_db_partition_key(&partition_key);
                canonical_partition
            });
        Box::new(iter)
    }
}

