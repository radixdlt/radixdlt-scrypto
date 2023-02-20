use crate::blueprints::resource::VaultSubstate;
use crate::ledger::{QueryableSubstateStore, ReadableSubstateStore};
use crate::system::global::GlobalSubstate;
use crate::system::node_substates::PersistedSubstate;
use radix_engine_interface::api::types::{
    AccountOffset, Address, ComponentOffset, GlobalOffset, KeyValueStoreOffset, NodeModuleId,
    RENodeId, SubstateId, SubstateOffset, VaultId, VaultOffset,
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
    fn visit_vault(&mut self, _vault_id: VaultId, _vault_substate: &VaultSubstate) {}
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
            RENodeId::Global(Address::Component(..)) => {
                let substate_id = SubstateId(
                    node_id,
                    NodeModuleId::SELF,
                    SubstateOffset::Global(GlobalOffset::Global),
                );
                let substate = self
                    .substate_store
                    .get_substate(&substate_id)
                    .ok_or(StateTreeTraverserError::RENodeNotFound(node_id))?;
                let global: GlobalSubstate = substate.substate.to_runtime().into();
                let derefed = global.node_deref();
                self.traverse_recursive(Some(&substate_id), derefed, depth + 1)
                    .expect("Broken Node Store");
            }
            RENodeId::Vault(vault_id) => {
                let substate_id = SubstateId(
                    RENodeId::Vault(vault_id),
                    NodeModuleId::SELF,
                    SubstateOffset::Vault(VaultOffset::Vault),
                );
                if let Some(output_value) = self.substate_store.get_substate(&substate_id) {
                    let vault_substate: VaultSubstate = output_value.substate.into();

                    self.visitor.visit_vault(vault_id, &vault_substate);
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
            RENodeId::Component(..) => {
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
            RENodeId::Account(..) => {
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
