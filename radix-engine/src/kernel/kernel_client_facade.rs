use super::Invokable;
use crate::errors::ApplicationError;
use crate::errors::KernelError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::LockFlags;
use crate::kernel::module::BaseModule;
use crate::kernel::{Kernel, KernelNodeApi, KernelSubstateApi};
use crate::system::global::GlobalAddressSubstate;
use crate::system::invocation::invoke_native::invoke_native_fn;
use crate::system::invocation::resolve_function::resolve_function;
use crate::system::invocation::resolve_method::resolve_method;
use crate::system::invocation::resolve_native::resolve_native;
use crate::system::kernel_modules::fee::FeeReserve;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::system::node_modules::auth::AccessRulesChainSubstate;
use crate::system::node_modules::metadata::MetadataSubstate;
use crate::system::node_substates::RuntimeSubstate;
use crate::system::package::PackageError;
use crate::types::*;
use crate::wasm::WasmEngine;
use crate::wasm::WasmValidator;
use radix_engine_interface::api::component::{
    ComponentInfoSubstate, ComponentRoyaltyAccumulatorSubstate, ComponentRoyaltyConfigSubstate,
    ComponentStateSubstate,
};
use radix_engine_interface::api::package::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::{
    ClientActorApi, ClientApi, ClientComponentApi, ClientDerefApi, ClientMeteringApi,
    ClientNativeInvokeApi, ClientNodeApi, ClientPackageApi, ClientSubstateApi,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::constants::RADIX_TOKEN;
use radix_engine_interface::data::types::Own;
use radix_engine_interface::data::*;
use sbor::rust::string::ToString;
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

        self.lock_substate(node_id, NodeModuleId::SELF, offset, flags)
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

impl<'g, 's, W, R, M> ClientNativeInvokeApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn call_native_raw(
        &mut self,
        native_fn: NativeFn,
        invocation: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let call_table_invocation = resolve_native(native_fn, invocation)?;
        match call_table_invocation {
            CallTableInvocation::Native(native_invocation) => {
                invoke_native_fn(native_invocation, self)
                    .map(|r| scrypto_encode(r.as_ref()).unwrap())
            }
            CallTableInvocation::Scrypto(_) => {
                panic!("TODO: better interface")
            }
        }
    }

    fn call_native<N: SerializableInvocation>(
        &mut self,
        invocation: N,
    ) -> Result<N::Output, RuntimeError> {
        let native_fn = N::native_fn();
        let invocation = scrypto_encode(&invocation).expect("Failed to encode native invocation");
        let return_data = self.call_native_raw(native_fn, invocation)?;
        Ok(scrypto_decode(&return_data).expect("Failed to decode native return data"))
    }
}

