use radix_engine::system::node_modules::type_info::TypeInfoSubstate;
use radix_engine::track::db_key_mapper::{
    MappedSubstateDatabase, SpreadPrefixKeyMapper, SubstateKeyContent,
};
use radix_engine::types::{MapKey, ScryptoValue, SubstateKey, TupleKey};
use radix_engine_interface::blueprints::account::ACCOUNT_BLUEPRINT;
use radix_engine_interface::blueprints::resource::{
    LiquidNonFungibleVault, FUNGIBLE_VAULT_BLUEPRINT, NON_FUNGIBLE_VAULT_BLUEPRINT,
};
use radix_engine_interface::constants::{ACCOUNT_PACKAGE, RESOURCE_PACKAGE};
use radix_engine_interface::data::scrypto::model::NonFungibleLocalId;
use radix_engine_interface::types::{
    FungibleVaultOffset, IndexedScryptoValue, ModuleNumber, NonFungibleVaultOffset, ObjectInfo,
    ResourceAddress, TypeInfoOffset, ACCESS_RULES_BASE_MODULE, METADATA_BASE_MODULE,
    OBJECT_BASE_MODULE, ROYALTY_BASE_MODULE, TYPE_INFO_BASE_MODULE,
};
use radix_engine_interface::{blueprints::resource::LiquidFungibleResource, types::NodeId};
use radix_engine_store_interface::interface::SubstateDatabase;
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
        _parent_id: Option<&(NodeId, ModuleNumber, SubstateKey)>,
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
        parent_node_id: Option<&(NodeId, ModuleNumber, SubstateKey)>,
        node_id: NodeId,
    ) {
        self.traverse_recursive(parent_node_id, node_id, 0)
    }

    fn traverse_recursive(
        &mut self,
        parent: Option<&(NodeId, ModuleNumber, SubstateKey)>,
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
                TYPE_INFO_BASE_MODULE,
                &TypeInfoOffset::TypeInfo.into(),
            )
            .expect("Missing TypeInfo substate");

        match type_info {
            TypeInfoSubstate::KeyValueStore(_) => {
                for (substate_key, value) in self
                    .substate_db
                    .list_mapped::<SpreadPrefixKeyMapper, ScryptoValue, MapKey>(
                        &node_id,
                        OBJECT_BASE_MODULE,
                    )
                {
                    let (_, owned_nodes, _) =
                        IndexedScryptoValue::from_scrypto_value(value).unpack();
                    for child_node_id in owned_nodes {
                        self.traverse_recursive(
                            Some(&(node_id, OBJECT_BASE_MODULE, substate_key.clone())),
                            child_node_id,
                            depth + 1,
                        );
                    }
                }
            }
            TypeInfoSubstate::Index | TypeInfoSubstate::SortedIndex => {}
            TypeInfoSubstate::Object(ObjectInfo {
                blueprint,
                outer_object,
                global: _,
                instance_schema: _,
            }) => {
                if blueprint.package_address.eq(&RESOURCE_PACKAGE)
                    && blueprint.blueprint_name.eq(FUNGIBLE_VAULT_BLUEPRINT)
                {
                    let liquid = self
                        .substate_db
                        .get_mapped::<SpreadPrefixKeyMapper, LiquidFungibleResource>(
                            &node_id,
                            OBJECT_BASE_MODULE,
                            &FungibleVaultOffset::LiquidFungible.into(),
                        )
                        .expect("Broken database");

                    self.visitor.visit_fungible_vault(
                        node_id,
                        &ResourceAddress::new_or_panic(outer_object.unwrap().into()),
                        &liquid,
                    );
                } else if blueprint.package_address.eq(&RESOURCE_PACKAGE)
                    && blueprint.blueprint_name.eq(NON_FUNGIBLE_VAULT_BLUEPRINT)
                {
                    let liquid = self
                        .substate_db
                        .get_mapped::<SpreadPrefixKeyMapper, LiquidNonFungibleVault>(
                            &node_id,
                            OBJECT_BASE_MODULE,
                            &NonFungibleVaultOffset::LiquidNonFungible.into(),
                        )
                        .expect("Broken database");

                    self.visitor.visit_non_fungible_vault(
                        node_id,
                        &ResourceAddress::new_or_panic(outer_object.unwrap().into()),
                        &liquid,
                    );

                    let entries = self
                        .substate_db
                        .list_mapped::<SpreadPrefixKeyMapper, NonFungibleLocalId, MapKey>(
                            liquid.ids.as_node_id(),
                            OBJECT_BASE_MODULE,
                        );
                    for (_key, non_fungible_local_id) in entries {
                        self.visitor.visit_non_fungible(
                            node_id,
                            &ResourceAddress::new_or_panic(outer_object.unwrap().into()),
                            &non_fungible_local_id,
                        );
                    }
                } else {
                    for module_num in [
                        TYPE_INFO_BASE_MODULE,
                        ROYALTY_BASE_MODULE,
                        ACCESS_RULES_BASE_MODULE,
                    ] {
                        self.traverse_substates::<TupleKey>(node_id, module_num, depth)
                    }
                    for module_num in [METADATA_BASE_MODULE] {
                        self.traverse_substates::<MapKey>(node_id, module_num, depth)
                    }

                    if blueprint.package_address.eq(&ACCOUNT_PACKAGE)
                        && blueprint.blueprint_name.eq(ACCOUNT_BLUEPRINT)
                    {
                        self.traverse_substates::<MapKey>(node_id, OBJECT_BASE_MODULE, depth)
                    } else {
                        self.traverse_substates::<TupleKey>(node_id, OBJECT_BASE_MODULE, depth)
                    }
                }
            }
        }
    }

    fn traverse_substates<K: SubstateKeyContent>(
        &mut self,
        node_id: NodeId,
        module_num: ModuleNumber,
        depth: u32,
    ) {
        let entries = self
            .substate_db
            .list_mapped::<SpreadPrefixKeyMapper, ScryptoValue, K>(&node_id, module_num);
        for (substate_key, substate_value) in entries {
            let (_, owned_nodes, _) =
                IndexedScryptoValue::from_scrypto_value(substate_value).unpack();
            for child_node_id in owned_nodes {
                self.traverse_recursive(
                    Some(&(node_id, module_num, substate_key.clone())),
                    child_node_id,
                    depth + 1,
                );
            }
        }
    }
}
