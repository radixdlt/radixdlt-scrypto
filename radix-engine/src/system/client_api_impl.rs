use crate::blueprints::resource::*;
use crate::errors::{ApplicationError, RuntimeError, SubstateValidationError};
use crate::errors::{KernelError, SystemError};
use crate::kernel::actor::{Actor, ActorIdentifier};
use crate::kernel::kernel::Kernel;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::kernel::kernel_api::{KernelInternalApi, KernelInvokeApi};
use crate::kernel::module::KernelModule;
use crate::kernel::module_mixer::KernelModuleMixer;
use crate::system::kernel_modules::costing::FIXED_LOW_FEE;
use crate::system::kernel_modules::events::EventError;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::system::node_modules::access_rules::MethodAccessRulesSubstate;
use crate::system::node_modules::type_info::{TypeInfoBlueprint, TypeInfoSubstate};
use crate::system::node_substates::RuntimeSubstate;
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::component::{
    ComponentRoyaltyAccumulatorSubstate, ComponentRoyaltyConfigSubstate, ComponentStateSubstate,
    KeyValueStoreEntrySubstate,
};
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::node_modules::metadata::*;
use radix_engine_interface::api::node_modules::royalty::*;
use radix_engine_interface::api::package::*;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::types::Level;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::unsafe_api::ClientCostingReason;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::schema::PackageSchema;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;

use super::node_modules::event_schema::PackageEventSchemaSubstate;

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
            let (package_address, blueprint) = self.get_object_type_info(node_id)?;
            if !matches!(
                (package_address, blueprint.as_str()),
                (RESOURCE_MANAGER_PACKAGE, VAULT_BLUEPRINT)
            ) {
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

        // TODO: support all self substates
        // TODO: add payload schema validation

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
        let info = self.kernel_get_lock_info(lock_handle)?;
        if info.flags.contains(LockFlags::MUTABLE) {}

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
        access_rules: AccessRulesConfig,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
        metadata: BTreeMap<String, String>,
        event_schema: BTreeMap<String, Vec<(LocalTypeIndex, Schema<ScryptoCustomTypeExtension>)>>,
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
                event_schema,
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

        self.kernel_invoke(invocation).map(|v| v.into())
    }
}

