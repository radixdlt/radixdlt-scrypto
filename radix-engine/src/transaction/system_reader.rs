use radix_engine_common::data::scrypto::ScryptoDecode;
use radix_engine_common::prelude::scrypto_encode;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::package::{
    BlueprintDefinition, BlueprintVersionKey, PACKAGE_BLUEPRINTS_PARTITION_OFFSET,
};
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use radix_engine_store_interface::{
    db_key_mapper::{DatabaseKeyMapper, MappedSubstateDatabase, SpreadPrefixKeyMapper},
    interface::SubstateDatabase,
};
use sbor::rust::prelude::*;

use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::system::KeyValueEntrySubstate;
use crate::track::TrackedNode;

pub enum SystemPartitionDescription {
    TypeInfo,
    Module(ObjectModuleId, PartitionOffset),
}

pub struct SystemReader<'a, S: SubstateDatabase> {
    substate_db: &'a S,
    tracked: &'a IndexMap<NodeId, TrackedNode>,
}

impl<'a, S: SubstateDatabase> SystemReader<'a, S> {
    pub fn new(substate_db: &'a S, tracked: &'a IndexMap<NodeId, TrackedNode>) -> Self {
        Self {
            substate_db,
            tracked,
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

    pub fn fetch_substate<M: DatabaseKeyMapper, D: ScryptoDecode>(
        &self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        key: &SubstateKey,
    ) -> Option<D> {
        // FIXME: explore if we can avoid loading from substate database
        // - Part of the engine still reads/writes substates without touching the TypeInfo;
        // - Track does not store the initial value of substate.

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
        self.tracked
            .get(node_id)
            .and_then(|tracked_node| tracked_node.tracked_partitions.get(&partition_num))
            .and_then(|tracked_module| tracked_module.substates.get(&M::to_db_sort_key(key)))
            .and_then(|tracked_key| {
                tracked_key
                    .substate_value
                    .get()
                    .map(|e| e.as_typed().unwrap())
            })
    }
}
