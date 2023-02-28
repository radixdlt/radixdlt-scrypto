use crate::blueprints::event_store::EventStoreNativePackage;
use crate::blueprints::resource::NonFungibleSubstate;
use crate::errors::RuntimeError;
use crate::errors::{KernelError, SystemError};
use crate::kernel::actor::ActorIdentifier;
use crate::kernel::kernel::Kernel;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::kernel::kernel_api::{Invokable, KernelInternalApi};
use crate::kernel::module::KernelModule;
use crate::kernel::module_mixer::KernelModuleMixer;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::system::node_modules::access_rules::MethodAccessRulesChainSubstate;
use crate::system::node_modules::metadata::MetadataSubstate;
use crate::system::node_substates::RuntimeSubstate;
use crate::types::*;
use crate::wasm::WasmEngine;
use native_sdk::resource::ResourceManager;
use radix_engine_interface::api::component::{
    ComponentRoyaltyAccumulatorSubstate, ComponentRoyaltyConfigSubstate, ComponentStateSubstate,
    KeyValueStoreEntrySubstate, TypeInfoSubstate,
};
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::unsafe_api::ClientCostingReason;
use radix_engine_interface::api::{package::*, ClientEventApi};
use radix_engine_interface::api::{
    ClientActorApi, ClientApi, ClientComponentApi, ClientNodeApi, ClientPackageApi,
    ClientSubstateApi, ClientUnsafeApi,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::constants::RADIX_TOKEN;
use radix_engine_interface::data::model::Own;
use radix_engine_interface::data::*;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;

impl<'g, 's, W> ClientNodeApi<RuntimeError> for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn sys_drop_node(&mut self, node_id: RENodeId) -> Result<(), RuntimeError> {
        self.kernel_drop_node(node_id)?;
        Ok(())
    }
}

impl<'g, 's, W> ClientSubstateApi<RuntimeError> for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn sys_lock_substate(
        &mut self,
        node_id: RENodeId,
        offset: SubstateOffset,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError> {
        if flags.contains(LockFlags::UNMODIFIED_BASE) || flags.contains(LockFlags::FORCE_WRITE) {
            if !matches!(node_id, RENodeId::Vault(_)) {
                return Err(RuntimeError::SystemError(SystemError::InvalidLockFlags));
            }
        }

        let module_id = if let ActorIdentifier::Method(method) =
            self.kernel_get_current_actor().unwrap().identifier
        {
            method.1
        } else {
            // TODO: Remove this
            NodeModuleId::SELF
        };

        self.kernel_lock_substate(node_id, module_id, offset, flags)
    }

    fn sys_read_substate(&mut self, lock_handle: LockHandle) -> Result<Vec<u8>, RuntimeError> {
        self.kernel_read_substate(lock_handle).map(|v| v.into())
    }

    fn sys_write_substate(
        &mut self,
        lock_handle: LockHandle,
        buffer: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        let offset = self.kernel_get_lock_info(lock_handle)?.offset;
        let substate = RuntimeSubstate::decode_from_buffer(&offset, &buffer)?;

        match substate {
            RuntimeSubstate::ComponentState(next) => {
                let state: &mut ComponentStateSubstate =
                    self.kernel_get_substate_ref_mut(lock_handle)?;
                *state = next
            }
            RuntimeSubstate::KeyValueStoreEntry(next) => {
                let entry: &mut KeyValueStoreEntrySubstate =
                    self.kernel_get_substate_ref_mut(lock_handle)?;
                *entry = next;
            }
            RuntimeSubstate::NonFungible(next) => {
                let non_fungible: &mut NonFungibleSubstate =
                    self.kernel_get_substate_ref_mut(lock_handle)?;
                *non_fungible = next;
            }
            _ => return Err(RuntimeError::KernelError(KernelError::InvalidOverwrite)),
        }

        Ok(())
    }

    fn sys_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError> {
        self.kernel_drop_lock(lock_handle)
    }
}

