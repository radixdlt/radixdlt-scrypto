use crate::interface::SubstateDatabase;
use radix_engine_interface::blueprints::resource::{ResourceType, VAULT_BLUEPRINT};
use radix_engine_interface::constants::RESOURCE_MANAGER_PACKAGE;
use radix_engine_interface::data::scrypto::scrypto_decode;
use radix_engine_interface::types::{
    IndexedScryptoValue, IntoEnumIterator, ModuleId, SubstateKey, TypedModuleId, VaultOffset,
};
use radix_engine_interface::{
    blueprints::resource::{LiquidFungibleResource, LiquidNonFungibleResource},
    types::NodeId,
};
use sbor::rust::prelude::*;

use super::{TypeInfoSubstate, VaultInfoSubstate};

pub struct StateTreeTraverser<'s, 'v, S: SubstateDatabase, V: StateTreeVisitor> {
    substate_db: &'s S,
    visitor: &'v mut V,
    max_depth: u32,
}

pub trait StateTreeVisitor {
    fn visit_fungible_vault(
        &mut self,
        _vault_id: NodeId,
        _info: &VaultInfoSubstate,
        _resource: &LiquidFungibleResource,
    ) {
    }

    fn visit_non_fungible_vault(
        &mut self,
        _vault_id: NodeId,
        _info: &VaultInfoSubstate,
        _resource: &LiquidNonFungibleResource,
    ) {
    }

    fn visit_node_id(
        &mut self,
        _parent_id: Option<&(NodeId, ModuleId, SubstateKey)>,
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
        parent_node_id: Option<&(NodeId, ModuleId, SubstateKey)>,
        node_id: NodeId,
    ) {
        self.traverse_recursive(parent_node_id, node_id, 0)
    }

    fn traverse_recursive(
        &mut self,
        parent: Option<&(NodeId, ModuleId, SubstateKey)>,
        node_id: NodeId,
        depth: u32,
    ) {
        if depth > self.max_depth {
            return;
        }

        // Notify visitor
        self.visitor.visit_node_id(parent, &node_id, depth);

        // Load type info
        let type_info: TypeInfoSubstate = scrypto_decode(
            &self
                .substate_db
                .get_substate(
                    &node_id,
                    TypedModuleId::TypeInfo.into(),
                    &SubstateKey::from_vec(vec![0]).unwrap(),
                )
                .expect("Failed to get substate")
                .expect("Missing TypeInfo substate")
                .0,
        )
        .expect("Failed to decode TypeInfo substate");

        match type_info {
            TypeInfoSubstate::KeyValueStore(_) => {
                for (substate_key, value) in self
                    .substate_db
                    .list_substates(&node_id, TypedModuleId::ObjectState.into())
                    .expect("Failed to list key value store")
                    .0
                {
                    let (_, owned_nodes, _) = IndexedScryptoValue::from_vec(value)
                        .expect("Substate is not a scrypto value")
                        .unpack();
                    for child_node_id in owned_nodes {
                        self.traverse_recursive(
                            Some(&(
                                node_id,
                                TypedModuleId::ObjectState.into(),
                                substate_key.clone(),
                            )),
                            child_node_id,
                            depth + 1,
                        );
                    }
                }
            }
            TypeInfoSubstate::Object {
                blueprint,
                global: _,
            } => {
                if blueprint.package_address.eq(&RESOURCE_MANAGER_PACKAGE)
                    && blueprint.blueprint_name.eq(VAULT_BLUEPRINT)
                {
                    let (value, _version) = self
                        .substate_db
                        .get_substate(
                            &node_id,
                            TypedModuleId::ObjectState.into(),
                            &VaultOffset::Info.into(),
                        )
                        .expect("Broken database")
                        .expect("Broken database");
                    let info: VaultInfoSubstate =
                        scrypto_decode(&value).expect("Failed to decode vault info");
                    match &info.resource_type {
                        ResourceType::Fungible { .. } => {
                            let liquid: LiquidFungibleResource = scrypto_decode(
                                &self
                                    .substate_db
                                    .get_substate(
                                        &node_id,
                                        TypedModuleId::ObjectState.into(),
                                        &VaultOffset::LiquidFungible.into(),
                                    )
                                    .expect("Broken database")
                                    .expect("Broken database")
                                    .0,
                            )
                            .expect("Failed to decode liquid fungible");

                            self.visitor
                                .visit_fungible_vault(node_id.into(), &info, &liquid);
                        }
                        ResourceType::NonFungible { .. } => {
                            let liquid: LiquidNonFungibleResource = scrypto_decode(
                                &self
                                    .substate_db
                                    .get_substate(
                                        &node_id,
                                        TypedModuleId::ObjectState.into(),
                                        &VaultOffset::LiquidNonFungible.into(),
                                    )
                                    .expect("Broken database")
                                    .expect("Broken database")
                                    .0,
                            )
                            .expect("Failed to decode liquid non-fungible");

                            self.visitor
                                .visit_non_fungible_vault(node_id.into(), &info, &liquid);
                        }
                    }
                } else {
                    for t in TypedModuleId::iter() {
                        // List all iterable modules (currently `ObjectState` & `Metadata`)
                        if let Ok(x) = self.substate_db.list_substates(&node_id, t.into()) {
                            for (substate_key, substate_value) in x.0 {
                                let (_, owned_nodes, _) =
                                    IndexedScryptoValue::from_vec(substate_value)
                                        .expect("Substate is not a scrypto value")
                                        .unpack();
                                for child_node_id in owned_nodes {
                                    self.traverse_recursive(
                                        Some(&(
                                            node_id,
                                            TypedModuleId::ObjectState.into(),
                                            substate_key.clone(),
                                        )),
                                        child_node_id,
                                        depth + 1,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