impl<'g, 's, W> ClientObjectApi<RuntimeError> for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn new_object(
        &mut self,
        blueprint_ident: &str,
        mut app_states: Vec<Vec<u8>>,
    ) -> Result<ObjectId, RuntimeError> {
        let package_address = self
            .kernel_get_current_actor()
            .unwrap()
            .fn_identifier
            .package_address();

        let handle = self.kernel_lock_substate(
            RENodeId::GlobalObject(package_address.into()),
            NodeModuleId::SELF,
            SubstateOffset::Package(PackageOffset::Info),
            LockFlags::read_only(),
        )?;
        let package: &PackageInfoSubstate = self.kernel_get_substate_ref(handle)?;
        let schema =
            package
                .schema
                .blueprints
                .get(blueprint_ident)
                .ok_or(RuntimeError::SystemError(
                    SystemError::SubstateValidationError(
                        SubstateValidationError::BlueprintNotFound(blueprint_ident.to_string()),
                    ),
                ))?;
        if schema.substates.len() != app_states.len() {
            return Err(RuntimeError::SystemError(
                SystemError::SubstateValidationError(
                    SubstateValidationError::WrongNumberOfSubstates(
                        blueprint_ident.to_string(),
                        app_states.len(),
                        schema.substates.len(),
                    ),
                ),
            ));
        }
        for i in 0..app_states.len() {
            validate_payload_against_schema(&app_states[i], &schema.schema, schema.substates[i])
                .map_err(|e| {
                    // TODO: make `LocatedValidationError` encodable
                    RuntimeError::SystemError(SystemError::SubstateValidationError(
                        SubstateValidationError::SchemaValidationError(
                            blueprint_ident.to_string(),
                            format!("{:?}", e),
                        ),
                    ))
                })?;
        }
        self.kernel_drop_lock(handle)?;

        struct SubstateSchemaParser<'a> {
            next_index: usize,
            app_states: &'a Vec<Vec<u8>>,
        }

        impl<'a> SubstateSchemaParser<'a> {
            fn new(app_states: &'a Vec<Vec<u8>>) -> Self {
                Self {
                    next_index: 0,
                    app_states,
                }
            }

            fn decode_next<S: ScryptoDecode>(&mut self) -> S {
                if let Some(substate_bytes) = self.app_states.get(self.next_index) {
                    let decoded = scrypto_decode(substate_bytes)
                        .expect("Unexpected decode error for app states");
                    self.next_index = self.next_index + 1;
                    decoded
                } else {
                    panic!("Unexpected missing app states");
                }
            }

            fn end(self) {
                if self.app_states.get(self.next_index).is_some() {
                    panic!("Unexpected extra app states");
                }
            }
        }

        let mut parser = SubstateSchemaParser::new(&mut app_states);
        let node_init = match package_address {
            RESOURCE_MANAGER_PACKAGE => match blueprint_ident {
                RESOURCE_MANAGER_BLUEPRINT => RENodeInit::Object(btreemap!(
                    SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager) => RuntimeSubstate::ResourceManager(parser.decode_next())
                )),
                PROOF_BLUEPRINT => RENodeInit::Object(btreemap!(
                    SubstateOffset::Proof(ProofOffset::Info) => RuntimeSubstate::ProofInfo(parser.decode_next()),
                    SubstateOffset::Proof(ProofOffset::Fungible) => RuntimeSubstate::FungibleProof(parser.decode_next()),
                    SubstateOffset::Proof(ProofOffset::NonFungible) => RuntimeSubstate::NonFungibleProof(parser.decode_next()),
                )),
                BUCKET_BLUEPRINT => RENodeInit::Object(btreemap!(
                    SubstateOffset::Bucket(BucketOffset::Info) => RuntimeSubstate::BucketInfo(parser.decode_next()),
                    SubstateOffset::Bucket(BucketOffset::LiquidFungible) => RuntimeSubstate::BucketLiquidFungible(parser.decode_next()),
                    SubstateOffset::Bucket(BucketOffset::LockedFungible) => RuntimeSubstate::BucketLockedFungible(parser.decode_next()),
                    SubstateOffset::Bucket(BucketOffset::LiquidNonFungible) => RuntimeSubstate::BucketLiquidNonFungible(parser.decode_next()),
                    SubstateOffset::Bucket(BucketOffset::LockedNonFungible) => RuntimeSubstate::BucketLockedNonFungible(parser.decode_next()),
                )),
                VAULT_BLUEPRINT => RENodeInit::Object(btreemap!(
                    SubstateOffset::Vault(VaultOffset::Info) => RuntimeSubstate::VaultInfo(parser.decode_next()),
                    SubstateOffset::Vault(VaultOffset::LiquidFungible) => RuntimeSubstate::VaultLiquidFungible(parser.decode_next()),
                    SubstateOffset::Vault(VaultOffset::LockedFungible) => RuntimeSubstate::VaultLockedFungible(parser.decode_next()),
                    SubstateOffset::Vault(VaultOffset::LiquidNonFungible) => RuntimeSubstate::VaultLiquidNonFungible(parser.decode_next()),
                    SubstateOffset::Vault(VaultOffset::LockedNonFungible) => RuntimeSubstate::VaultLockedNonFungible(parser.decode_next()),
                )),
                blueprint => panic!("Unexpected blueprint {}", blueprint),
            },
            METADATA_PACKAGE => RENodeInit::Object(btreemap!()),
            ROYALTY_PACKAGE => match blueprint_ident {
                COMPONENT_ROYALTY_BLUEPRINT => RENodeInit::Object(btreemap!(
                    SubstateOffset::Royalty(RoyaltyOffset::RoyaltyConfig) => RuntimeSubstate::ComponentRoyaltyConfig(parser.decode_next()),
                    SubstateOffset::Royalty(RoyaltyOffset::RoyaltyAccumulator) => RuntimeSubstate::ComponentRoyaltyAccumulator(parser.decode_next())
                )),
                blueprint => panic!("Unexpected blueprint {}", blueprint),
            },
            ACCESS_RULES_PACKAGE => RENodeInit::Object(btreemap!(
                SubstateOffset::AccessRules(AccessRulesOffset::AccessRules) => RuntimeSubstate::MethodAccessRules(parser.decode_next())
            )),
            EPOCH_MANAGER_PACKAGE => match blueprint_ident {
                VALIDATOR_BLUEPRINT => RENodeInit::Object(btreemap!(
                    SubstateOffset::Validator(ValidatorOffset::Validator) => RuntimeSubstate::Validator(parser.decode_next())
                )),
                EPOCH_MANAGER_BLUEPRINT => RENodeInit::Object(btreemap!(
                    SubstateOffset::EpochManager(EpochManagerOffset::EpochManager) => RuntimeSubstate::EpochManager(parser.decode_next()),
                    SubstateOffset::EpochManager(EpochManagerOffset::CurrentValidatorSet) => RuntimeSubstate::ValidatorSet(parser.decode_next()),
                    SubstateOffset::EpochManager(EpochManagerOffset::PreparingValidatorSet) => RuntimeSubstate::ValidatorSet(parser.decode_next())
                )),
                blueprint => panic!("Unexpected blueprint {}", blueprint),
            },
            ACCESS_CONTROLLER_PACKAGE => RENodeInit::Object(btreemap!(
                SubstateOffset::AccessController(AccessControllerOffset::AccessController)
                    => RuntimeSubstate::AccessController(parser.decode_next())
            )),
            IDENTITY_PACKAGE => RENodeInit::Object(btreemap!()),
            ACCOUNT_PACKAGE => RENodeInit::Object(btreemap!(
                SubstateOffset::Account(AccountOffset::Account)
                    => RuntimeSubstate::Account(parser.decode_next())
            )),
            CLOCK_PACKAGE => RENodeInit::Object(btreemap!(
                SubstateOffset::Clock(ClockOffset::CurrentTimeRoundedToMinutes)
                    => RuntimeSubstate::CurrentTimeRoundedToMinutes(parser.decode_next())
            )),
            _ => RENodeInit::Object(btreemap!(
                SubstateOffset::Component(ComponentOffset::State0) => RuntimeSubstate::ComponentState(
                    ComponentStateSubstate::new(scrypto_encode(&parser.decode_next::<ScryptoValue>()).unwrap())
                )
            )),
        };
        parser.end();

        let node_id = self.kernel_allocate_node_id(RENodeType::Object)?;

        self.kernel_create_node(
            node_id,
            node_init,
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
        modules: BTreeMap<NodeModuleId, ObjectId>,
    ) -> Result<Address, RuntimeError> {
        let node_type = match node_id {
            RENodeId::Object(..) => {
                let (package_address, blueprint) = TypeInfoBlueprint::get_type(node_id, self)?;
                match (package_address, blueprint.as_str()) {
                    (ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT) => RENodeType::GlobalAccount,
                    (IDENTITY_PACKAGE, IDENTITY_BLUEPRINT) => RENodeType::GlobalIdentity,
                    (ACCESS_CONTROLLER_PACKAGE, ACCESS_CONTROLLER_BLUEPRINT) => {
                        RENodeType::GlobalAccessController
                    }
                    (EPOCH_MANAGER_PACKAGE, VALIDATOR_BLUEPRINT) => RENodeType::GlobalValidator,
                    _ => RENodeType::GlobalComponent,
                }
            }
            _ => return Err(RuntimeError::SystemError(SystemError::CannotGlobalize)),
        };

        let global_node_id = self.kernel_allocate_node_id(node_type)?;
        self.globalize_with_address(node_id, modules, global_node_id.into())
    }

    fn globalize_with_address(
        &mut self,
        node_id: RENodeId,
        modules: BTreeMap<NodeModuleId, ObjectId>,
        address: Address,
    ) -> Result<Address, RuntimeError> {
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

        for (module_id, object_id) in modules {
            match module_id {
                NodeModuleId::SELF
                | NodeModuleId::TypeInfo
                | NodeModuleId::PackageRoyalty
                | NodeModuleId::FunctionAccessRules
                | NodeModuleId::PackageEventSchema => {
                    return Err(RuntimeError::SystemError(SystemError::InvalidModule))
                }
                NodeModuleId::AccessRules | NodeModuleId::AccessRules1 => {
                    let node_id = RENodeId::Object(object_id);
                    let (package_address, blueprint) = self.get_object_type_info(node_id)?;
                    if !matches!(
                        (package_address, blueprint.as_str()),
                        (ACCESS_RULES_PACKAGE, ACCESS_RULES_BLUEPRINT)
                    ) {
                        return Err(RuntimeError::SystemError(SystemError::InvalidModuleType {
                            expected_package: ACCESS_RULES_PACKAGE,
                            expected_blueprint: ACCOUNT_BLUEPRINT.to_string(),
                            actual_package: package_address,
                            actual_blueprint: blueprint,
                        }));
                    }

                    let mut node = self.kernel_drop_node(RENodeId::Object(object_id))?;

                    let access_rules = node
                        .substates
                        .remove(&(
                            NodeModuleId::SELF,
                            SubstateOffset::AccessRules(AccessRulesOffset::AccessRules),
                        ))
                        .unwrap();
                    let access_rules: MethodAccessRulesSubstate = access_rules.into();

                    module_init
                        .insert(module_id, RENodeModuleInit::MethodAccessRules(access_rules));
                }
                NodeModuleId::Metadata => {
                    let node_id = RENodeId::Object(object_id);
                    let (package_address, blueprint) = self.get_object_type_info(node_id)?;
                    if !matches!(
                        (package_address, blueprint.as_str()),
                        (METADATA_PACKAGE, METADATA_BLUEPRINT)
                    ) {
                        return Err(RuntimeError::SystemError(SystemError::InvalidModuleType {
                            expected_package: METADATA_PACKAGE,
                            expected_blueprint: METADATA_BLUEPRINT.to_string(),
                            actual_package: package_address,
                            actual_blueprint: blueprint,
                        }));
                    }

                    let node = self.kernel_drop_node(node_id)?;

                    let mut substates = BTreeMap::new();
                    for ((module_id, offset), substate) in node.substates {
                        if let NodeModuleId::SELF = module_id {
                            substates.insert(offset, substate);
                        }
                    }

                    module_init.insert(
                        NodeModuleId::Metadata,
                        RENodeModuleInit::Metadata(substates),
                    );
                }
                NodeModuleId::ComponentRoyalty => {
                    let node_id = RENodeId::Object(object_id);
                    let (package_address, blueprint) = self.get_object_type_info(node_id)?;
                    if !matches!(
                        (package_address, blueprint.as_str()),
                        (ROYALTY_PACKAGE, COMPONENT_ROYALTY_BLUEPRINT)
                    ) {
                        return Err(RuntimeError::SystemError(SystemError::InvalidModuleType {
                            expected_package: ROYALTY_PACKAGE,
                            expected_blueprint: COMPONENT_ROYALTY_BLUEPRINT.to_string(),
                            actual_package: package_address,
                            actual_blueprint: blueprint,
                        }));
                    }

                    let mut node = self.kernel_drop_node(node_id)?;

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
            RENodeInit::GlobalObject(component_substates),
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

        self.kernel_invoke(invocation).map(|v| v.into())
    }

    fn get_object_type_info(
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
        vault_id: ObjectId,
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
    fn emit_event(&mut self, event_name: String, event_data: Vec<u8>) -> Result<(), RuntimeError> {
        // Costing event emission.
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

        // Construct the event type identifier based on the current actor
        let (event_type_id, package_address, blueprint_name) = match self.kernel_get_current_actor()
        {
            Some(Actor {
                identifier: ActorIdentifier::Method(MethodIdentifier(node_id, node_module_id, ..)),
                ..
            }) => {
                let event_type_id = EventTypeIdentifier(
                    Emitter::Method(node_id, node_module_id),
                    event_name.clone(),
                );
                let (package_address, blueprint_name) = match node_module_id {
                    NodeModuleId::AccessRules | NodeModuleId::AccessRules1 => {
                        Ok((ACCESS_RULES_PACKAGE, ACCESS_RULES_BLUEPRINT.into()))
                    }
                    NodeModuleId::ComponentRoyalty => {
                        Ok((ROYALTY_PACKAGE, COMPONENT_ROYALTY_BLUEPRINT.into()))
                    }
                    NodeModuleId::PackageRoyalty => {
                        Ok((ROYALTY_PACKAGE, PACKAGE_ROYALTY_BLUEPRINT.into()))
                    }
                    NodeModuleId::FunctionAccessRules => {
                        Ok((ACCESS_RULES_PACKAGE, FUNCTION_ACCESS_RULES_BLUEPRINT.into()))
                    }
                    NodeModuleId::Metadata => Ok((METADATA_PACKAGE, METADATA_BLUEPRINT.into())),
                    NodeModuleId::SELF => self.get_object_type_info(node_id),
                    NodeModuleId::TypeInfo | NodeModuleId::PackageEventSchema => {
                        Err(RuntimeError::ApplicationError(
                            ApplicationError::EventError(EventError::NoAssociatedPackage),
                        ))
                    }
                }?;

                Ok((event_type_id, package_address, blueprint_name.to_owned()))
            }
            Some(Actor {
                identifier:
                    ActorIdentifier::Function(FnIdentifier {
                        package_address,
                        blueprint_name,
                        ..
                    }),
                ..
            }) => Ok((
                EventTypeIdentifier(
                    Emitter::Function(
                        RENodeId::GlobalObject(Address::Package(package_address)),
                        NodeModuleId::SELF,
                        blueprint_name.clone(),
                    ),
                    event_name.clone(),
                ),
                package_address,
                blueprint_name.to_owned(),
            )),
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::EventError(EventError::InvalidActor),
            )),
        }?;

        // Reading the schema to validate the payload against it
        let (local_type_index, schema) = {
            let handle = self.kernel_lock_substate(
                RENodeId::GlobalObject(Address::Package(package_address)),
                NodeModuleId::PackageEventSchema,
                SubstateOffset::PackageEventSchema(PackageEventSchemaOffset::PackageEventSchema),
                LockFlags::read_only(),
            )?;
            let package_schema =
                self.kernel_get_substate_ref::<PackageEventSchemaSubstate>(handle)?;
            let contained_schema = package_schema
                .0
                .get(&blueprint_name)
                .and_then(|blueprint_schema| blueprint_schema.get(&event_name))
                .map_or(
                    Err(RuntimeError::ApplicationError(
                        ApplicationError::EventError(EventError::SchemaNotFoundError {
                            package_address,
                            blueprint_name,
                            event_name,
                        }),
                    )),
                    |item| Ok(item.clone()),
                )?;
            self.kernel_drop_lock(handle)?;
            contained_schema
        };

        // Validating the event data against the event schema
        validate_payload_against_schema(&event_data, &schema, local_type_index).map_err(|_| {
            RuntimeError::ApplicationError(ApplicationError::EventError(
                EventError::InvalidEventSchema,
            ))
        })?;

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