impl<'g, 's, W> ClientActorApi<RuntimeError> for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn get_fn_identifier(&mut self) -> Result<FnIdentifier, RuntimeError> {
        Ok(self.kernel_get_current_actor().unwrap().fn_identifier)
    }
}

impl<'g, 's, W> ClientPackageApi<RuntimeError> for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn new_package(
        &mut self,
        code: Vec<u8>,
        abi: BTreeMap<String, BlueprintAbi>,
        access_rules: AccessRules,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
        metadata: BTreeMap<String, String>,
    ) -> Result<PackageAddress, RuntimeError> {
        let result = self.call_function(
            PACKAGE_LOADER,
            PACKAGE_LOADER_BLUEPRINT,
            PACKAGE_LOADER_PUBLISH_WASM_IDENT,
            scrypto_encode(&PackageLoaderPublishWasmInput {
                package_address: None,
                code,
                abi,
                access_rules,
                royalty_config,
                metadata,
            })
            .unwrap(),
        )?;

        let package_address: PackageAddress = scrypto_decode(&result).unwrap();
        Ok(package_address)
    }

    fn call_function(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let invocation = FunctionInvocation {
            fn_identifier: FnIdentifier::new(
                package_address,
                blueprint_name.to_string(),
                function_name.to_string(),
            ),
            args,
        };

        self.kernel_invoke(invocation)
            .map(|v| scrypto_encode(&v).expect("Failed to encode scrypto fn return"))
    }

    fn get_code(&mut self, package_address: PackageAddress) -> Result<PackageCode, RuntimeError> {
        let handle = self.kernel_lock_substate(
            RENodeId::GlobalPackage(package_address),
            NodeModuleId::SELF,
            SubstateOffset::Package(PackageOffset::Code),
            LockFlags::read_only(),
        )?;
        let package: &PackageCodeSubstate = self.kernel_get_substate_ref(handle)?;
        let code = package.code().to_vec();
        self.kernel_drop_lock(handle)?;
        Ok(PackageCode::Wasm(code))
    }

    fn get_abi(
        &mut self,
        package_address: PackageAddress,
    ) -> Result<BTreeMap<String, BlueprintAbi>, RuntimeError> {
        let handle = self.kernel_lock_substate(
            RENodeId::GlobalPackage(package_address),
            NodeModuleId::SELF,
            SubstateOffset::Package(PackageOffset::Info),
            LockFlags::read_only(),
        )?;
        let package: &PackageInfoSubstate = self.kernel_get_substate_ref(handle)?;
        let abi = package.blueprint_abis.clone();
        self.kernel_drop_lock(handle)?;
        Ok(abi)
    }
}

