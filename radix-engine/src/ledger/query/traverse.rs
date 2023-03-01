use crate::blueprints::resource::VaultInfoSubstate;
use crate::ledger::{QueryableSubstateStore, ReadableSubstateStore};
use crate::system::node_substates::PersistedSubstate;
use radix_engine_interface::api::types::{
    AccountOffset, ComponentAddress, ComponentOffset, KeyValueStoreOffset, NodeModuleId, RENodeId,
    SubstateId, SubstateOffset, VaultId, VaultOffset,
};
use radix_engine_interface::blueprints::resource::{
    LiquidFungibleResource, LiquidNonFungibleResource, ResourceType,
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
        _vault_id: VaultId,
        _info: &VaultInfoSubstate,
        _resource: &LiquidFungibleResource,
    ) {
    }

    fn visit_non_fungible_vault(
        &mut self,
        _vault_id: VaultId,
        _info: &VaultInfoSubstate,
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
            RENodeId::Vault(vault_id) => {
                if let Some(output_value) = self.substate_store.get_substate(&SubstateId(
                    RENodeId::Vault(vault_id),
                    NodeModuleId::SELF,
                    SubstateOffset::Vault(VaultOffset::Info),
                )) {
                    let info: VaultInfoSubstate = output_value.substate.into();
                    match &info.resource_type {
                        ResourceType::Fungible { .. } => {
                            let liquid: LiquidFungibleResource = self
                                .substate_store
                                .get_substate(&SubstateId(
                                    RENodeId::Vault(vault_id),
                                    NodeModuleId::SELF,
                                    SubstateOffset::Vault(VaultOffset::LiquidFungible),
                                ))
                                .unwrap()
                                .substate
                                .into();

                            self.visitor.visit_fungible_vault(vault_id, &info, &liquid);
                        }
                        ResourceType::NonFungible { .. } => {
                            let liquid: LiquidNonFungibleResource = self
                                .substate_store
                                .get_substate(&SubstateId(
                                    RENodeId::Vault(vault_id),
                                    NodeModuleId::SELF,
                                    SubstateOffset::Vault(VaultOffset::LiquidNonFungible),
                                ))
                                .unwrap()
                                .substate
                                .into();

                            self.visitor
                                .visit_non_fungible_vault(vault_id, &info, &liquid);
                        }
                    }
                } else {
                    return Err(StateTreeTraverserError::RENodeNotFound(node_id));
                }
            }
            RENodeId::KeyValueStore(kv_store_id) => {
                let map = self.substate_store.get_kv_store_entries(&kv_store_id);
                for (entry_id, substate) in map.iter() {
                    let substate_id = SubstateId(
                        RENodeId::KeyValueStore(kv_store_id),
                        NodeModuleId::SELF,
                        SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(entry_id.clone())),
                    );
                    if let PersistedSubstate::KeyValueStoreEntry(entry) = substate {
                        for child_node_id in entry.owned_node_ids() {
                            self.traverse_recursive(Some(&substate_id), child_node_id, depth + 1)
                                .expect("Broken Node Store");
                        }
                    }
                }
            }
            RENodeId::Object(..) => {
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
            RENodeId::GlobalComponent(ComponentAddress::Account(..))
            | RENodeId::GlobalComponent(ComponentAddress::EcdsaSecp256k1VirtualAccount(..))
            | RENodeId::GlobalComponent(ComponentAddress::EddsaEd25519VirtualAccount(..)) => {
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
            _ => {}
        };

        Ok(())
    }
}
