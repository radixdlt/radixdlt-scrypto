use crate::errors::{ApplicationError, InvalidDropNodeAccess, InvalidModuleSet, InvalidModuleType, InvalidSubstateAccess, KernelError, RuntimeError, SubstateValidationError};
use crate::errors::{SystemError, SystemInvokeError};
use crate::kernel::actor::{Actor, ExecutionMode};
use crate::kernel::call_frame::RefType;
use crate::kernel::heap::HeapNode;
use crate::kernel::kernel_api::*;
use crate::system::node_init::ModuleInit;
use crate::system::node_modules::type_info::{TypeInfoBlueprint, TypeInfoSubstate};
use crate::system::system_modules::costing::FIXED_LOW_FEE;
use crate::system::system_modules::events::EventError;
use crate::system::system_upstream::{SystemInvocation, SystemUpstream};
use crate::types::*;
use crate::vm::wasm::WasmEngine;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::node_modules::metadata::*;
use radix_engine_interface::api::node_modules::royalty::*;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::clock::CLOCK_BLUEPRINT;
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::schema::KeyValueStoreSchema;
use resources_tracker_macro::trace_resources;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;

use super::system_modules::auth::{convert_contextless, Authentication};
use super::system_modules::costing::CostingReason;

pub struct SystemDownstream<'a, 'g, Y: KernelApi<SystemUpstream<'g, W>>, W: WasmEngine + 'g> {
    pub api: &'a mut Y,
    pub phantom: PhantomData<&'g W>,
}

impl<'a, 'g, Y, W> SystemDownstream<'a, 'g, Y, W>
where
    Y: KernelApi<SystemUpstream<'g, W>>,
    W: WasmEngine + 'g,
{
    pub fn new(api: &'a mut Y) -> Self {
        Self {
            api,
            phantom: PhantomData::default(),
        }
    }

    fn can_substate_be_accessed(
        actor: &Actor,
        node_id: &NodeId,
    ) -> bool {
        // TODO: Remove
        if is_native_package(actor.blueprint().package_address) {
            return true;
        }

        if node_id.is_internal_kv_store() {
            return true;
        }

        match actor {
            Actor::Method {
                node_id: actor_node_id,
                ..
            } if actor_node_id == node_id => true,
            _ => false,
        }
    }
}

