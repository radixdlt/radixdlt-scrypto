use crate::errors::KernelError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::LockFlags;
use crate::kernel::module::BaseModule;
use crate::kernel::{Kernel, KernelNodeApi, KernelSubstateApi};
use crate::system::component::{
    ComponentInfoSubstate, ComponentRoyaltyAccumulatorSubstate, ComponentRoyaltyConfigSubstate,
    ComponentStateSubstate,
};
use crate::system::invocation::invoke_native::invoke_native_fn;
use crate::system::invocation::invoke_scrypto::invoke_scrypto_fn;
use crate::system::invocation::resolve_function::resolve_function;
use crate::system::invocation::resolve_method::resolve_method;
use crate::system::kernel_modules::fee::FeeReserve;
use crate::system::node::RENodeInit;
use crate::system::node_modules::auth::AccessRulesChainSubstate;
use crate::system::node_modules::metadata::MetadataSubstate;
use crate::system::node_substates::RuntimeSubstate;
use crate::system::package::PackageInfoSubstate;
use crate::system::package::PackageRoyaltyAccumulatorSubstate;
use crate::system::package::PackageRoyaltyConfigSubstate;
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

    fn sys_read_substate(&mut self, lock_handle: LockHandle) -> Result<Vec<u8>, RuntimeError> {
        self.get_ref(lock_handle)
            .map(|substate_ref| substate_ref.to_scrypto_value().into_vec())
    }

    fn sys_write_substate(
        &mut self,
        lock_handle: LockHandle,
        buffer: Vec<u8>,
    ) -> Result<(), RuntimeError> {
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
    fn instantiate_package(
        &mut self,
        code: Vec<u8>,
        abi: BTreeMap<String, BlueprintAbi>,
        access_rules: AccessRules,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
        metadata: BTreeMap<String, String>,
    ) -> Result<PackageAddress, RuntimeError> {
        let node_id = self.allocate_node_id(RENodeType::Package)?;

        // Create a royalty vault
        let royalty_vault_id = self
            .invoke(ResourceManagerCreateVaultInvocation {
                receiver: RADIX_TOKEN,
            })?
            .vault_id();

        // Create royalty substates
        let royalty_config_substate = PackageRoyaltyConfigSubstate { royalty_config };
        let royalty_accumulator_substate = PackageRoyaltyAccumulatorSubstate {
            royalty: Own::Vault(royalty_vault_id.into()),
        };

        // Create metadata substates
        let metadata_substate = MetadataSubstate { metadata };

        // Create auth substates (TODO: set up auth in client space)
        let auth_substate = AccessRulesChainSubstate {
            access_rules_chain: vec![access_rules],
        };

        let node = RENodeInit::Package(
            PackageInfoSubstate {
                code,
                blueprint_abis: abi,
            },
            royalty_config_substate,
            royalty_accumulator_substate,
            metadata_substate,
            auth_substate,
        );

        self.create_node(node_id, node)?;

        Ok(node_id.into())
    }

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
    fn instantiate_component(
        &mut self,
        blueprint_ident: &str,
        app_states: BTreeMap<u8, Vec<u8>>,
        access_rules: AccessRules,
        royalty_config: RoyaltyConfig,
        metadata: BTreeMap<String, String>,
    ) -> Result<ComponentId, RuntimeError> {
        let node_id = self.allocate_node_id(RENodeType::Component)?;

        // Create a royalty vault
        let royalty_vault_id = self
            .invoke(ResourceManagerCreateVaultInvocation {
                receiver: RADIX_TOKEN,
            })?
            .vault_id();

        // Create royalty substates
        let royalty_config_substate = ComponentRoyaltyConfigSubstate { royalty_config };
        let royalty_accumulator_substate = ComponentRoyaltyAccumulatorSubstate {
            royalty: Own::Vault(royalty_vault_id.into()),
        };

        // Create metadata substates
        let metadata_substate = MetadataSubstate { metadata };

        // Create auth substates (TODO: set up auth in client space)
        let auth_substate = AccessRulesChainSubstate {
            access_rules_chain: vec![access_rules],
        };

        // Create component RENode
        // FIXME: support native blueprints
        let package_address = match self.current_frame.actor.identifier {
            FnIdentifier::Scrypto(s) => s.package_address,
            FnIdentifier::Native(_) => todo!(),
        };
        let blueprint_ident = blueprint_ident.to_string();
        // FIXME: generalize app substates;
        // FIXME: remove unwrap;
        // FIXME: support native blueprints
        let abi_enforced_app_substate = app_states.into_iter().next().unwrap().1;
        let node = RENodeInit::Component(
            ComponentInfoSubstate::new(package_address, blueprint_ident.to_string()),
            ComponentStateSubstate::new(abi_enforced_app_substate),
            royalty_config_substate,
            royalty_accumulator_substate,
            metadata_substate,
            auth_substate,
        );

        self.create_node(node_id, node)?;

        Ok(node_id.into())
    }

    fn globalize_component(
        &mut self,
        component_id: ComponentId,
    ) -> Result<ComponentAddress, RuntimeError> {
        todo!()
    }

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