impl<'g, 's, W, R, M> ClientPackageApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn new_package(
        &mut self,
        code: Vec<u8>,
        abi: Vec<u8>,
        access_rules_chain: Vec<AccessRules>,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
        metadata: BTreeMap<String, String>,
    ) -> Result<PackageAddress, RuntimeError> {
        // Validate code
        let abi = scrypto_decode(&abi).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidAbi(e),
            ))
        })?;
        WasmValidator::default()
            .validate(&code, &abi)
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::PackageError(
                    PackageError::InvalidWasm(e),
                ))
            })?;

        // Allocate node id
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

        // Create auth substates
        let auth_substate = AccessRulesChainSubstate { access_rules_chain };

        self.create_node(
            node_id,
            RENodeInit::Package(PackageInfoSubstate {
                code,
                blueprint_abis: abi,
            }),
            btreemap!(
                NodeModuleId::PackageRoyalty => RENodeModuleInit::PackageRoyalty(
                    royalty_config_substate,
                    royalty_accumulator_substate
                ),
                NodeModuleId::Metadata => RENodeModuleInit::Metadata(metadata_substate),
                NodeModuleId::AccessRules => RENodeModuleInit::AccessRulesChain(auth_substate),
            ),
        )?;

        Ok(node_id.into())
    }

    fn call_function(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        // TODO: Use execution mode?
        let invocation = resolve_function(
            package_address,
            blueprint_name.to_string(),
            function_name.to_string(),
            args,
            self,
        )?;
        match invocation {
            CallTableInvocation::Native(native_invocation) => Ok(scrypto_encode(
                invoke_native_fn(native_invocation, self)?.as_ref(),
            )
            .expect("Failed to encode native fn return")),
            CallTableInvocation::Scrypto(scrypto_invocation) => self
                .invoke(scrypto_invocation)
                .map(|v| scrypto_encode(&v).expect("Failed to encode scrypto fn return")),
        }
    }

    fn get_code(&mut self, package_address: PackageAddress) -> Result<PackageCode, RuntimeError> {
        let package_global = RENodeId::Global(GlobalAddress::Package(package_address));
        let handle = self.lock_substate(
            package_global,
            NodeModuleId::SELF,
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
            NodeModuleId::SELF,
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
    fn lookup_global_component(
        &mut self,
        component_address: ComponentAddress,
    ) -> Result<ComponentId, RuntimeError> {
        let offset = SubstateOffset::Global(GlobalOffset::Global);
        let handle = self.lock_substate(
            RENodeId::Global(GlobalAddress::Component(component_address)),
            NodeModuleId::SELF,
            offset,
            LockFlags::empty(),
        )?;
        let substate_ref = self.get_ref(handle)?;
        Ok(substate_ref.global_address().node_deref().into())
    }

    fn new_component(
        &mut self,
        blueprint_ident: &str,
        app_states: BTreeMap<u8, Vec<u8>>,
        access_rules_chain: Vec<AccessRules>,
        royalty_config: RoyaltyConfig,
        metadata: BTreeMap<String, String>,
    ) -> Result<ComponentId, RuntimeError> {
        // Allocate node id
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

        // Create auth substates
        let auth_substate = AccessRulesChainSubstate { access_rules_chain };

        // Create component RENode
        // FIXME: support native blueprints
        let package_address = match self.current_frame.actor.identifier.clone() {
            FnIdentifier::Scrypto(s) => s.package_address,
            FnIdentifier::Native(_) => todo!(),
        };
        let blueprint_ident = blueprint_ident.to_string();
        // FIXME: generalize app substates;
        // FIXME: remove unwrap;
        // FIXME: support native blueprints
        let abi_enforced_app_substate = app_states.into_iter().next().unwrap().1;

        self.create_node(
            node_id,
            RENodeInit::Component(ComponentStateSubstate::new(abi_enforced_app_substate)),
            btreemap!(
                NodeModuleId::ComponentTypeInfo => RENodeModuleInit::ComponentTypeInfo(
                    ComponentInfoSubstate::new(package_address, blueprint_ident.to_string())
                ),
                NodeModuleId::ComponentRoyalty => RENodeModuleInit::ComponentRoyalty(
                    royalty_config_substate,
                    royalty_accumulator_substate
                ),
                NodeModuleId::Metadata => RENodeModuleInit::Metadata(metadata_substate),
                NodeModuleId::AccessRules => RENodeModuleInit::AccessRulesChain(auth_substate),
            ),
        )?;

        Ok(node_id.into())
    }

    fn globalize_component(
        &mut self,
        component_id: ComponentId,
    ) -> Result<ComponentAddress, RuntimeError> {
        let node_id = self.allocate_node_id(RENodeType::GlobalComponent)?;

        self.create_node(
            node_id,
            RENodeInit::Global(GlobalAddressSubstate::Component(component_id)),
            btreemap!(),
        )?;

        Ok(node_id.into())
    }

    fn call_method(
        &mut self,
        receiver: ScryptoReceiver,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        // TODO: Use execution mode?
        let invocation = resolve_method(receiver, method_name, &args, self)?;
        match invocation {
            CallTableInvocation::Native(native_invocation) => Ok(scrypto_encode(
                invoke_native_fn(native_invocation, self)?.as_ref(),
            )
            .expect("Failed to encode native fn return")),
            CallTableInvocation::Scrypto(scrypto_invocation) => self
                .invoke(scrypto_invocation)
                .map(|v| scrypto_encode(&v).expect("Failed to encode scrypto fn return")),
        }
    }

    fn get_component_type_info(
        &mut self,
        component_id: ComponentId,
    ) -> Result<(PackageAddress, String), RuntimeError> {
        let component_node_id = RENodeId::Component(component_id);
        let handle = self.lock_substate(
            component_node_id,
            NodeModuleId::ComponentTypeInfo,
            SubstateOffset::ComponentTypeInfo(ComponentTypeInfoOffset::TypeInfo),
            LockFlags::read_only(),
        )?;
        let substate_ref = self.get_ref(handle)?;
        let info = substate_ref.component_info();
        let package_address = info.package_address.clone();
        let blueprint_ident = info.blueprint_name.clone();
        self.drop_lock(handle)?;
        Ok((package_address, blueprint_ident))
    }

    fn new_key_value_store(&mut self) -> Result<KeyValueStoreId, RuntimeError> {
        let node_id = self.allocate_node_id(RENodeType::KeyValueStore)?;

        self.create_node(node_id, RENodeInit::KeyValueStore, btreemap!())?;

        Ok(node_id.into())
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