impl<'g, 's, W> ClientComponentApi<RuntimeError> for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn new_component(
        &mut self,
        blueprint_ident: &str,
        app_states: BTreeMap<u8, Vec<u8>>,
        access_rules_chain: Vec<AccessRules>,
        royalty_config: RoyaltyConfig,
        metadata: BTreeMap<String, String>,
    ) -> Result<ComponentId, RuntimeError> {
        // Allocate node id
        let node_id = self.kernel_allocate_node_id(RENodeType::Component)?;

        // Create a royalty vault
        let royalty_vault_id = ResourceManager(RADIX_TOKEN).new_vault(self)?.vault_id();

        // Create royalty substates
        let royalty_config_substate = ComponentRoyaltyConfigSubstate { royalty_config };
        let royalty_accumulator_substate = ComponentRoyaltyAccumulatorSubstate {
            royalty: Own::Vault(royalty_vault_id.into()),
        };

        // Create metadata substates
        let metadata_substate = MetadataSubstate { metadata };

        // Create auth substates
        let auth_substate = MethodAccessRulesChainSubstate { access_rules_chain };

        // Create component RENode
        // FIXME: support native blueprints
        let package_address = self
            .kernel_get_current_actor()
            .unwrap()
            .fn_identifier
            .package_address();

        let blueprint_ident = blueprint_ident.to_string();
        // FIXME: generalize app substates;
        // FIXME: remove unwrap;
        // FIXME: support native blueprints
        let abi_enforced_app_substate = app_states.into_iter().next().unwrap().1;

        self.kernel_create_node(
            node_id,
            RENodeInit::Component(ComponentStateSubstate::new(abi_enforced_app_substate)),
            btreemap!(
                NodeModuleId::TypeInfo => RENodeModuleInit::TypeInfo(
                    TypeInfoSubstate::new(package_address, blueprint_ident.to_string())
                ),
                NodeModuleId::ComponentRoyalty => RENodeModuleInit::ComponentRoyalty(
                    royalty_config_substate,
                    royalty_accumulator_substate
                ),
                NodeModuleId::Metadata => RENodeModuleInit::Metadata(metadata_substate),
                NodeModuleId::AccessRules => RENodeModuleInit::ObjectAccessRulesChain(auth_substate),
            ),
        )?;

        Ok(node_id.into())
    }

    fn globalize(&mut self, node_id: RENodeId) -> Result<ComponentAddress, RuntimeError> {
        let node_type = match node_id {
            RENodeId::Component(..) => RENodeType::GlobalComponent,
            RENodeId::Identity(..) => RENodeType::GlobalIdentity,
            RENodeId::Validator(..) => RENodeType::GlobalValidator,
            RENodeId::EpochManager(..) => RENodeType::GlobalEpochManager,
            RENodeId::Clock(..) => RENodeType::GlobalClock,
            RENodeId::Account(..) => RENodeType::GlobalAccount,
            RENodeId::AccessController(..) => RENodeType::GlobalAccessController,
            _ => return Err(RuntimeError::SystemError(SystemError::CannotGlobalize)),
        };

        let global_node_id = self.kernel_allocate_node_id(node_type)?;
        self.globalize_with_address(node_id, global_node_id.into())
    }

    fn globalize_with_address(
        &mut self,
        node_id: RENodeId,
        address: Address,
    ) -> Result<ComponentAddress, RuntimeError> {
        let node = self.kernel_drop_node(node_id)?;

        let mut module_substates = BTreeMap::new();
        let mut component_substates = BTreeMap::new();
        for ((node_module_id, offset), substate) in node.substates {
            match node_module_id {
                NodeModuleId::SELF => component_substates.insert(offset, substate),
                _ => module_substates.insert((node_module_id, offset), substate),
            };
        }

        let mut module_init = BTreeMap::new();

        let type_info = module_substates
            .remove(&(
                NodeModuleId::TypeInfo,
                SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
            ))
            .unwrap();
        let type_info_substate: TypeInfoSubstate = type_info.into();
        module_init.insert(
            NodeModuleId::TypeInfo,
            RENodeModuleInit::TypeInfo(type_info_substate),
        );

        if let Some(access_rules) = module_substates.remove(&(
            NodeModuleId::AccessRules,
            SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain),
        )) {
            let access_rules_substate: MethodAccessRulesChainSubstate = access_rules.into();
            module_init.insert(
                NodeModuleId::AccessRules,
                RENodeModuleInit::ObjectAccessRulesChain(access_rules_substate),
            );
        }
        if let Some(royalty_config) = module_substates.remove(&(
            NodeModuleId::ComponentRoyalty,
            SubstateOffset::Royalty(RoyaltyOffset::RoyaltyConfig),
        )) {
            let royalty_config_substate: ComponentRoyaltyConfigSubstate = royalty_config.into();
            let royalty_accumulator = module_substates
                .remove(&(
                    NodeModuleId::ComponentRoyalty,
                    SubstateOffset::Royalty(RoyaltyOffset::RoyaltyAccumulator),
                ))
                .unwrap();
            let royalty_accumulator_substate: ComponentRoyaltyAccumulatorSubstate =
                royalty_accumulator.into();
            module_init.insert(
                NodeModuleId::ComponentRoyalty,
                RENodeModuleInit::ComponentRoyalty(
                    royalty_config_substate,
                    royalty_accumulator_substate,
                ),
            );
        }

        if let Some(metadata) = module_substates.remove(&(
            NodeModuleId::Metadata,
            SubstateOffset::Metadata(MetadataOffset::Metadata),
        )) {
            let metadata_substate: MetadataSubstate = metadata.into();
            module_init.insert(
                NodeModuleId::Metadata,
                RENodeModuleInit::Metadata(metadata_substate),
            );
        }

        self.kernel_create_node(
            address.into(),
            RENodeInit::GlobalComponent(component_substates),
            module_init,
        )?;

        Ok(address.into())
    }

    fn call_method(
        &mut self,
        receiver: RENodeId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        self.call_module_method(receiver, NodeModuleId::SELF, method_name, args)
    }

    fn call_module_method(
        &mut self,
        receiver: RENodeId,
        node_module_id: NodeModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let invocation = MethodInvocation {
            identifier: MethodIdentifier(receiver, node_module_id, method_name.to_string()),
            args,
        };

        self.kernel_invoke(invocation)
            .map(|v| scrypto_encode(&v).expect("Failed to encode scrypto fn return"))
    }

    fn get_component_type_info(
        &mut self,
        node_id: RENodeId,
    ) -> Result<(PackageAddress, String), RuntimeError> {
        // TODO: Use Node Module method instead of direct access
        let handle = self.kernel_lock_substate(
            node_id,
            NodeModuleId::TypeInfo,
            SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
            LockFlags::read_only(),
        )?;
        let info: &TypeInfoSubstate = self.kernel_get_substate_ref(handle)?;
        let package_address = info.package_address;
        let blueprint_ident = info.blueprint_name.clone();
        self.kernel_drop_lock(handle)?;
        Ok((package_address, blueprint_ident))
    }

    fn new_key_value_store(&mut self) -> Result<KeyValueStoreId, RuntimeError> {
        let node_id = self.kernel_allocate_node_id(RENodeType::KeyValueStore)?;

        self.kernel_create_node(node_id, RENodeInit::KeyValueStore, btreemap!())?;

        Ok(node_id.into())
    }
}

