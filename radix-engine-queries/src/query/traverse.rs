use radix_engine::system::node_modules::type_info::TypeInfoSubstate;
use radix_engine::types::{FieldKey, MapKey, ScryptoValue, SubstateKey};
use radix_engine_interface::blueprints::account::ACCOUNT_BLUEPRINT;
use radix_engine_interface::blueprints::resource::{
    LiquidNonFungibleVault, FUNGIBLE_VAULT_BLUEPRINT, NON_FUNGIBLE_VAULT_BLUEPRINT,
};
use radix_engine_interface::constants::{ACCOUNT_PACKAGE, RESOURCE_PACKAGE};
use radix_engine_interface::data::scrypto::model::NonFungibleLocalId;
use radix_engine_interface::types::{
    AccountPartitionOffset, FungibleVaultField, IndexedScryptoValue, NonFungibleVaultField,
    PartitionNumber, PartitionOffset, ResourceAddress, TypeInfoField, ACCESS_RULES_BASE_PARTITION,
    MAIN_BASE_PARTITION, METADATA_KV_STORE_PARTITION, ROYALTY_BASE_PARTITION,
    TYPE_INFO_FIELD_PARTITION,
};
use radix_engine_interface::{blueprints::resource::LiquidFungibleResource, types::NodeId};
use radix_engine_store_interface::{
    db_key_mapper::{MappedSubstateDatabase, SpreadPrefixKeyMapper, SubstateKeyContent},
    interface::SubstateDatabase,
};
use sbor::rust::prelude::*;

pub struct StateTreeTraverser<'s, 'v, S: SubstateDatabase, V: StateTreeVisitor> {
    substate_db: &'s S,
    visitor: &'v mut V,
    max_depth: u32,
}

pub trait StateTreeVisitor {
    fn visit_fungible_vault(
        &mut self,
        _vault_id: NodeId,
        _address: &ResourceAddress,
        _resource: &LiquidFungibleResource,
    ) {
    }

    fn visit_non_fungible_vault(
        &mut self,
        _vault_id: NodeId,
        _address: &ResourceAddress,
        _resource: &LiquidNonFungibleVault,
    ) {
    }

    fn visit_non_fungible(
        &mut self,
        _vault_id: NodeId,
        _address: &ResourceAddress,
        _id: &NonFungibleLocalId,
    ) {
    }

    fn visit_node_id(
        &mut self,
        _parent_id: Option<&(NodeId, PartitionNumber, SubstateKey)>,
        _node_id: &NodeId,
        _depth: u32,
    ) {
    }
}

impl<'s, 'v, S: SubstateDatabase, V: StateTreeVisitor> StateTreeTraverser<'s, 'v, S, V> {
    pub fn new(substate_db: &'s S, visitor: &'v mut V, max_depth: u32) -> Self {
        StateTreeTraverser {
            substate_db,
            visitor,
            max_depth,
        }
    }

    pub fn traverse_all_descendents(
        &mut self,
        parent_node_id: Option<&(NodeId, PartitionNumber, SubstateKey)>,
        node_id: NodeId,
    ) {
        self.traverse_recursive(parent_node_id, node_id, 0)
    }