impl<'a, 'g, Y, W> ClientSubstateApi<RuntimeError> for SystemDownstream<'a, 'g, Y, W>
where
    Y: KernelApi<SystemUpstream<'g, W>>,
    W: WasmEngine + 'g,
{
    fn sys_lock_substate(
        &mut self,
        node_id: &NodeId,
        substate_key: &SubstateKey,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError> {
        let type_info = TypeInfoBlueprint::get_type(&node_id, self.api)?;

        // TODO: Remove
        if flags.contains(LockFlags::UNMODIFIED_BASE) || flags.contains(LockFlags::FORCE_WRITE) {
            match &type_info {
                TypeInfoSubstate::Object(info)
                    if info.blueprint.package_address.eq(&RESOURCE_MANAGER_PACKAGE)
                        && info.blueprint.blueprint_name.eq(FUNGIBLE_VAULT_BLUEPRINT) => {}
                _ => {
                    return Err(RuntimeError::SystemError(SystemError::InvalidLockFlags));
                }
            }
        }

        let actor = self.api.kernel_get_current_actor().unwrap();

        // TODO: Check if valid substate_key for node_id
        if !Self::can_substate_be_accessed(
            &actor,
            node_id,
        ) {
            return Err(RuntimeError::KernelError(
                KernelError::InvalidSubstateAccess(Box::new(InvalidSubstateAccess {
                    actor: actor.clone(),
                    node_id: node_id.clone(),
                    substate_key: substate_key.clone(),
                    flags,
                })),
            ));
        }

        let module_id = match type_info {
            TypeInfoSubstate::KeyValueStore(..) => SysModuleId::ObjectMap,
            TypeInfoSubstate::Object(ObjectInfo { blueprint, .. }) => {
                if let Actor::Method { module_id, .. } = &actor
                {
                    match module_id {
                        ObjectModuleId::SELF => {
                            match (blueprint.package_address, blueprint.blueprint_name.as_str()) {
                                (METADATA_PACKAGE, METADATA_BLUEPRINT) => SysModuleId::ObjectMap,
                                _ => SysModuleId::ObjectTuple,
                            }
                        }
                        ObjectModuleId::Metadata => SysModuleId::Metadata,
                        ObjectModuleId::Royalty => SysModuleId::Royalty,
                        ObjectModuleId::AccessRules => SysModuleId::AccessRules,
                    }
                } else {
                    match (blueprint.package_address, blueprint.blueprint_name.as_str()) {
                        (METADATA_PACKAGE, METADATA_BLUEPRINT) => SysModuleId::ObjectMap,
                        _ => SysModuleId::ObjectTuple,
                    }
                }
            }
        };

        self.api
            .kernel_lock_substate(&node_id, module_id, substate_key, flags)
    }

    fn sys_read_substate(&mut self, lock_handle: LockHandle) -> Result<Vec<u8>, RuntimeError> {
        self.api
            .kernel_read_substate(lock_handle)
            .map(|v| v.as_slice().to_vec())
    }

    fn sys_write_substate(
        &mut self,
        lock_handle: LockHandle,
        buffer: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        let LockInfo {
            node_id, module_id, ..
        } = self.api.kernel_get_lock_info(lock_handle)?;

        if module_id.eq(&SysModuleId::ObjectMap) {
            let type_info = TypeInfoBlueprint::get_type(&node_id, self.api)?;
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
                _ => {
                    // TODO: Other schema checks
                    // TODO: Check objects stored are storeable
                }
            }
        } else {
            // TODO: Other schema checks
            // TODO: Check objects stored are storeable
        }

        let substate = IndexedScryptoValue::from_vec(buffer)
            .map_err(|_| RuntimeError::SystemError(SystemError::InvalidSubstateWrite))?;
        self.api.kernel_write_substate(lock_handle, substate)?;

        Ok(())
    }

    fn sys_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError> {
        let info = self.api.kernel_get_lock_info(lock_handle)?;
        if info.flags.contains(LockFlags::MUTABLE) {}

        self.api.kernel_drop_lock(lock_handle)
    }
}

