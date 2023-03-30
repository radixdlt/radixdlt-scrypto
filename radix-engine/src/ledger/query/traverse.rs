use crate::blueprints::resource::{
    FungibleVaultInfoSubstate, NonFungibleVaultInfoSubstate, VaultUtil,
};
use crate::ledger::{QueryableSubstateStore, ReadableSubstateStore};
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::node_substates::PersistedSubstate;
use crate::types::*;
use radix_engine_interface::blueprints::resource::{
    LiquidFungibleResource, LiquidNonFungibleResource, ResourceType, FUNGIBLE_VAULT_BLUEPRINT,
    NON_FUNGIBLE_VAULT_BLUEPRINT,
};

#[derive(Debug)]
pub enum StateTreeTraverserError {
    RENodeNotFound(RENodeId),
    MaxDepthExceeded,
}

pub struct StateTreeTraverser<
    's,
    'v,
    S: ReadableSubstateStore + QueryableSubstateStore,
    V: StateTreeVisitor,
> {
    substate_store: &'s S,
    visitor: &'v mut V,
    max_depth: u32,
}

pub trait StateTreeVisitor {
    fn visit_fungible_vault(
        &mut self,
        _vault_id: ObjectId,
        _info: &FungibleVaultInfoSubstate,
        _resource: &LiquidFungibleResource,
    ) {
    }

    fn visit_non_fungible_vault(
        &mut self,
        _vault_id: ObjectId,
        _info: &NonFungibleVaultInfoSubstate,
        _resource: &LiquidNonFungibleResource,
    ) {
    }

    fn visit_node_id(&mut self, _parent_id: Option<&SubstateId>, _node_id: &RENodeId, _depth: u32) {
    }
}

