use crate::blueprints::kv_store::KeyValueStore;
use crate::errors::KernelError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::LockFlags;
use crate::kernel::module::BaseModule;
use crate::kernel::{Kernel, KernelNodeApi, KernelSubstateApi, RENodeInit};
use crate::system::component::{
    ComponentInfoSubstate, ComponentRoyaltyAccumulatorSubstate, ComponentRoyaltyConfigSubstate,
    ComponentStateSubstate,
};
use crate::system::invocation::invoke_native::invoke_native_fn;
use crate::system::invocation::invoke_scrypto::invoke_scrypto_fn;
use crate::system::invocation::resolve_function::resolve_function;
use crate::system::invocation::resolve_method::resolve_method;
use crate::system::kernel_modules::fee::FeeReserve;
use crate::system::node_modules::auth::AccessRulesChainSubstate;
use crate::system::node_modules::metadata::MetadataSubstate;
use crate::system::substates::RuntimeSubstate;
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::{
    ClientActorApi, ClientApi, ClientComponentApi, ClientDerefApi, ClientMeteringApi,
    ClientNodeApi, ClientPackageApi, ClientStaticInvokeApi, ClientSubstateApi, Invokable,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::constants::RADIX_TOKEN;
use radix_engine_interface::data::types::Own;
use radix_engine_interface::data::*;
use sbor::rust::string::ToString;
use sbor::rust::vec;
use sbor::rust::vec::Vec;

impl<'g, 's, W, R, M> ClientNodeApi<RuntimeError> for Kernel<'g, 's, W, R, M>
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
}

impl<'g, 's, W, R, M> ClientSubstateApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
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

impl<'g, 's, W, R, M> ClientDerefApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn deref(&mut self, node_id: RENodeId) -> Result<Option<(RENodeId, LockHandle)>, RuntimeError> {
        self.node_method_deref(node_id)
    }
}

impl<'g, 's, W, R, M> ClientActorApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn fn_identifier(&mut self) -> Result<FnIdentifier, RuntimeError> {
        Ok(self.current_frame.actor.identifier.clone())
    }
}

impl<'g, 's, W, R, M> ClientStaticInvokeApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
}

impl<'g, 's, W, R, M> ClientPackageApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn call_function(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: String,
        function_name: String,
        args: Vec<u8>,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        // TODO: Use execution mode?
        let invocation =
            resolve_function(package_address, blueprint_name, function_name, args, self)?;
        Ok(match invocation {
            CallTableInvocation::Native(native) => {
                IndexedScryptoValue::from_typed(invoke_native_fn(native, self)?.as_ref())
            }
            CallTableInvocation::Scrypto(scrypto) => invoke_scrypto_fn(scrypto, self)?,
        })
    }

    fn get_code(&mut self, package_address: PackageAddress) -> Result<PackageCode, RuntimeError> {
        let package_global = RENodeId::Global(GlobalAddress::Package(package_address));
        let handle = self.lock_substate(
            package_global,
            SubstateOffset::Package(PackageOffset::Info),
            LockFlags::read_only(),
        )?;
        let substate_ref = self.get_ref(handle)?;
        let package = substate_ref.package_info();
        let code = package.code().to_vec();
        self.drop_lock(handle)?;
        Ok(PackageCode::Wasm(code))
    }

    fn get_abi(
        &mut self,
        package_address: PackageAddress,
    ) -> Result<BTreeMap<String, BlueprintAbi>, RuntimeError> {
        let package_global = RENodeId::Global(GlobalAddress::Package(package_address));
        let handle = self.lock_substate(
            package_global,
            SubstateOffset::Package(PackageOffset::Info),
            LockFlags::read_only(),
        )?;
        let substate_ref = self.get_ref(handle)?;
        let package = substate_ref.package_info();
        let abi = package.blueprint_abis.clone();
        self.drop_lock(handle)?;
        Ok(abi)
    }
}

impl<'g, 's, W, R, M> ClientComponentApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn call_method(
        &mut self,
        receiver: ScryptoReceiver,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        // TODO: Use execution mode?
        let invocation = resolve_method(receiver, method_name, &args, self)?;
        Ok(match invocation {
            CallTableInvocation::Native(native) => {
                IndexedScryptoValue::from_typed(invoke_native_fn(native, self)?.as_ref())
            }
            CallTableInvocation::Scrypto(scrypto) => invoke_scrypto_fn(scrypto, self)?,
        })
    }
}

impl<'g, 's, W, R, M> ClientMeteringApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn consume_cost_units(&mut self, units: u32) -> Result<(), RuntimeError> {
        self.module
            .on_wasm_costing(&self.current_frame, &mut self.heap, &mut self.track, units)
            .map_err(RuntimeError::ModuleError)?;

        Ok(())
    }

    fn lock_fee(
        &mut self,
        vault_id: VaultId,
        fee: Resource,
        contingent: bool,
    ) -> Result<Resource, RuntimeError> {
        let rtn = self.module.on_lock_fee(
            &self.current_frame,
            &mut self.heap,
            &mut self.track,
            vault_id,
            fee,
            contingent,
        )?;

        Ok(rtn)
    }
}

impl<'g, 's, W, R, M> ClientApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
}
