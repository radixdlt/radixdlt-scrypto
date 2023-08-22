use radix_engine_common::data::scrypto::ScryptoDecode;
use radix_engine_common::prelude::{scrypto_decode, scrypto_encode};
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use radix_engine_store_interface::{
    db_key_mapper::{DatabaseKeyMapper, MappedSubstateDatabase, SpreadPrefixKeyMapper},
    interface::SubstateDatabase,
};
use sbor::rust::prelude::*;
use sbor::HasLatestVersion;

use crate::blueprints::package::PackageBlueprintVersionDefinitionEntrySubstate;
use crate::system::type_info::TypeInfoSubstate;
use crate::track::TrackedNode;

pub enum SystemPartitionDescription {
    TypeInfo,
    Schema,
    Module(ObjectModuleId, PartitionOffset),
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
            .list_mapped::<SpreadPrefixKeyMapper, PackageBlueprintVersionDefinitionEntrySubstate, MapKey>(
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

            blueprints.insert(
                bp_version_key,
                blueprint_definition.value.unwrap().into_latest(),
            );
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

    pub fn get_blueprint_definition(
        &self,
        blueprint_id: &BlueprintId,
    ) -> Option<BlueprintDefinition> {
        let bp_version_key = BlueprintVersionKey::new_default(blueprint_id.blueprint_name.clone());
        let definition = self
            .fetch_substate::<SpreadPrefixKeyMapper, PackageBlueprintVersionDefinitionEntrySubstate>(
                blueprint_id.package_address.as_node_id(),
                MAIN_BASE_PARTITION
                    .at_offset(PACKAGE_BLUEPRINTS_PARTITION_OFFSET)
                    .unwrap(),
                &SubstateKey::Map(scrypto_encode(&bp_version_key).unwrap()),
            )?;

        definition.value.map(|v| v.into_latest())
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
        substate_key: &SubstateKey,
    ) -> Option<D> {
        if let Some(tracked) = self.tracked {
            tracked
                .get(node_id)
                .and_then(|tracked_node| tracked_node.tracked_partitions.get(&partition_num))
                .and_then(|tracked_module| {
                    tracked_module
                        .substates
                        .get(&M::to_db_sort_key(&substate_key))
                })
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
