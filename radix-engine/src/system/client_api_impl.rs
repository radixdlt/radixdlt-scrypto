use crate::errors::SystemError;
use crate::errors::{
    ApplicationError, InvalidModuleSet, InvalidModuleType, RuntimeError, SubstateValidationError,
};
use crate::kernel::actor::{Actor, ExecutionMode};
use crate::kernel::kernel::Kernel;
use crate::kernel::kernel_api::*;
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
};
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::node_modules::metadata::*;
use radix_engine_interface::api::node_modules::royalty::*;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::types::ClientCostingReason;
use radix_engine_interface::api::types::Level;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::*;
use radix_engine_interface::api::object_api::ClientIterableMapApi;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::schema::{IterableMapSchema, KeyValueStoreSchema};
use resources_tracker_macro::trace_resources;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;

use super::kernel_modules::auth::{convert_contextless, Authentication};
use super::kernel_modules::costing::CostingReason;

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
        // TODO: Remove
        if flags.contains(LockFlags::UNMODIFIED_BASE) || flags.contains(LockFlags::FORCE_WRITE) {
            let info = self.get_object_info(node_id)?;
            if !matches!(
                (
                    info.blueprint.package_address,
                    info.blueprint.blueprint_name.as_str()
                ),
                (RESOURCE_MANAGER_PACKAGE, FUNGIBLE_VAULT_BLUEPRINT)
            ) {
                return Err(RuntimeError::SystemError(SystemError::InvalidLockFlags));
            }
        }

        let module_id =
            if let Actor::Method { module_id, .. } = self.kernel_get_current_actor().unwrap() {
                module_id
            } else {
                // TODO: Remove this
                NodeModuleId::SELF
            };

        self.kernel_lock_substate(&node_id, module_id, offset, flags)
    }

    fn sys_read_substate(&mut self, lock_handle: LockHandle) -> Result<Vec<u8>, RuntimeError> {
        self.kernel_read_substate(lock_handle).map(|v| v.into())
    }

    fn sys_write_substate(
        &mut self,
        lock_handle: LockHandle,
        buffer: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        let LockInfo {
            node_id,
            module_id,
            offset,
            ..
        } = self.kernel_get_lock_info(lock_handle)?;

        if module_id.eq(&NodeModuleId::SELF) {
            let type_info = TypeInfoBlueprint::get_type(&node_id, self)?;
            match type_info {
                TypeInfoSubstate::KeyValueStore(schema) => {
                    validate_payload_against_schema(&buffer, &schema.schema, schema.value)
                        .map_err(|_| {
                            RuntimeError::SystemError(SystemError::InvalidSubstateWrite)
                        })?;

                    if !schema.can_own {
                        let indexed = IndexedScryptoValue::from_slice(&buffer).map_err(|_| {
                            RuntimeError::SystemError(SystemError::InvalidSubstateWrite)
                        })?;
                        let (_, own, _) = indexed.unpack();
                        if !own.is_empty() {
                            return Err(RuntimeError::SystemError(
                                SystemError::InvalidKeyValueStoreOwnership,
                            ));
                        }
                    }
                }
                _ => {}
            }
        }

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
                let entry: &mut Option<ScryptoValue> =
                    self.kernel_get_substate_ref_mut(lock_handle)?;
                *entry = next;
            }
            _ => return Err(RuntimeError::SystemError(SystemError::InvalidSubstateWrite)),
        }

        Ok(())
    }

    fn sys_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError> {
        let info = self.kernel_get_lock_info(lock_handle)?;
        if info.flags.contains(LockFlags::MUTABLE) {}

        self.kernel_drop_lock(lock_handle)
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
        let actor = self.kernel_get_current_actor().unwrap();

        let package_address = actor.package_address().clone();
        let handle = self.kernel_lock_substate(
            &RENodeId::GlobalObject(package_address.into()),
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
                    SystemError::SubstateValidationError(Box::new(
                        SubstateValidationError::BlueprintNotFound(blueprint_ident.to_string()),
                    )),
                ))?;

        let type_parent = if let Some(parent) = &schema.parent {
            match actor {
                Actor::Method {
                    global_address: Some(address),
                    blueprint,
                    ..
                } if parent.eq(blueprint.blueprint_name.as_str()) => Some(address),
                _ => {
                    return Err(RuntimeError::SystemError(
                        SystemError::InvalidChildObjectCreation,
                    ));
                }
            }
        } else {
            None
        };

        if schema.substates.len() != app_states.len() {
            return Err(RuntimeError::SystemError(
                SystemError::SubstateValidationError(Box::new(
                    SubstateValidationError::WrongNumberOfSubstates(
                        blueprint_ident.to_string(),
                        app_states.len(),
                        schema.substates.len(),
                    ),
                )),
            ));
        }
        for i in 0..app_states.len() {
            validate_payload_against_schema(&app_states[i], &schema.schema, schema.substates[i])
                .map_err(|err| {
                    RuntimeError::SystemError(SystemError::SubstateValidationError(Box::new(
                        SubstateValidationError::SchemaValidationError(
                            blueprint_ident.to_string(),
                            err.error_message(&schema.schema),
                        ),
                    )))
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
        let (node_init, node_type) = match package_address {
            RESOURCE_MANAGER_PACKAGE => match blueprint_ident {
                FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT => (
                    RENodeInit::Object(btreemap!(
                        SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager) => RuntimeSubstate::ResourceManager(parser.decode_next())
                    )),
                    AllocateEntityType::Object,
                ),
                NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT => (
                    RENodeInit::Object(btreemap!(
                        SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager) => RuntimeSubstate::NonFungibleResourceManager(parser.decode_next())
                    )),
                    AllocateEntityType::Object,
                ),
                PROOF_BLUEPRINT => (
                    RENodeInit::Object(btreemap!(
                        SubstateOffset::Proof(ProofOffset::Info) => RuntimeSubstate::ProofInfo(parser.decode_next()),
                        SubstateOffset::Proof(ProofOffset::Fungible) => RuntimeSubstate::FungibleProof(parser.decode_next()),
                        SubstateOffset::Proof(ProofOffset::NonFungible) => RuntimeSubstate::NonFungibleProof(parser.decode_next()),
                    )),
                    AllocateEntityType::Object,
                ),
                BUCKET_BLUEPRINT => (
                    RENodeInit::Object(btreemap!(
                        SubstateOffset::Bucket(BucketOffset::Info) => RuntimeSubstate::BucketInfo(parser.decode_next()),
                        SubstateOffset::Bucket(BucketOffset::LiquidFungible) => RuntimeSubstate::BucketLiquidFungible(parser.decode_next()),
                        SubstateOffset::Bucket(BucketOffset::LockedFungible) => RuntimeSubstate::BucketLockedFungible(parser.decode_next()),
                        SubstateOffset::Bucket(BucketOffset::LiquidNonFungible) => RuntimeSubstate::BucketLiquidNonFungible(parser.decode_next()),
                        SubstateOffset::Bucket(BucketOffset::LockedNonFungible) => RuntimeSubstate::BucketLockedNonFungible(parser.decode_next()),
                    )),
                    AllocateEntityType::Object,
                ),
                FUNGIBLE_VAULT_BLUEPRINT => (
                    RENodeInit::Object(btreemap!(
                        SubstateOffset::Vault(VaultOffset::Info) => RuntimeSubstate::FungibleVaultInfo(parser.decode_next()),
                        SubstateOffset::Vault(VaultOffset::LiquidFungible) => RuntimeSubstate::VaultLiquidFungible(parser.decode_next()),
                        SubstateOffset::Vault(VaultOffset::LockedFungible) => RuntimeSubstate::VaultLockedFungible(parser.decode_next()),
                    )),
                    AllocateEntityType::Vault,
                ),
                NON_FUNGIBLE_VAULT_BLUEPRINT => (
                    RENodeInit::Object(btreemap!(
                        SubstateOffset::Vault(VaultOffset::Info) => RuntimeSubstate::NonFungibleVaultInfo(parser.decode_next()),
                        SubstateOffset::Vault(VaultOffset::LiquidNonFungible) => RuntimeSubstate::VaultLiquidNonFungible(parser.decode_next()),
                        SubstateOffset::Vault(VaultOffset::LockedNonFungible) => RuntimeSubstate::VaultLockedNonFungible(parser.decode_next()),
                    )),
                    AllocateEntityType::Vault,
                ),
                blueprint => panic!("Unexpected blueprint {}", blueprint),
            },
            METADATA_PACKAGE => (RENodeInit::Object(btreemap!()), AllocateEntityType::Object),
            ROYALTY_PACKAGE => match blueprint_ident {
                COMPONENT_ROYALTY_BLUEPRINT => (
                    RENodeInit::Object(btreemap!(
                        SubstateOffset::Royalty(RoyaltyOffset::RoyaltyConfig) => RuntimeSubstate::ComponentRoyaltyConfig(parser.decode_next()),
                        SubstateOffset::Royalty(RoyaltyOffset::RoyaltyAccumulator) => RuntimeSubstate::ComponentRoyaltyAccumulator(parser.decode_next())
                    )),
                    AllocateEntityType::Object,
                ),
                blueprint => panic!("Unexpected blueprint {}", blueprint),
            },
            ACCESS_RULES_PACKAGE => (
                RENodeInit::Object(btreemap!(
                    SubstateOffset::AccessRules(AccessRulesOffset::AccessRules) => RuntimeSubstate::MethodAccessRules(parser.decode_next())
                )),
                AllocateEntityType::Object,
            ),
            EPOCH_MANAGER_PACKAGE => match blueprint_ident {
                VALIDATOR_BLUEPRINT => (
                    RENodeInit::Object(btreemap!(
                        SubstateOffset::Validator(ValidatorOffset::Validator) => RuntimeSubstate::Validator(parser.decode_next())
                    )),
                    AllocateEntityType::Object,
                ),
                EPOCH_MANAGER_BLUEPRINT => (
                    RENodeInit::Object(btreemap!(
                        SubstateOffset::EpochManager(EpochManagerOffset::EpochManager) => RuntimeSubstate::EpochManager(parser.decode_next()),
                        SubstateOffset::EpochManager(EpochManagerOffset::CurrentValidatorSet) => RuntimeSubstate::ValidatorSet(parser.decode_next()),
                        SubstateOffset::EpochManager(EpochManagerOffset::RegisteredValidators) => RuntimeSubstate::RegisteredValidators(parser.decode_next()),
                    )),
                    AllocateEntityType::Object,
                ),
                blueprint => panic!("Unexpected blueprint {}", blueprint),
            },
            ACCESS_CONTROLLER_PACKAGE => (
                RENodeInit::Object(btreemap!(
                    SubstateOffset::AccessController(AccessControllerOffset::AccessController)
                        => RuntimeSubstate::AccessController(parser.decode_next())
                )),
                AllocateEntityType::Object,
            ),
            IDENTITY_PACKAGE => (RENodeInit::Object(btreemap!()), AllocateEntityType::Object),
            ACCOUNT_PACKAGE => (
                RENodeInit::Object(btreemap!(
                    SubstateOffset::Account(AccountOffset::Account)
                        => RuntimeSubstate::Account(parser.decode_next())
                )),
                AllocateEntityType::Object,
            ),
            CLOCK_PACKAGE => (
                RENodeInit::Object(btreemap!(
                    SubstateOffset::Clock(ClockOffset::CurrentTimeRoundedToMinutes)
                        => RuntimeSubstate::CurrentTimeRoundedToMinutes(parser.decode_next())
                )),
                AllocateEntityType::Object,
            ),
            _ => (
                RENodeInit::Object(btreemap!(
                    SubstateOffset::Component(ComponentOffset::State0) => RuntimeSubstate::ComponentState(
                        ComponentStateSubstate (parser.decode_next::<ScryptoValue>())
                    )
                )),
                AllocateEntityType::Object,
            ),
        };
        parser.end();

        let node_id = self.kernel_allocate_node_id(node_type)?;

        self.kernel_create_node(
            node_id,
            node_init,
            btreemap!(
                NodeModuleId::TypeInfo => RENodeModuleInit::TypeInfo(
                    TypeInfoSubstate::Object(ObjectInfo {
                        blueprint: Blueprint::new(&package_address, blueprint_ident),
                        global: false,
                        type_parent,
                    })
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
        // FIXME check completeness of modules

        let node_type = match node_id {
            RENodeId::Object(..) => {
                let type_info = TypeInfoBlueprint::get_type(&node_id, self)?;
                let blueprint = match type_info {
                    TypeInfoSubstate::Object(ObjectInfo {
                        blueprint, global, ..
                    }) if !global => blueprint,
                    _ => return Err(RuntimeError::SystemError(SystemError::CannotGlobalize)),
                };

                match (blueprint.package_address, blueprint.blueprint_name.as_str()) {
                    (ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT) => AllocateEntityType::GlobalAccount,
                    (IDENTITY_PACKAGE, IDENTITY_BLUEPRINT) => AllocateEntityType::GlobalIdentity,
                    (ACCESS_CONTROLLER_PACKAGE, ACCESS_CONTROLLER_BLUEPRINT) => {
                        AllocateEntityType::GlobalAccessController
                    }
                    (EPOCH_MANAGER_PACKAGE, VALIDATOR_BLUEPRINT) => {
                        AllocateEntityType::GlobalValidator
                    }
                    _ => AllocateEntityType::GlobalComponent,
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
        let module_ids = modules.keys().cloned().collect::<BTreeSet<NodeModuleId>>();
        let standard_object = btreeset!(
            NodeModuleId::Metadata,
            NodeModuleId::ComponentRoyalty,
            NodeModuleId::AccessRules
        );
        // TODO: remove
        let resource_manager_object = btreeset!(
            NodeModuleId::Metadata,
            NodeModuleId::ComponentRoyalty,
            NodeModuleId::AccessRules,
        );
        if module_ids != standard_object && module_ids != resource_manager_object {
            return Err(RuntimeError::SystemError(SystemError::InvalidModuleSet(
                Box::new(InvalidModuleSet(node_id, module_ids)),
            )));
        }

        let node = self.kernel_drop_node(&node_id)?;

        let mut module_substates = BTreeMap::new();
        let mut component_substates = BTreeMap::new();
        for (node_module_id, substates) in node.substates {
            match node_module_id {
                NodeModuleId::SELF => {
                    for (offset, substate) in substates {
                        component_substates.insert(offset, substate);
                    }
                },
                _ => {
                    for (offset, substate) in substates {
                        module_substates.insert((node_module_id, offset), substate);
                    }
                },
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

        match type_info_substate {
            TypeInfoSubstate::Object(ObjectInfo { ref mut global, .. }) if !*global => {
                *global = true
            }
            _ => return Err(RuntimeError::SystemError(SystemError::CannotGlobalize)),
        };

        module_init.insert(
            NodeModuleId::TypeInfo,
            RENodeModuleInit::TypeInfo(type_info_substate),
        );

        // TODO: Check node type matches modules provided

        for (module_id, object_id) in modules {
            match module_id {
                NodeModuleId::SELF | NodeModuleId::Iterable | NodeModuleId::TypeInfo => {
                    return Err(RuntimeError::SystemError(SystemError::InvalidModule))
                }
                NodeModuleId::AccessRules => {
                    let node_id = RENodeId::Object(object_id);
                    let blueprint = self.get_object_info(node_id)?.blueprint;
                    let expected = Blueprint::new(&ACCESS_RULES_PACKAGE, ACCESS_RULES_BLUEPRINT);
                    if !blueprint.eq(&expected) {
                        return Err(RuntimeError::SystemError(SystemError::InvalidModuleType(
                            Box::new(InvalidModuleType {
                                expected_blueprint: expected,
                                actual_blueprint: blueprint,
                            }),
                        )));
                    }

                    let mut node = self.kernel_drop_node(&RENodeId::Object(object_id))?;

                    let mut access_rules_substates = node
                        .substates.remove(&NodeModuleId::SELF).unwrap();

                    let access_rules = access_rules_substates
                        .remove(&SubstateOffset::AccessRules(AccessRulesOffset::AccessRules))
                        .unwrap();
                    let access_rules: MethodAccessRulesSubstate = access_rules.into();

                    module_init
                        .insert(module_id, RENodeModuleInit::MethodAccessRules(access_rules));
                }
                NodeModuleId::Metadata => {
                    let node_id = RENodeId::Object(object_id);
                    let blueprint = self.get_object_info(node_id)?.blueprint;
                    let expected = Blueprint::new(&METADATA_PACKAGE, METADATA_BLUEPRINT);
                    if !blueprint.eq(&expected) {
                        return Err(RuntimeError::SystemError(SystemError::InvalidModuleType(
                            Box::new(InvalidModuleType {
                                expected_blueprint: expected,
                                actual_blueprint: blueprint,
                            }),
                        )));
                    }

                    let mut node = self.kernel_drop_node(&node_id)?;

                    let metadata_substates = node.substates.remove(&NodeModuleId::SELF);
                    let mut substates = BTreeMap::new();
                    if let Some(metadata_substates) = metadata_substates {
                        for (offset, substate) in metadata_substates {
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
                    let blueprint = self.get_object_info(node_id)?.blueprint;
                    let expected = Blueprint::new(&ROYALTY_PACKAGE, COMPONENT_ROYALTY_BLUEPRINT);
                    if !blueprint.eq(&expected) {
                        return Err(RuntimeError::SystemError(SystemError::InvalidModuleType(
                            Box::new(InvalidModuleType {
                                expected_blueprint: expected,
                                actual_blueprint: blueprint,
                            }),
                        )));
                    }

                    let mut node = self.kernel_drop_node(&node_id)?;

                    let mut royalty_substates = node
                        .substates.remove(&NodeModuleId::SELF).unwrap();


                    let config = royalty_substates
                        .remove(&SubstateOffset::Royalty(RoyaltyOffset::RoyaltyConfig))
                        .unwrap();
                    let config: ComponentRoyaltyConfigSubstate = config.into();
                    let accumulator = royalty_substates
                        .remove(&SubstateOffset::Royalty(RoyaltyOffset::RoyaltyAccumulator))
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
        receiver: &RENodeId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        self.call_module_method(receiver, NodeModuleId::SELF, method_name, args)
    }

    fn call_module_method(
        &mut self,
        receiver: &RENodeId,
        node_module_id: NodeModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let invocation = Box::new(MethodInvocation {
            identifier: MethodIdentifier(receiver.clone(), node_module_id, method_name.to_string()),
            args,
        });

        self.kernel_invoke(invocation).map(|v| v.into())
    }

    fn call_function(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let invocation = Box::new(FunctionInvocation {
            identifier: FunctionIdentifier::new(
                Blueprint::new(&package_address, blueprint_name),
                function_name.to_string(),
            ),
            args,
        });

        self.kernel_invoke(invocation).map(|v| v.into())
    }

    fn get_object_info(&mut self, node_id: RENodeId) -> Result<ObjectInfo, RuntimeError> {
        let type_info = TypeInfoBlueprint::get_type(&node_id, self)?;
        let object_info = match type_info {
            TypeInfoSubstate::Object(info) => info,
            TypeInfoSubstate::KeyValueStore(..) => {
                return Err(RuntimeError::SystemError(SystemError::NotAnObject))
            }
            TypeInfoSubstate::IterableMap(..) => {
                return Err(RuntimeError::SystemError(SystemError::NotAnObject))
            }
        };

        Ok(object_info)
    }

    fn new_key_value_store(
        &mut self,
        schema: KeyValueStoreSchema,
    ) -> Result<KeyValueStoreId, RuntimeError> {
        schema
            .schema
            .validate()
            .map_err(|e| RuntimeError::SystemError(SystemError::InvalidKeyValueStoreSchema(e)))?;

        let node_id = self.kernel_allocate_node_id(AllocateEntityType::KeyValueStore)?;

        self.kernel_create_node(
            node_id,
            RENodeInit::KeyValueStore,
            btreemap!(
                NodeModuleId::TypeInfo => RENodeModuleInit::TypeInfo(TypeInfoSubstate::KeyValueStore(schema)),
        ))?;

        Ok(node_id.into())
    }

    fn get_key_value_store_info(
        &mut self,
        node_id: RENodeId,
    ) -> Result<KeyValueStoreSchema, RuntimeError> {
        let type_info = TypeInfoBlueprint::get_type(&node_id, self)?;
        let schema = match type_info {
            TypeInfoSubstate::Object { .. } | TypeInfoSubstate::IterableMap(..) => {
                return Err(RuntimeError::SystemError(SystemError::NotAKeyValueStore))
            }
            TypeInfoSubstate::KeyValueStore(schema) => schema,
        };

        Ok(schema)
    }


    fn drop_object(&mut self, node_id: RENodeId) -> Result<(), RuntimeError> {
        self.kernel_drop_node(&node_id)?;
        Ok(())
    }
}

impl<'g, 's, W> ClientCostingApi<RuntimeError> for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    #[trace_resources(log=units)]
    fn consume_cost_units(
        &mut self,
        units: u32,
        reason: ClientCostingReason,
    ) -> Result<(), RuntimeError> {
        // No costing applied

        self.kernel_get_module_state().costing.apply_execution_cost(
            match reason {
                ClientCostingReason::RunWasm => CostingReason::RunWasm,
                ClientCostingReason::RunNative => CostingReason::RunNative,
                ClientCostingReason::RunSystem => CostingReason::RunSystem,
            },
            |_| units,
            5,
        )
    }

    fn credit_cost_units(
        &mut self,
        vault_id: ObjectId,
        locked_fee: LiquidFungibleResource,
        contingent: bool,
    ) -> Result<LiquidFungibleResource, RuntimeError> {
        // No costing applied

        self.kernel_get_module_state()
            .costing
            .credit_cost_units(vault_id, locked_fee, contingent)
    }
}

impl<'g, 's, W> ClientActorApi<RuntimeError> for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn get_global_address(&mut self) -> Result<Address, RuntimeError> {
        self.kernel_get_current_actor()
            .and_then(|e| match e {
                Actor::Method {
                    global_address: Some(address),
                    ..
                } => Some(address),
                _ => None,
            })
            .ok_or(RuntimeError::SystemError(
                SystemError::GlobalAddressDoesNotExist,
            ))
    }

    fn get_blueprint(&mut self) -> Result<Blueprint, RuntimeError> {
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        Ok(self.kernel_get_current_actor().unwrap().blueprint().clone())
    }
}

impl<'g, 's, W> ClientAuthApi<RuntimeError> for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn get_auth_zone(&mut self) -> Result<ObjectId, RuntimeError> {
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        let auth_zone_id = self.kernel_get_module_state().auth.last_auth_zone();

        Ok(auth_zone_id.into())
    }

    fn assert_access_rule(&mut self, rule: AccessRule) -> Result<(), RuntimeError> {
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        // Decide `authorization`, `barrier_crossing_allowed`, and `tip_auth_zone_id`
        let authorization = convert_contextless(&rule);
        let barrier_crossings_required = 1;
        let barrier_crossings_allowed = 1;
        let auth_zone_id = self.kernel_get_module_state().auth.last_auth_zone();

        // Authenticate
        // TODO: should we just run in `Client` model?
        // Currently, this is to allow authentication to read auth zone substates directly without invocation.
        self.execute_in_mode(ExecutionMode::System, |api| {
            if !Authentication::verify_method_auth(
                barrier_crossings_required,
                barrier_crossings_allowed,
                auth_zone_id,
                &authorization,
                api,
            )? {
                return Err(RuntimeError::SystemError(
                    SystemError::AssertAccessRuleFailed,
                ));
            }
            Ok(())
        })
    }
}

impl<'g, 's, W> ClientTransactionLimitsApi<RuntimeError> for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn update_wasm_memory_usage(&mut self, consumed_memory: usize) -> Result<(), RuntimeError> {
        // No costing applied

        let current_depth = self.kernel_get_current_depth();
        self.kernel_get_module_state()
            .transaction_limits
            .update_wasm_memory_usage(current_depth, consumed_memory)
    }
}

impl<'g, 's, W> ClientExecutionTraceApi<RuntimeError> for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn update_instruction_index(&mut self, new_index: usize) -> Result<(), RuntimeError> {
        // No costing applied

        self.kernel_get_module_state()
            .execution_trace
            .update_instruction_index(new_index);
        Ok(())
    }
}

impl<'g, 's, W> ClientEventApi<RuntimeError> for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn emit_event(&mut self, event_name: String, event_data: Vec<u8>) -> Result<(), RuntimeError> {
        // Costing event emission.
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        let actor = self.kernel_get_current_actor();

        // Locking the package info substate associated with the emitter's package
        let (handle, blueprint_schema, local_type_index) = {
            // Getting the package address and blueprint name associated with the actor
            let blueprint = match actor {
                Some(Actor::Method {
                    node_id, module_id, ..
                }) => match module_id {
                    NodeModuleId::AccessRules => Ok(Blueprint::new(
                        &ACCESS_RULES_PACKAGE,
                        ACCESS_RULES_BLUEPRINT,
                    )),
                    NodeModuleId::ComponentRoyalty => Ok(Blueprint::new(
                        &ROYALTY_PACKAGE,
                        COMPONENT_ROYALTY_BLUEPRINT,
                    )),
                    NodeModuleId::Metadata => {
                        Ok(Blueprint::new(&METADATA_PACKAGE, METADATA_BLUEPRINT))
                    }
                    NodeModuleId::SELF => self.get_object_info(node_id).map(|i| i.blueprint),
                    NodeModuleId::TypeInfo | NodeModuleId::Iterable => Err(RuntimeError::ApplicationError(
                        ApplicationError::EventError(Box::new(EventError::NoAssociatedPackage)),
                    )),
                },
                Some(Actor::Function { ref blueprint, .. }) => Ok(blueprint.clone()),
                _ => Err(RuntimeError::ApplicationError(
                    ApplicationError::EventError(Box::new(EventError::InvalidActor)),
                )),
            }?;

            let handle = self.kernel_lock_substate(
                &RENodeId::GlobalObject(Address::Package(blueprint.package_address)),
                NodeModuleId::SELF,
                SubstateOffset::Package(PackageOffset::Info),
                LockFlags::read_only(),
            )?;
            let package_info = self.kernel_get_substate_ref::<PackageInfoSubstate>(handle)?;
            let blueprint_schema = package_info
                .schema
                .blueprints
                .get(&blueprint.blueprint_name)
                .map_or(
                    Err(RuntimeError::ApplicationError(
                        ApplicationError::EventError(Box::new(EventError::SchemaNotFoundError {
                            blueprint: blueprint.clone(),
                            event_name: event_name.clone(),
                        })),
                    )),
                    Ok,
                )?;

            // Translating the event name to it's local_type_index which is stored in the blueprint
            // schema
            let local_type_index = blueprint_schema.event_schema.get(&event_name).map_or(
                Err(RuntimeError::ApplicationError(
                    ApplicationError::EventError(Box::new(EventError::SchemaNotFoundError {
                        blueprint: blueprint.clone(),
                        event_name,
                    })),
                )),
                Ok,
            )?;

            (handle, blueprint_schema, local_type_index)
        };

        // Construct the event type identifier based on the current actor
        let event_type_identifier = match actor {
            Some(Actor::Method {
                node_id, module_id, ..
            }) => Ok(EventTypeIdentifier(
                Emitter::Method(node_id, module_id),
                *local_type_index,
            )),
            Some(Actor::Function { ref blueprint, .. }) => Ok(EventTypeIdentifier(
                Emitter::Function(
                    RENodeId::GlobalObject(Address::Package(blueprint.package_address)),
                    NodeModuleId::SELF,
                    blueprint.blueprint_name.to_string(),
                ),
                *local_type_index,
            )),
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::EventError(Box::new(EventError::InvalidActor)),
            )),
        }?;

        // Validating the event data against the event schema
        validate_payload_against_schema(
            &event_data,
            &blueprint_schema.schema,
            event_type_identifier.1,
        )
        .map_err(|err| {
            RuntimeError::ApplicationError(ApplicationError::EventError(Box::new(
                EventError::EventSchemaNotMatch(err.error_message(&blueprint_schema.schema)),
            )))
        })?;

        // Adding the event to the event store
        self.kernel_get_module_state()
            .events
            .add_event(event_type_identifier, event_data);

        // Dropping the lock on the PackageInfo
        self.kernel_drop_lock(handle)?;
        Ok(())
    }
}

impl<'g, 's, W> ClientLoggerApi<RuntimeError> for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn log_message(&mut self, level: Level, message: String) -> Result<(), RuntimeError> {
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        self.kernel_get_module_state()
            .logger
            .add_log(level, message);
        Ok(())
    }
}

impl<'g, 's, W> ClientTransactionRuntimeApi<RuntimeError> for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    fn get_transaction_hash(&mut self) -> Result<Hash, RuntimeError> {
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        Ok(self
            .kernel_get_module_state()
            .transaction_runtime
            .transaction_hash())
    }

    fn generate_uuid(&mut self) -> Result<u128, RuntimeError> {
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        Ok(self
            .kernel_get_module_state()
            .transaction_runtime
            .generate_uuid())
    }
}

impl<'g, 's, W> ClientIterableMapApi<RuntimeError> for Kernel<'g, 's, W> where W: WasmEngine
{
    fn new_iterable_map(&mut self, schema: IterableMapSchema) -> Result<ObjectId, RuntimeError> {
        schema
            .schema
            .validate()
            .map_err(|e| RuntimeError::SystemError(SystemError::InvalidKeyValueStoreSchema(e)))?;

        let node_id = self.kernel_allocate_node_id(AllocateEntityType::KeyValueStore)?;

        self.kernel_create_node(
            node_id,
            RENodeInit::KeyValueStore,
            btreemap!(
                NodeModuleId::TypeInfo => RENodeModuleInit::TypeInfo(TypeInfoSubstate::IterableMap(schema)),
        ))?;

        Ok(node_id.into())
    }

    fn insert_into_iterable_map(&mut self, node_id: RENodeId, key: Vec<u8>, value: Vec<u8>) -> Result<(), RuntimeError> {
        self.kernel_insert_into_iterable_map(&node_id, &NodeModuleId::Iterable, key, value)
    }

    fn remove_from_iterable_map(&mut self, node_id: RENodeId, key: Vec<u8>) {
        self.kernel_remove_from_iterable_map(&node_id, &NodeModuleId::Iterable, key);
    }

    fn first_in_iterable_map(&mut self, node_id: RENodeId, count: u32) -> Result<Vec<Vec<u8>>, RuntimeError> {
        let first = self.kernel_get_first_in_iterable_map(&node_id, &NodeModuleId::Iterable, count)?;
        let first = first.into_iter().map(|(_id, substate)| {
            let (bytes, _, _) = substate.to_ref().to_scrypto_value().unpack();
            bytes
        }).collect();
        Ok(first)
    }
}

impl<'g, 's, W> ClientApi<RuntimeError> for Kernel<'g, 's, W> where W: WasmEngine {}