impl<'a, 'g, Y, W> ClientObjectApi<RuntimeError> for SystemDownstream<'a, 'g, Y, W>
where
    Y: KernelApi<SystemUpstream<'g, W>>,
    W: WasmEngine + 'g,
{
    fn new_object(
        &mut self,
        blueprint_ident: &str,
        object_states: Vec<Vec<u8>>,
    ) -> Result<NodeId, RuntimeError> {
        let actor = self.api.kernel_get_current_actor().unwrap();
        let package_address = actor.package_address().clone();

        let handle = self.api.kernel_lock_substate(
            package_address.as_node_id(),
            SysModuleId::ObjectTuple,
            &PackageOffset::Info.into(),
            LockFlags::read_only(),
        )?;
        let package: PackageInfoSubstate =
            self.api.kernel_read_substate(handle)?.as_typed().unwrap();
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
        if schema.substates.len() != object_states.len() {
            return Err(RuntimeError::SystemError(
                SystemError::SubstateValidationError(Box::new(
                    SubstateValidationError::WrongNumberOfSubstates(
                        blueprint_ident.to_string(),
                        object_states.len(),
                        schema.substates.len(),
                    ),
                )),
            ));
        }
        for i in 0..object_states.len() {
            validate_payload_against_schema(&object_states[i], &schema.schema, schema.substates[i])
                .map_err(|err| {
                    RuntimeError::SystemError(SystemError::SubstateValidationError(Box::new(
                        SubstateValidationError::SchemaValidationError(
                            blueprint_ident.to_string(),
                            err.error_message(&schema.schema),
                        ),
                    )))
                })?;
        }
        self.api.kernel_drop_lock(handle)?;

        let entity_type = match (package_address, blueprint_ident) {
            (RESOURCE_MANAGER_PACKAGE, FUNGIBLE_VAULT_BLUEPRINT) => {
                EntityType::InternalFungibleVault
            }
            (RESOURCE_MANAGER_PACKAGE, NON_FUNGIBLE_VAULT_BLUEPRINT) => {
                EntityType::InternalNonFungibleVault
            }
            (ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT) => EntityType::InternalAccount,
            _ => EntityType::InternalGenericComponent,
        };

        let node_id = self.api.kernel_allocate_node_id(entity_type)?;
        let node_init: BTreeMap<SubstateKey, IndexedScryptoValue> = object_states
            .into_iter()
            .enumerate()
            .map(|(i, x)| {
                (
                    // TODO check size during package publishing time
                    SubstateKey::from_vec(vec![i as u8]).unwrap(),
                    IndexedScryptoValue::from_vec(x).expect("Checked by payload-schema validation"),
                )
            })
            .collect();

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

        let self_module_id = match (package_address, blueprint_ident) {
            (METADATA_PACKAGE, METADATA_BLUEPRINT) => SysModuleId::ObjectMap,
            _ => SysModuleId::ObjectTuple,
        };

        self.api.kernel_create_node(
            node_id,
            btreemap!(
                self_module_id => node_init,
                SysModuleId::TypeInfo => ModuleInit::TypeInfo(
                    TypeInfoSubstate::Object(ObjectInfo {
                        blueprint: Blueprint::new(&package_address,blueprint_ident),
                        global:false,
                        type_parent
                    })
                ).to_substates(),
            ),
        )?;

        Ok(node_id.into())
    }

    fn globalize(
        &mut self,
        modules: BTreeMap<ObjectModuleId, NodeId>,
    ) -> Result<GlobalAddress, RuntimeError> {
        // FIXME check completeness of modules

        let node_id = modules
            .get(&ObjectModuleId::SELF)
            .ok_or(RuntimeError::SystemError(SystemError::MissingModule(
                ObjectModuleId::SELF,
            )))?;

        let type_info = TypeInfoBlueprint::get_type(node_id, self.api)?;
        let blueprint = match type_info {
            TypeInfoSubstate::Object(ObjectInfo {
                blueprint, global, ..
            }) if !global => blueprint,
            _ => return Err(RuntimeError::SystemError(SystemError::CannotGlobalize)),
        };

        let entity_type = match (blueprint.package_address, blueprint.blueprint_name.as_str()) {
            (ACCOUNT_PACKAGE, PACKAGE_BLUEPRINT) => EntityType::GlobalPackage,
            (RESOURCE_MANAGER_PACKAGE, FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT) => {
                EntityType::GlobalFungibleResource
            }
            (RESOURCE_MANAGER_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT) => {
                EntityType::GlobalNonFungibleResource
            }
            (EPOCH_MANAGER_PACKAGE, EPOCH_MANAGER_BLUEPRINT) => EntityType::GlobalEpochManager,
            (EPOCH_MANAGER_PACKAGE, VALIDATOR_BLUEPRINT) => EntityType::GlobalValidator,
            (CLOCK_PACKAGE, CLOCK_BLUEPRINT) => EntityType::GlobalClock,
            (ACCESS_CONTROLLER_PACKAGE, ACCESS_CONTROLLER_BLUEPRINT) => {
                EntityType::GlobalAccessController
            }
            (ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT) => EntityType::GlobalAccount,
            (IDENTITY_PACKAGE, IDENTITY_BLUEPRINT) => EntityType::GlobalIdentity,
            _ => EntityType::GlobalGenericComponent,
        };

        let global_node_id = self.api.kernel_allocate_node_id(entity_type)?;
        let global_address = GlobalAddress::new_unchecked(global_node_id.into());
        self.globalize_with_address(modules, global_address)?;
        Ok(global_address)
    }

    fn globalize_with_address(
        &mut self,
        mut modules: BTreeMap<ObjectModuleId, NodeId>,
        address: GlobalAddress,
    ) -> Result<(), RuntimeError> {
        // Check module configuration
        let module_ids = modules
            .keys()
            .cloned()
            .collect::<BTreeSet<ObjectModuleId>>();
        let standard_object = btreeset!(
            ObjectModuleId::SELF,
            ObjectModuleId::Metadata,
            ObjectModuleId::Royalty,
            ObjectModuleId::AccessRules
        );
        if module_ids != standard_object {
            return Err(RuntimeError::SystemError(SystemError::InvalidModuleSet(
                Box::new(InvalidModuleSet(module_ids)),
            )));
        }

        // Drop the node
        let node_id = modules
            .remove(&ObjectModuleId::SELF)
            .ok_or(RuntimeError::SystemError(SystemError::MissingModule(
                ObjectModuleId::SELF,
            )))?;
        let node = self.api.kernel_drop_node(&node_id)?;
        let mut node_substates = node.substates;

        // Update the `global` flag of the type info substate.
        let type_info_module = node_substates
            .get_mut(&SysModuleId::TypeInfo)
            .unwrap()
            .remove(&TypeInfoOffset::TypeInfo.into())
            .unwrap();
        let mut type_info: TypeInfoSubstate = type_info_module.as_typed().unwrap();
        match type_info {
            TypeInfoSubstate::Object(ObjectInfo { ref mut global, .. }) if !*global => {
                *global = true
            }
            _ => return Err(RuntimeError::SystemError(SystemError::CannotGlobalize)),
        };
        node_substates
            .get_mut(&SysModuleId::TypeInfo)
            .unwrap()
            .insert(
                TypeInfoOffset::TypeInfo.into(),
                IndexedScryptoValue::from_typed(&type_info),
            );

        //  Drop the module nodes and move the substates to the designated module ID.
        for (module_id, node_id) in modules {
            match module_id {
                ObjectModuleId::SELF => panic!("Should have been removed already"),
                ObjectModuleId::AccessRules => {
                    let blueprint = self.get_object_info(&node_id)?.blueprint;
                    let expected = Blueprint::new(&ACCESS_RULES_PACKAGE, ACCESS_RULES_BLUEPRINT);
                    if !blueprint.eq(&expected) {
                        return Err(RuntimeError::SystemError(SystemError::InvalidModuleType(
                            Box::new(InvalidModuleType {
                                expected_blueprint: expected,
                                actual_blueprint: blueprint,
                            }),
                        )));
                    }

                    let mut node = self.api.kernel_drop_node(&node_id)?;
                    let access_rules = node.substates.remove(&SysModuleId::ObjectTuple).unwrap();
                    node_substates.insert(SysModuleId::AccessRules, access_rules);
                }
                ObjectModuleId::Metadata => {
                    let blueprint = self.get_object_info(&node_id)?.blueprint;
                    let expected = Blueprint::new(&METADATA_PACKAGE, METADATA_BLUEPRINT);
                    if !blueprint.eq(&expected) {
                        return Err(RuntimeError::SystemError(SystemError::InvalidModuleType(
                            Box::new(InvalidModuleType {
                                expected_blueprint: expected,
                                actual_blueprint: blueprint,
                            }),
                        )));
                    }

                    let mut node = self.api.kernel_drop_node(&node_id)?;
                    let metadata = node.substates.remove(&SysModuleId::ObjectMap).unwrap();
                    node_substates.insert(SysModuleId::Metadata, metadata);
                }
                ObjectModuleId::Royalty => {
                    let blueprint = self.get_object_info(&node_id)?.blueprint;
                    let expected = Blueprint::new(&ROYALTY_PACKAGE, COMPONENT_ROYALTY_BLUEPRINT);
                    if !blueprint.eq(&expected) {
                        return Err(RuntimeError::SystemError(SystemError::InvalidModuleType(
                            Box::new(InvalidModuleType {
                                expected_blueprint: expected,
                                actual_blueprint: blueprint,
                            }),
                        )));
                    }

                    let mut node = self.api.kernel_drop_node(&node_id)?;
                    let royalty = node.substates.remove(&SysModuleId::ObjectTuple).unwrap();
                    node_substates.insert(SysModuleId::Royalty, royalty);
                }
            }
        }

        self.api
            .kernel_create_node(address.into(), node_substates)?;

        Ok(())
    }

    fn call_method(
        &mut self,
        receiver: &NodeId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        self.call_module_method(receiver, ObjectModuleId::SELF, method_name, args)
    }

    fn call_module_method(
        &mut self,
        receiver: &NodeId,
        module_id: ObjectModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let (blueprint, global_address) = match module_id {
            ObjectModuleId::SELF => {
                let type_info = TypeInfoBlueprint::get_type(receiver, self.api)?;
                match type_info {
                    TypeInfoSubstate::Object(ObjectInfo {
                        blueprint, global, ..
                    }) => {
                        let global_address = if global {
                            Some(GlobalAddress::new_unchecked(receiver.clone().into()))
                        } else {
                            // See if we have a parent

                            // TODO: Cleanup, this is a rather crude way of trying to figure out
                            // TODO: whether the node reference is a child of the current parent
                            // TODO: this should be cleaned up once call_frame is refactored
                            let (visibility, on_heap) =
                                self.api.kernel_get_node_info(receiver).unwrap();
                            match (visibility, on_heap) {
                                (RefType::Normal, false) => {
                                    self.api.kernel_get_current_actor().and_then(|a| match a {
                                        Actor::Method { global_address, .. } => global_address,
                                        _ => None,
                                    })
                                }
                                _ => None,
                            }
                        };

                        (blueprint, global_address)
                    }

                    TypeInfoSubstate::KeyValueStore(..) => {
                        return Err(RuntimeError::SystemError(
                            SystemError::CallMethodOnKeyValueStore,
                        ))
                    }
                }
            }
            ObjectModuleId::Metadata => {
                // TODO: Check if type has metadata
                (Blueprint::new(&METADATA_PACKAGE, METADATA_BLUEPRINT), None)
            }
            ObjectModuleId::Royalty => {
                // TODO: Check if type has royalty
                (
                    Blueprint::new(&ROYALTY_PACKAGE, COMPONENT_ROYALTY_BLUEPRINT),
                    None,
                )
            }
            ObjectModuleId::AccessRules => {
                // TODO: Check if type has access rules
                (
                    Blueprint::new(&ACCESS_RULES_PACKAGE, ACCESS_RULES_BLUEPRINT),
                    None,
                )
            }
        };

        let identifier = MethodIdentifier(receiver.clone(), module_id, method_name.to_string());
        let payload_size = args.len() + identifier.2.len();

        let invocation = KernelInvocation {
            resolved_actor: Actor::method(global_address, identifier.clone(), blueprint.clone()),
            sys_invocation: SystemInvocation {
                blueprint,
                ident: FnIdent::Application(identifier.2.clone()),
                receiver: Some(identifier),
            },
            args: IndexedScryptoValue::from_vec(args).map_err(|e| {
                RuntimeError::SystemInvokeError(SystemInvokeError::InputDecodeError(e))
            })?,
            payload_size,
        };

        self.api
            .kernel_invoke_downstream(Box::new(invocation))
            .map(|v| v.into())
    }

    fn get_object_info(&mut self, node_id: &NodeId) -> Result<ObjectInfo, RuntimeError> {
        let type_info = TypeInfoBlueprint::get_type(&node_id, self.api)?;
        let object_info = match type_info {
            TypeInfoSubstate::Object(info) => info,
            TypeInfoSubstate::KeyValueStore(..) => {
                return Err(RuntimeError::SystemError(SystemError::NotAnObject))
            }
        };

        Ok(object_info)
    }

    fn get_key_value_store_info(
        &mut self,
        node_id: &NodeId,
    ) -> Result<KeyValueStoreSchema, RuntimeError> {
        let type_info = TypeInfoBlueprint::get_type(node_id, self.api)?;
        let schema = match type_info {
            TypeInfoSubstate::Object { .. } => {
                return Err(RuntimeError::SystemError(SystemError::NotAKeyValueStore))
            }
            TypeInfoSubstate::KeyValueStore(schema) => schema,
        };

        Ok(schema)
    }

    fn new_key_value_store(&mut self, schema: KeyValueStoreSchema) -> Result<NodeId, RuntimeError> {
        schema
            .schema
            .validate()
            .map_err(|e| RuntimeError::SystemError(SystemError::InvalidKeyValueStoreSchema(e)))?;

        let entity_type = EntityType::InternalKeyValueStore;
        let node_id = self.api.kernel_allocate_node_id(entity_type)?;

        self.api.kernel_create_node(
            node_id,
            btreemap!(
                SysModuleId::ObjectMap => btreemap!(),
                SysModuleId::TypeInfo => ModuleInit::TypeInfo(
                    TypeInfoSubstate::KeyValueStore(schema)
                ).to_substates(),
            ),
        )?;

        Ok(node_id.into())
    }

    fn drop_object(&mut self, node_id: NodeId) -> Result<(), RuntimeError> {
        // TODO: Cleanup
        if let Some(actor) = self.api.kernel_get_current_actor() {
            let info = self.get_object_info(&node_id)?;
            if !info.blueprint.package_address.eq(actor.package_address()) {
                return Err(RuntimeError::KernelError(
                    KernelError::InvalidDropNodeAccess(Box::new(InvalidDropNodeAccess {
                        mode: ExecutionMode::Client,
                        actor: actor.clone(),
                        node_id: node_id.clone(),
                        package_address: info.blueprint.package_address,
                        blueprint_name: info.blueprint.blueprint_name,
                    })),
                ));
            }
        }

        self.api.kernel_drop_node(&node_id)?;

        Ok(())
    }
}