impl<'g, 's, W> ClientUnsafeApi<RuntimeError> for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn consume_cost_units(
        &mut self,
        units: u32,
        reason: ClientCostingReason,
    ) -> Result<(), RuntimeError> {
        KernelModuleMixer::on_consume_cost_units(self, units, reason)
    }

    fn credit_cost_units(
        &mut self,
        vault_id: VaultId,
        locked_fee: LiquidFungibleResource,
        contingent: bool,
    ) -> Result<LiquidFungibleResource, RuntimeError> {
        KernelModuleMixer::on_credit_cost_units(self, vault_id, locked_fee, contingent)
    }

    fn update_instruction_index(&mut self, new_index: usize) -> Result<(), RuntimeError> {
        KernelModuleMixer::on_update_instruction_index(self, new_index)
    }

    fn update_wasm_memory_usage(&mut self, size: usize) -> Result<(), RuntimeError> {
        KernelModuleMixer::on_update_wasm_memory_usage(self, size)
    }
}

impl<'g, 's, W> ClientEventApi<RuntimeError> for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn emit_event<T: ScryptoEncode + abi::LegacyDescribe>(
        &mut self,
        event: T,
    ) -> Result<(), RuntimeError> {
        EventStoreNativePackage::emit_event(event, self)
    }

    fn emit_raw_event(
        &mut self,
        schema_hash: Hash,
        event_data: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        EventStoreNativePackage::emit_raw_event(schema_hash, event_data, self)
    }
}

impl<'g, 's, W> ClientApi<RuntimeError> for Kernel<'g, 's, W> where W: WasmEngine {}
