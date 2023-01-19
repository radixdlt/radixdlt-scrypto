use crate::engine::{
    BaseModule, Kernel, KernelError, LockFlags, RENodeInit, RuntimeError, SystemApi,
};
use crate::fee::FeeReserve;
use crate::model::MetadataSubstate;
use crate::model::{
    AccessRulesChainSubstate, ComponentInfoSubstate, ComponentRoyaltyAccumulatorSubstate,
    ComponentRoyaltyConfigSubstate, ComponentStateSubstate, KeyValueStore, RuntimeSubstate,
};
use crate::types::BTreeMap;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::{
    ComponentFn, LockHandle, NativeFn, RENodeId, RENodeType, ScryptoRENode, SubstateOffset,
};
use radix_engine_interface::api::{EngineApi, Invokable};
use radix_engine_interface::constants::RADIX_TOKEN;
use radix_engine_interface::data::types::Own;
use radix_engine_interface::model::{
    AccessRule, AccessRuleKey, AccessRules, ResourceManagerCreateVaultInvocation, RoyaltyConfig,
};
use sbor::rust::string::ToString;
use sbor::rust::vec;
use sbor::rust::vec::Vec;

impl<'g, 's, W, R, M> EngineApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn sys_create_node(&mut self, node: ScryptoRENode) -> Result<RENodeId, RuntimeError> {
        let (node_id, node) = match node {
            ScryptoRENode::Component(package_address, blueprint_name, state) => {
                let node_id = self.allocate_node_id(RENodeType::Component)?;

                // Create a royalty vault
                let royalty_vault_id = self
                    .invoke(ResourceManagerCreateVaultInvocation {
                        receiver: RADIX_TOKEN,
                    })?
                    .vault_id();

                // Royalty initialization done here
                let royalty_config = ComponentRoyaltyConfigSubstate {
                    royalty_config: RoyaltyConfig::default(),
                };
                let royalty_accumulator = ComponentRoyaltyAccumulatorSubstate {
                    royalty: Own::Vault(royalty_vault_id.into()),
                };

                // TODO: Remove Royalties from Node's access rule chain, possibly implement this
                // TODO: via associated nodes rather than inheritance?
                let mut access_rules =
                    AccessRules::new().default(AccessRule::AllowAll, AccessRule::AllowAll);
                access_rules.set_group_and_mutability(
                    AccessRuleKey::Native(NativeFn::Component(ComponentFn::ClaimRoyalty)),
                    "royalty".to_string(),
                    AccessRule::DenyAll,
                );
                access_rules.set_group_and_mutability(
                    AccessRuleKey::Native(NativeFn::Component(ComponentFn::SetRoyaltyConfig)),
                    "royalty".to_string(),
                    AccessRule::DenyAll,
                );
                access_rules.set_group_access_rule_and_mutability(
                    "royalty".to_string(),
                    AccessRule::AllowAll,
                    AccessRule::AllowAll,
                );

                let node = RENodeInit::Component(
                    ComponentInfoSubstate::new(package_address, blueprint_name),
                    ComponentStateSubstate::new(state),
                    royalty_config,
                    royalty_accumulator,
                    MetadataSubstate {
                        metadata: BTreeMap::new(),
                    },
                    AccessRulesChainSubstate {
                        access_rules_chain: vec![access_rules],
                    },
                );

                (node_id, node)
            }
            ScryptoRENode::KeyValueStore => {
                let node_id = self.allocate_node_id(RENodeType::KeyValueStore)?;
                let node = RENodeInit::KeyValueStore(KeyValueStore::new());
                (node_id, node)
            }
        };

        self.create_node(node_id, node)?;

        Ok(node_id)
    }

    fn sys_drop_node(&mut self, node_id: RENodeId) -> Result<(), RuntimeError> {
        self.drop_node(node_id)?;
        Ok(())
    }

    fn sys_get_visible_nodes(&mut self) -> Result<Vec<RENodeId>, RuntimeError> {
        self.get_visible_nodes()
    }

    fn sys_lock_substate(
        &mut self,
        node_id: RENodeId,
        offset: SubstateOffset,
        mutable: bool,
    ) -> Result<LockHandle, RuntimeError> {
        let flags = if mutable {
            LockFlags::MUTABLE
        } else {
            // TODO: Do we want to expose full flag functionality to Scrypto?
            LockFlags::read_only()
        };

        self.lock_substate(node_id, offset, flags)
    }

    fn sys_read(&mut self, lock_handle: LockHandle) -> Result<Vec<u8>, RuntimeError> {
        self.get_ref(lock_handle)
            .map(|substate_ref| substate_ref.to_scrypto_value().into_vec())
    }

    fn sys_write(&mut self, lock_handle: LockHandle, buffer: Vec<u8>) -> Result<(), RuntimeError> {
        let offset = self.get_lock_info(lock_handle)?.offset;
        let substate = RuntimeSubstate::decode_from_buffer(&offset, &buffer)?;
        let mut substate_mut = self.get_ref_mut(lock_handle)?;

        match substate {
            RuntimeSubstate::ComponentState(next) => *substate_mut.component_state() = next,
            RuntimeSubstate::KeyValueStoreEntry(next) => {
                *substate_mut.kv_store_entry() = next;
            }
            RuntimeSubstate::NonFungible(next) => {
                *substate_mut.non_fungible() = next;
            }
            _ => return Err(RuntimeError::KernelError(KernelError::InvalidOverwrite)),
        }

        Ok(())
    }

    fn sys_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError> {
        self.drop_lock(lock_handle)
    }
}