impl<'a, 'g, Y, W> ClientBlueprintApi<RuntimeError> for SystemDownstream<'a, 'g, Y, W>
where
    Y: KernelApi<SystemUpstream<'g, W>>,
    W: WasmEngine + 'g,
{
    fn call_function(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let identifier = FunctionIdentifier::new(
            Blueprint::new(&package_address, blueprint_name),
            function_name.to_string(),
        );
        let payload_size = args.len() + identifier.size();

        let invocation = KernelInvocation {
            resolved_actor: Actor::function(identifier.clone()),
            args: IndexedScryptoValue::from_vec(args).map_err(|e| {
                RuntimeError::SystemInvokeError(SystemInvokeError::InputDecodeError(e))
            })?,
            sys_invocation: SystemInvocation {
                blueprint: identifier.0,
                ident: FnIdent::Application(identifier.1),
                receiver: None,
            },
            payload_size,
        };

        self.api
            .kernel_invoke_downstream(Box::new(invocation))
            .map(|v| v.into())
    }
}

impl<'a, 'g, Y, W> ClientCostingApi<RuntimeError> for SystemDownstream<'a, 'g, Y, W>
where
    Y: KernelApi<SystemUpstream<'g, W>>,
    W: WasmEngine + 'g,
{
    #[trace_resources(log=units)]
    fn consume_cost_units(
        &mut self,
        units: u32,
        reason: ClientCostingReason,
    ) -> Result<(), RuntimeError> {
        // No costing applied

        self.api
            .kernel_get_system()
            .modules
            .costing
            .apply_execution_cost(
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
        vault_id: NodeId,
        locked_fee: LiquidFungibleResource,
        contingent: bool,
    ) -> Result<LiquidFungibleResource, RuntimeError> {
        // No costing applied

        self.api
            .kernel_get_system()
            .modules
            .costing
            .credit_cost_units(vault_id, locked_fee, contingent)
    }
}

impl<'a, 'g, Y, W> ClientActorApi<RuntimeError> for SystemDownstream<'a, 'g, Y, W>
where
    Y: KernelApi<SystemUpstream<'g, W>>,
    W: WasmEngine + 'g,
{
    fn get_global_address(&mut self) -> Result<GlobalAddress, RuntimeError> {
        self.api
            .kernel_get_current_actor()
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

        Ok(self
            .api
            .kernel_get_current_actor()
            .unwrap()
            .blueprint()
            .clone())
    }
}

impl<'a, 'g, Y, W> ClientAuthApi<RuntimeError> for SystemDownstream<'a, 'g, Y, W>
where
    Y: KernelApi<SystemUpstream<'g, W>>,
    W: WasmEngine + 'g,
{
    fn get_auth_zone(&mut self) -> Result<NodeId, RuntimeError> {
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        let auth_zone_id = self.api.kernel_get_system().modules.auth.last_auth_zone();

        Ok(auth_zone_id.into())
    }

    fn assert_access_rule(&mut self, rule: AccessRule) -> Result<(), RuntimeError> {
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        // Decide `authorization`, `barrier_crossing_allowed`, and `tip_auth_zone_id`
        let authorization = convert_contextless(&rule);
        let barrier_crossings_required = 1;
        let barrier_crossings_allowed = 1;
        let auth_zone_id = self.api.kernel_get_system().modules.auth.last_auth_zone();

        // Authenticate
        // TODO: should we just run in `Client` model?
        // Currently, this is to allow authentication to read auth zone substates directly without invocation.
        //self.api.execute_in_mode(ExecutionMode::System, |api| {
        self.api.kernel_set_mode(ExecutionMode::System);

        if !Authentication::verify_method_auth(
            barrier_crossings_required,
            barrier_crossings_allowed,
            auth_zone_id,
            &authorization,
            self,
        )? {
            return Err(RuntimeError::SystemError(
                SystemError::AssertAccessRuleFailed,
            ));
        }

        self.api.kernel_set_mode(ExecutionMode::Client);

        Ok(())
    }
}

impl<'a, 'g, Y, W> ClientTransactionLimitsApi<RuntimeError> for SystemDownstream<'a, 'g, Y, W>
where
    Y: KernelApi<SystemUpstream<'g, W>>,
    W: WasmEngine + 'g,
{
    fn update_wasm_memory_usage(&mut self, consumed_memory: usize) -> Result<(), RuntimeError> {
        // No costing applied

        let current_depth = self.api.kernel_get_current_depth();
        self.api
            .kernel_get_system()
            .modules
            .transaction_limits
            .update_wasm_memory_usage(current_depth, consumed_memory)
    }
}

impl<'a, 'g, Y, W> ClientExecutionTraceApi<RuntimeError> for SystemDownstream<'a, 'g, Y, W>
where
    Y: KernelApi<SystemUpstream<'g, W>>,
    W: WasmEngine + 'g,
{
    fn update_instruction_index(&mut self, new_index: usize) -> Result<(), RuntimeError> {
        // No costing applied

        self.api
            .kernel_get_system()
            .modules
            .execution_trace
            .update_instruction_index(new_index);
        Ok(())
    }
}

impl<'a, 'g, Y, W> ClientEventApi<RuntimeError> for SystemDownstream<'a, 'g, Y, W>
where
    Y: KernelApi<SystemUpstream<'g, W>>,
    W: WasmEngine + 'g,
{
    fn emit_event(&mut self, event_name: String, event_data: Vec<u8>) -> Result<(), RuntimeError> {
        // Costing event emission.
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        let actor = self.api.kernel_get_current_actor();

        // Locking the package info substate associated with the emitter's package
        let (handle, blueprint_schema, local_type_index) = {
            // Getting the package address and blueprint name associated with the actor
            let blueprint = match actor {
                Some(Actor::Method {
                    node_id, module_id, ..
                }) => match module_id {
                    ObjectModuleId::AccessRules => Ok(Blueprint::new(
                        &ACCESS_RULES_PACKAGE,
                        ACCESS_RULES_BLUEPRINT,
                    )),
                    ObjectModuleId::Royalty => Ok(Blueprint::new(
                        &ROYALTY_PACKAGE,
                        COMPONENT_ROYALTY_BLUEPRINT,
                    )),
                    ObjectModuleId::Metadata => {
                        Ok(Blueprint::new(&METADATA_PACKAGE, METADATA_BLUEPRINT))
                    }
                    ObjectModuleId::SELF => self.get_object_info(&node_id).map(|x| x.blueprint),
                },
                Some(Actor::Function { ref blueprint, .. }) => Ok(blueprint.clone()),
                _ => Err(RuntimeError::ApplicationError(
                    ApplicationError::EventError(Box::new(EventError::InvalidActor)),
                )),
            }?;

            let handle = self.api.kernel_lock_substate(
                blueprint.package_address.as_node_id(),
                SysModuleId::ObjectTuple,
                &PackageOffset::Info.into(),
                LockFlags::read_only(),
            )?;
            let package_info: PackageInfoSubstate =
                self.api.kernel_read_substate(handle)?.as_typed().unwrap();
            let blueprint_schema = package_info
                .schema
                .blueprints
                .get(&blueprint.blueprint_name)
                .cloned()
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
            let local_type_index =
                if let Some(index) = blueprint_schema.event_schema.get(&event_name).cloned() {
                    index
                } else {
                    return Err(RuntimeError::ApplicationError(
                        ApplicationError::EventError(Box::new(EventError::SchemaNotFoundError {
                            blueprint: blueprint.clone(),
                            event_name,
                        })),
                    ));
                };

            (handle, blueprint_schema, local_type_index)
        };

        // Construct the event type identifier based on the current actor
        let event_type_identifier = match actor {
            Some(Actor::Method {
                node_id, module_id, ..
            }) => Ok(EventTypeIdentifier(
                Emitter::Method(node_id, module_id),
                local_type_index,
            )),
            Some(Actor::Function { ref blueprint, .. }) => Ok(EventTypeIdentifier(
                Emitter::Function(
                    blueprint.package_address.into(),
                    ObjectModuleId::SELF,
                    blueprint.blueprint_name.to_string(),
                ),
                local_type_index,
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
        self.api
            .kernel_get_system()
            .modules
            .events
            .add_event(event_type_identifier, event_data);

        // Dropping the lock on the PackageInfo
        self.api.kernel_drop_lock(handle)?;
        Ok(())
    }
}

impl<'a, 'g, Y, W> ClientLoggerApi<RuntimeError> for SystemDownstream<'a, 'g, Y, W>
where
    Y: KernelApi<SystemUpstream<'g, W>>,
    W: WasmEngine + 'g,
{
    fn log_message(&mut self, level: Level, message: String) -> Result<(), RuntimeError> {
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        self.api
            .kernel_get_system()
            .modules
            .logger
            .add_log(level, message);
        Ok(())
    }
}

impl<'a, 'g, Y, W> ClientTransactionRuntimeApi<RuntimeError> for SystemDownstream<'a, 'g, Y, W>
where
    Y: KernelApi<SystemUpstream<'g, W>>,
    W: WasmEngine + 'g,
{
    fn get_transaction_hash(&mut self) -> Result<Hash, RuntimeError> {
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        Ok(self
            .api
            .kernel_get_system()
            .modules
            .transaction_runtime
            .transaction_hash())
    }

    fn generate_uuid(&mut self) -> Result<u128, RuntimeError> {
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        Ok(self
            .api
            .kernel_get_system()
            .modules
            .transaction_runtime
            .generate_uuid())
    }
}

impl<'a, 'g, Y, W> ClientApi<RuntimeError> for SystemDownstream<'a, 'g, Y, W>
where
    Y: KernelApi<SystemUpstream<'g, W>>,
    W: WasmEngine + 'g,
{
}

impl<'a, 'g, Y, W> KernelNodeApi for SystemDownstream<'a, 'g, Y, W>
where
    W: 'g + WasmEngine,
    Y: KernelApi<SystemUpstream<'g, W>>,
{
    fn kernel_drop_node(&mut self, node_id: &NodeId) -> Result<HeapNode, RuntimeError> {
        self.api.kernel_drop_node(node_id)
    }

    fn kernel_allocate_virtual_node_id(&mut self, node_id: NodeId) -> Result<(), RuntimeError> {
        self.api.kernel_allocate_virtual_node_id(node_id)
    }

    fn kernel_allocate_node_id(&mut self, node_type: EntityType) -> Result<NodeId, RuntimeError> {
        self.api.kernel_allocate_node_id(node_type)
    }

    fn kernel_create_node(
        &mut self,
        node_id: NodeId,
        module_init: BTreeMap<SysModuleId, BTreeMap<SubstateKey, IndexedScryptoValue>>,
    ) -> Result<(), RuntimeError> {
        self.api.kernel_create_node(node_id, module_init)
    }
}

impl<'a, 'g, Y, W> KernelSubstateApi for SystemDownstream<'a, 'g, Y, W>
where
    W: 'g + WasmEngine,
    Y: KernelApi<SystemUpstream<'g, W>>,
{
    fn kernel_lock_substate(
        &mut self,
        node_id: &NodeId,
        module_id: SysModuleId,
        substate_key: &SubstateKey,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError> {
        self.api
            .kernel_lock_substate(node_id, module_id, substate_key, flags)
    }

    fn kernel_get_lock_info(&mut self, lock_handle: LockHandle) -> Result<LockInfo, RuntimeError> {
        self.api.kernel_get_lock_info(lock_handle)
    }

    fn kernel_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError> {
        self.api.kernel_drop_lock(lock_handle)
    }

    fn kernel_read_substate(
        &mut self,
        lock_handle: LockHandle,
    ) -> Result<&IndexedScryptoValue, RuntimeError> {
        self.api.kernel_read_substate(lock_handle)
    }

    fn kernel_write_substate(
        &mut self,
        lock_handle: LockHandle,
        value: IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        self.api.kernel_write_substate(lock_handle, value)
    }
}
