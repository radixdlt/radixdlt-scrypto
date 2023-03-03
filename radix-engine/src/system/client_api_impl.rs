use crate::blueprints::resource::NonFungibleSubstate;
use crate::errors::{ApplicationError, RuntimeError};
use crate::errors::{KernelError, SystemError};
use crate::kernel::actor::{Actor, ActorIdentifier};
use crate::kernel::kernel::Kernel;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::kernel::kernel_api::{Invokable, KernelInternalApi};
use crate::kernel::module::KernelModule;
use crate::kernel::module_mixer::KernelModuleMixer;
use crate::system::events::EventError;
use crate::system::kernel_modules::costing::FIXED_LOW_FEE;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::system::node_modules::access_rules::MethodAccessRulesSubstate;
use crate::system::node_modules::metadata::MetadataSubstate;
use crate::system::node_modules::type_info::{TypeInfoBlueprint, TypeInfoSubstate};
use crate::system::node_substates::RuntimeSubstate;
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::component::{
    ComponentRoyaltyAccumulatorSubstate, ComponentRoyaltyConfigSubstate, ComponentStateSubstate,
    KeyValueStoreEntrySubstate,
};
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::unsafe_api::ClientCostingReason;
use radix_engine_interface::api::ClientUnsafeApi;
use radix_engine_interface::api::{package::*, ClientEventApi};
use radix_engine_interface::api::{types::*, ClientLoggerApi};
use radix_engine_interface::api::{
    ClientActorApi, ClientApi, ClientComponentApi, ClientNodeApi, ClientPackageApi,
    ClientSubstateApi,
};
use radix_engine_interface::blueprints::logger::Level;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::events::EventTypeIdentifier;
use radix_engine_interface::schema::PackageSchema;
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
        schema: PackageSchema,
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
                schema,
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
}