impl<'s, 'v, S: ReadableSubstateStore + QueryableSubstateStore, V: StateTreeVisitor>
    StateTreeTraverser<'s, 'v, S, V>
{
    pub fn new(substate_store: &'s S, visitor: &'v mut V, max_depth: u32) -> Self {
        StateTreeTraverser {
            substate_store,
            visitor,
            max_depth,
        }
    }

    pub fn traverse_all_descendents(
        &mut self,
        parent_node_id: Option<&SubstateId>,
        node_id: RENodeId,
    ) -> Result<(), StateTreeTraverserError> {
        self.traverse_recursive(parent_node_id, node_id, 0)
    }

    fn traverse_recursive(
        &mut self,
        parent: Option<&SubstateId>,
        node_id: RENodeId,
        depth: u32,
    ) -> Result<(), StateTreeTraverserError> {
        if depth > self.max_depth {
            return Err(StateTreeTraverserError::MaxDepthExceeded);
        }
        self.visitor.visit_node_id(parent, &node_id, depth);
        match node_id {
            RENodeId::KeyValueStore(kv_store_id) => {
                let map = self.substate_store.get_kv_store_entries(&kv_store_id);
                for (entry_id, substate) in map.iter() {
                    let substate_id = SubstateId(
                        RENodeId::KeyValueStore(kv_store_id),
                        NodeModuleId::SELF,
                        SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(entry_id.clone())),
                    );
                    if let PersistedSubstate::KeyValueStoreEntry(entry) = substate {
                        if let Some(value) = entry {
                            let (_, own, _) =
                                IndexedScryptoValue::from_scrypto_value(value.clone()).unpack();
                            for child_node_id in own {
                                self.traverse_recursive(
                                    Some(&substate_id),
                                    child_node_id,
                                    depth + 1,
                                )
                                .expect("Broken Node Store");
                            }
                        }
                    }
                }
            }
            RENodeId::Object(..) => {
                let substate_id = SubstateId(
                    node_id,
                    NodeModuleId::TypeInfo,
                    SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
                );
                let output_value = self
                    .substate_store
                    .get_substate(&substate_id)
                    .expect("Broken Node Store");
                let runtime_substate = output_value.substate.to_runtime();
                let type_substate: TypeInfoSubstate = runtime_substate.into();

                match type_substate {
                    TypeInfoSubstate::Object { blueprint, .. }
                        if VaultUtil::is_vault_blueprint(&blueprint) =>
                    {
                        if let Some(output_value) = self.substate_store.get_substate(&SubstateId(
                            node_id,
                            NodeModuleId::SELF,
                            SubstateOffset::Vault(VaultOffset::Info),
                        )) {
                            match blueprint.blueprint_name.as_str() {
                                FUNGIBLE_VAULT_BLUEPRINT => {
                                    let info: FungibleVaultInfoSubstate =
                                        output_value.substate.into();
                                    let liquid: LiquidFungibleResource = self
                                        .substate_store
                                        .get_substate(&SubstateId(
                                            node_id,
                                            NodeModuleId::SELF,
                                            SubstateOffset::Vault(VaultOffset::LiquidFungible),
                                        ))
                                        .unwrap()
                                        .substate
                                        .into();

                                    self.visitor.visit_fungible_vault(
                                        node_id.into(),
                                        &info,
                                        &liquid,
                                    );
                                }
                                NON_FUNGIBLE_VAULT_BLUEPRINT => {
                                    let info: NonFungibleVaultInfoSubstate =
                                        output_value.substate.into();
                                    let liquid: LiquidNonFungibleResource = self
                                        .substate_store
                                        .get_substate(&SubstateId(
                                            node_id,
                                            NodeModuleId::SELF,
                                            SubstateOffset::Vault(VaultOffset::LiquidNonFungible),
                                        ))
                                        .unwrap()
                                        .substate
                                        .into();

                                    self.visitor.visit_non_fungible_vault(
                                        node_id.into(),
                                        &info,
                                        &liquid,
                                    );
                                }
                                _ => panic!("Unexpected vault blueprint"),
                            }
                        } else {
                            return Err(StateTreeTraverserError::RENodeNotFound(node_id));
                        }
                    }
                    _ => {
                        let substate_id = SubstateId(
                            node_id,
                            NodeModuleId::SELF,
                            SubstateOffset::Component(ComponentOffset::State0),
                        );
                        let output_value = self
                            .substate_store
                            .get_substate(&substate_id)
                            .expect("Broken Node Store");
                        let runtime_substate = output_value.substate.to_runtime();
                        let substate_ref = runtime_substate.to_ref();
                        let (_, owned_nodes) = substate_ref.references_and_owned_nodes();
                        for child_node_id in owned_nodes {
                            self.traverse_recursive(Some(&substate_id), child_node_id, depth + 1)
                                .expect("Broken Node Store");
                        }
                    }
                }
            }
            RENodeId::GlobalObject(Address::Component(ComponentAddress::Account(..)))
            | RENodeId::GlobalObject(Address::Component(
                ComponentAddress::EcdsaSecp256k1VirtualAccount(..),
            ))
            | RENodeId::GlobalObject(Address::Component(
                ComponentAddress::EddsaEd25519VirtualAccount(..),
            )) => {
                let substate_id = SubstateId(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Account(AccountOffset::Account),
                );
                let output_value = self
                    .substate_store
                    .get_substate(&substate_id)
                    .expect("Broken Node Store");
                let runtime_substate = output_value.substate.to_runtime();
                let substate_ref = runtime_substate.to_ref();
                let (_, owned_nodes) = substate_ref.references_and_owned_nodes();
                for child_node_id in owned_nodes {
                    self.traverse_recursive(Some(&substate_id), child_node_id, depth + 1)
                        .expect("Broken Node Store");
                }
            }
            RENodeId::GlobalObject(Address::Component(_)) => {
                let substate_id = SubstateId(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Component(ComponentOffset::State0),
                );
                let output_value = self
                    .substate_store
                    .get_substate(&substate_id)
                    .expect("Broken Node Store");
                let runtime_substate = output_value.substate.to_runtime();
                let substate_ref = runtime_substate.to_ref();
                let (_, owned_nodes) = substate_ref.references_and_owned_nodes();
                for child_node_id in owned_nodes {
                    self.traverse_recursive(Some(&substate_id), child_node_id, depth + 1)
                        .expect("Broken Node Store");
                }
            }
            _ => {}
        };

        Ok(())
    }
}