    fn traverse_recursive(
        &mut self,
        parent: Option<&(NodeId, PartitionNumber, SubstateKey)>,
        node_id: NodeId,
        depth: u32,
    ) {
        if depth > self.max_depth {
            return;
        }

        // Notify visitor
        self.visitor.visit_node_id(parent, &node_id, depth);

        // Load type info
        let type_info = self
            .substate_db
            .get_mapped::<SpreadPrefixKeyMapper, TypeInfoSubstate>(
                &node_id,
                TYPE_INFO_FIELD_PARTITION,
                &TypeInfoField::TypeInfo.into(),
            )
            .expect("Missing TypeInfo substate");

        match type_info {
            TypeInfoSubstate::KeyValueStore(_) => {
                for (substate_key, value) in self
                    .substate_db
                    .list_mapped::<SpreadPrefixKeyMapper, ScryptoValue, MapKey>(
                        &node_id,
                        MAIN_BASE_PARTITION,
                    )
                {
                    let (_, owned_nodes, _) =
                        IndexedScryptoValue::from_scrypto_value(value).unpack();
                    for child_node_id in owned_nodes {
                        self.traverse_recursive(
                            Some(&(node_id, MAIN_BASE_PARTITION, substate_key.clone())),
                            child_node_id,
                            depth + 1,
                        );
                    }
                }
            }
            TypeInfoSubstate::Object(info) => {
                if info.blueprint_id.package_address.eq(&RESOURCE_PACKAGE)
                    && info
                        .blueprint_id
                        .blueprint_name
                        .eq(FUNGIBLE_VAULT_BLUEPRINT)
                {
                    let liquid = self
                        .substate_db
                        .get_mapped::<SpreadPrefixKeyMapper, LiquidFungibleResource>(
                            &node_id,
                            MAIN_BASE_PARTITION,
                            &FungibleVaultField::LiquidFungible.into(),
                        )
                        .expect("Broken database");

                    self.visitor.visit_fungible_vault(
                        node_id,
                        &ResourceAddress::new_or_panic(info.get_outer_object().into()),
                        &liquid,
                    );
                } else if info.blueprint_id.package_address.eq(&RESOURCE_PACKAGE)
                    && info
                        .blueprint_id
                        .blueprint_name
                        .eq(NON_FUNGIBLE_VAULT_BLUEPRINT)
                {
                    let liquid = self
                        .substate_db
                        .get_mapped::<SpreadPrefixKeyMapper, LiquidNonFungibleVault>(
                            &node_id,
                            MAIN_BASE_PARTITION,
                            &NonFungibleVaultField::LiquidNonFungible.into(),
                        )
                        .expect("Broken database");

                    self.visitor.visit_non_fungible_vault(
                        node_id,
                        &ResourceAddress::new_or_panic(info.get_outer_object().into()),
                        &liquid,
                    );

                    let entries = self
                        .substate_db
                        .list_mapped::<SpreadPrefixKeyMapper, NonFungibleLocalId, MapKey>(
                            &node_id,
                            MAIN_BASE_PARTITION.at_offset(PartitionOffset(1u8)).unwrap(),
                        );
                    for (_key, non_fungible_local_id) in entries {
                        self.visitor.visit_non_fungible(
                            node_id,
                            &ResourceAddress::new_or_panic(info.get_outer_object().into()),
                            &non_fungible_local_id,
                        );
                    }
                } else {
                    for partition_num in [
                        TYPE_INFO_FIELD_PARTITION,
                        ROYALTY_BASE_PARTITION,
                        ACCESS_RULES_BASE_PARTITION,
                    ] {
                        self.traverse_substates::<FieldKey>(node_id, partition_num, depth)
                    }
                    for partition_num in [METADATA_KV_STORE_PARTITION] {
                        self.traverse_substates::<MapKey>(node_id, partition_num, depth)
                    }

                    if info.blueprint_id.package_address.eq(&ACCOUNT_PACKAGE)
                        && info.blueprint_id.blueprint_name.eq(ACCOUNT_BLUEPRINT)
                    {
                        self.traverse_substates::<MapKey>(
                            node_id,
                            MAIN_BASE_PARTITION
                                .at_offset(
                                    AccountPartitionOffset::AccountVaultsByResourceAddress.into(),
                                )
                                .unwrap(),
                            depth,
                        );
                        self.traverse_substates::<MapKey>(
                            node_id,
                            MAIN_BASE_PARTITION
                                .at_offset(
                                    AccountPartitionOffset::AccountResourceDepositRuleByAddress
                                        .into(),
                                )
                                .unwrap(),
                            depth,
                        )
                    } else {
                        self.traverse_substates::<FieldKey>(node_id, MAIN_BASE_PARTITION, depth)
                    }
                }
            }
            _ => {}
        }
    }

    fn traverse_substates<K: SubstateKeyContent>(
        &mut self,
        node_id: NodeId,
        partition_num: PartitionNumber,
        depth: u32,
    ) {
        let entries = self
            .substate_db
            .list_mapped::<SpreadPrefixKeyMapper, ScryptoValue, K>(&node_id, partition_num);
        for (substate_key, substate_value) in entries {
            let (_, owned_nodes, _) =
                IndexedScryptoValue::from_scrypto_value(substate_value).unpack();
            for child_node_id in owned_nodes {
                self.traverse_recursive(
                    Some(&(node_id, partition_num, substate_key.clone())),
                    child_node_id,
                    depth + 1,
                );
            }
        }
    }
}