impl<'g, 's, W> ClientComponentApi<RuntimeError> for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn new_component(
        &mut self,
        blueprint_ident: &str,
        app_states: BTreeMap<u8, Vec<u8>>,
    ) -> Result<ComponentId, RuntimeError> {
        // Allocate node id
        let node_id = self.kernel_allocate_node_id(RENodeType::Component)?;

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

        // TODO: Check that blueprint exists here rather than in kernel

        self.kernel_create_node(
            node_id,
            RENodeInit::Component(ComponentStateSubstate::new(abi_enforced_app_substate)),
            btreemap!(
                NodeModuleId::TypeInfo => RENodeModuleInit::TypeInfo(
                    TypeInfoSubstate::new(package_address, blueprint_ident.to_string(), false)
                ),
            ),
        )?;

        Ok(node_id.into())
    }

    fn globalize(
        &mut self,
        node_id: RENodeId,
        modules: BTreeMap<NodeModuleId, Vec<u8>>,
    ) -> Result<ComponentAddress, RuntimeError> {
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
        self.globalize_with_address(node_id, modules, global_node_id.into())
    }

    fn globalize_with_address(
        &mut self,
        node_id: RENodeId,
        modules: BTreeMap<NodeModuleId, Vec<u8>>,
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
        let mut type_info_substate: TypeInfoSubstate = type_info.into();
        type_info_substate.global = true;
        module_init.insert(
            NodeModuleId::TypeInfo,
            RENodeModuleInit::TypeInfo(type_info_substate),
        );

        // TODO: Check node type matches modules provided

        for (module_id, init) in modules {
            match module_id {
                NodeModuleId::SELF
                | NodeModuleId::TypeInfo
                | NodeModuleId::AccessRules1
                | NodeModuleId::PackageRoyalty
                | NodeModuleId::FunctionAccessRules => {
                    return Err(RuntimeError::SystemError(SystemError::InvalidModule))
                }
                NodeModuleId::AccessRules => {
                    let access_rules: Own = scrypto_decode(&init).map_err(|e| {
                        RuntimeError::SystemError(SystemError::InvalidAccessRules(e))
                    })?;

                    let component_id = access_rules.component_id();
                    let mut node = self.kernel_drop_node(RENodeId::Component(component_id))?;

                    let access_rules = node
                        .substates
                        .remove(&(
                            NodeModuleId::SELF,
                            SubstateOffset::AccessRules(AccessRulesOffset::AccessRules),
                        ))
                        .unwrap();
                    let access_rules: MethodAccessRulesSubstate = access_rules.into();

                    module_init.insert(
                        NodeModuleId::AccessRules,
                        RENodeModuleInit::MethodAccessRules(access_rules),
                    );
                }
                NodeModuleId::Metadata => {
                    let metadata: Own = scrypto_decode(&init)
                        .map_err(|e| RuntimeError::SystemError(SystemError::InvalidMetadata(e)))?;

                    let component_id = metadata.component_id();
                    let mut node = self.kernel_drop_node(RENodeId::Component(component_id))?;

                    let metadata = node
                        .substates
                        .remove(&(
                            NodeModuleId::SELF,
                            SubstateOffset::Metadata(MetadataOffset::Metadata),
                        ))
                        .unwrap();
                    let metadata: MetadataSubstate = metadata.into();

                    module_init
                        .insert(NodeModuleId::Metadata, RENodeModuleInit::Metadata(metadata));
                }
                NodeModuleId::ComponentRoyalty => {
                    let royalty: Own = scrypto_decode(&init).map_err(|e| {
                        RuntimeError::SystemError(SystemError::InvalidRoyaltyConfig(e))
                    })?;

                    let component_id = royalty.component_id();
                    let mut node = self.kernel_drop_node(RENodeId::Component(component_id))?;

                    let config = node
                        .substates
                        .remove(&(
                            NodeModuleId::SELF,
                            SubstateOffset::Royalty(RoyaltyOffset::RoyaltyConfig),
                        ))
                        .unwrap();
                    let config: ComponentRoyaltyConfigSubstate = config.into();
                    let accumulator = node
                        .substates
                        .remove(&(
                            NodeModuleId::SELF,
                            SubstateOffset::Royalty(RoyaltyOffset::RoyaltyAccumulator),
                        ))
                        .unwrap();
                    let accumulator: ComponentRoyaltyAccumulatorSubstate = accumulator.into();

                    module_init.insert(
                        NodeModuleId::ComponentRoyalty,
                        RENodeModuleInit::ComponentRoyalty(config, accumulator),
                    );
                }
            }
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
        TypeInfoBlueprint::get_type(node_id, self)
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
    fn emit_raw_event(
        &mut self,
        schema_hash: Hash,
        event_data: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        // Costing event emission.
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

        // Construct the event type identifier based on the current actor
        let event_type_id = match self.kernel_get_current_actor() {
            Some(Actor {
                identifier: ActorIdentifier::Method(MethodIdentifier(node_id, node_module_id, ..)),
                ..
            }) => Ok(EventTypeIdentifier(node_id, node_module_id, schema_hash)),
            Some(Actor {
                identifier:
                    ActorIdentifier::Function(FnIdentifier {
                        package_address, ..
                    }),
                ..
            }) => Ok(EventTypeIdentifier(
                RENodeId::GlobalPackage(package_address),
                NodeModuleId::SELF,
                schema_hash,
            )),
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::EventError(EventError::InvalidActor),
            )),
        }?;

        // TODO: Validate that the event schema matches that given by the event schema hash.
        // Need to wait for David's PR for schema validation and move away from LegacyDescribe
        // over to new Describe.

        // NOTE: We need to ensure that the event being emitted is an SBOR struct or an enum,
        // this is not done here, this should be done at event registration time. Thus, if the
        // event has been successfully registered, it can be emitted (from a schema POV).

        // Adding the event to the event store
        self.kernel_get_module_state()
            .events
            .add_event(event_type_id, event_data);

        Ok(())
    }
}

impl<'g, 's, W> ClientLoggerApi<RuntimeError> for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn log_message(&mut self, level: Level, message: String) -> Result<(), RuntimeError> {
        self.kernel_get_module_state()
            .logger
            .add_log(level, message);
        Ok(())
    }
}

impl<'g, 's, W> ClientApi<RuntimeError> for Kernel<'g, 's, W> where W: WasmEngine {}
