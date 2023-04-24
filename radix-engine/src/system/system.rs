use crate::errors::{
    ApplicationError, InvalidDropNodeAccess, InvalidModuleSet, InvalidModuleType, KernelError,
    RuntimeError, SubstateValidationError,
};
use crate::errors::{SystemError, SystemUpstreamError};
use crate::kernel::actor::Actor;
use crate::kernel::call_frame::RefType;
use crate::kernel::kernel_api::*;
use crate::system::node_init::ModuleInit;
use crate::system::node_modules::type_info::{TypeInfoBlueprint, TypeInfoSubstate};
use crate::system::system_callback::{SystemConfig, SystemInvocation, SystemLockData};
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_modules::costing::FIXED_LOW_FEE;
use crate::system::system_modules::events::EventError;
use crate::system::system_modules::execution_trace::{BucketSnapshot, ProofSnapshot};
use crate::types::*;
use radix_engine_interface::api::index_api::ClientIndexApi;
use radix_engine_interface::api::key_value_store_api::{ClientKeyValueStoreApi, KeyValueEntryLockHandle};
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::node_modules::metadata::*;
use radix_engine_interface::api::node_modules::royalty::*;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::sorted_index_api::SortedKey;
use radix_engine_interface::api::substate_lock_api::LockFlags;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::clock::CLOCK_BLUEPRINT;
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::schema::{BlueprintSchema, KeyValueStoreSchema};
use radix_engine_stores::interface::NodeSubstates;
use resources_tracker_macro::trace_resources;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;

use super::system_modules::auth::{convert_contextless, Authentication};
use super::system_modules::costing::CostingReason;

pub struct SystemDownstream<'a, Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject> {
    pub api: &'a mut Y,
    pub phantom: PhantomData<V>,
}

impl<'a, Y, V> SystemDownstream<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    pub fn new(api: &'a mut Y) -> Self {
        Self {
            api,
            phantom: PhantomData::default(),
        }
    }

    fn get_blueprint_schema(&mut self, blueprint: &Blueprint) -> Result<BlueprintSchema, RuntimeError> {
        let handle = self.api.kernel_lock_substate(
            blueprint.package_address.as_node_id(),
            SysModuleId::User.into(),
            &PackageOffset::Info.into(),
            LockFlags::read_only(),
            SystemLockData::default(),
        )?;
        let package: PackageInfoSubstate =
            self.api.kernel_read_substate(handle)?.as_typed().unwrap();
        let schema =
            package
                .schema
                .blueprints
                .get(blueprint.blueprint_name.as_str())
                .ok_or(RuntimeError::SystemError(
                    SystemError::SubstateValidationError(Box::new(
                        SubstateValidationError::BlueprintNotFound(blueprint.blueprint_name.to_string()),
                    )),
                ))?.clone();

        self.api.kernel_drop_lock(handle)?;

        Ok(schema)
    }

    fn verify_blueprint_fields(&mut self, blueprint: &Blueprint, fields: &Vec<Vec<u8>>) -> Result<Option<String>, RuntimeError> {
        let handle = self.api.kernel_lock_substate(
            blueprint.package_address.as_node_id(),
            SysModuleId::User.into(),
            &PackageOffset::Info.into(),
            LockFlags::read_only(),
            SystemLockData::default(),
        )?;
        let package: PackageInfoSubstate =
            self.api.kernel_read_substate(handle)?.as_typed().unwrap();
        let schema =
            package
                .schema
                .blueprints
                .get(blueprint.blueprint_name.as_str())
                .ok_or(RuntimeError::SystemError(
                    SystemError::SubstateValidationError(Box::new(
                        SubstateValidationError::BlueprintNotFound(blueprint.blueprint_name.to_string()),
                    )),
                ))?;

        if schema.substates.len() != fields.len() {
            return Err(RuntimeError::SystemError(
                SystemError::SubstateValidationError(Box::new(
                    SubstateValidationError::WrongNumberOfSubstates(
                        blueprint.clone(),
                        fields.len(),
                        schema.substates.len(),
                    ),
                )),
            ));
        }
        for i in 0..fields.len() {
            validate_payload_against_schema(&fields[i], &schema.schema, schema.substates[i])
                .map_err(|err| {
                    RuntimeError::SystemError(SystemError::SubstateValidationError(Box::new(
                        SubstateValidationError::SchemaValidationError(
                            blueprint.clone(),
                            err.error_message(&schema.schema),
                        ),
                    )))
                })?;
        }

        let parent_blueprint = schema.parent.clone();

        self.api.kernel_drop_lock(handle)?;

        Ok(parent_blueprint)
    }
}

impl<'a, Y, V> ClientSubstateLockApi<RuntimeError> for SystemDownstream<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
    fn sys_read_substate(&mut self, lock_handle: LockHandle) -> Result<Vec<u8>, RuntimeError> {
        let LockInfo {
            data, ..
        } = self.api.kernel_get_lock_info(lock_handle)?;
        if data.is_kv_store {
            panic!("Not a field");
        }

        self.api
            .kernel_read_substate(lock_handle)
            .map(|v| v.as_slice().to_vec())
    }

    #[trace_resources]
    fn sys_write_substate(
        &mut self,
        lock_handle: LockHandle,
        buffer: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        let LockInfo {
            node_id, data, ..
        } = self.api.kernel_get_lock_info(lock_handle)?;

        if data.is_kv_store {
            panic!("Not a field");
        } else {
            // TODO: Other schema checks
            // TODO: Check objects stored are storeable
        }

        let substate = IndexedScryptoValue::from_vec(buffer)
            .map_err(|_| RuntimeError::SystemError(SystemError::InvalidSubstateWrite))?;
        self.api.kernel_write_substate(lock_handle, substate)?;

        Ok(())
    }

    #[trace_resources]
    fn sys_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError> {
        let info = self.api.kernel_get_lock_info(lock_handle)?;
        if info.flags.contains(LockFlags::MUTABLE) {}

        self.api.kernel_drop_lock(lock_handle)
    }
}

impl<'a, Y, V> ClientObjectApi<RuntimeError> for SystemDownstream<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
    fn new_object(
        &mut self,
        blueprint_ident: &str,
        fields: Vec<Vec<u8>>,
    ) -> Result<NodeId, RuntimeError> {
        let actor = self.api.kernel_get_current_actor().unwrap();
        let package_address = actor.package_address().clone();
        let blueprint = Blueprint::new(&package_address, blueprint_ident);
        let parent_blueprint = self.verify_blueprint_fields(&blueprint, &fields)?;

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
        let node_init: BTreeMap<SubstateKey, IndexedScryptoValue> = fields
            .into_iter()
            .enumerate()
            .map(|(i, x)| {
                (
                    // TODO check size during package publishing time
                    SubstateKey::Tuple(i as u8),
                    IndexedScryptoValue::from_vec(x).expect("Checked by payload-schema validation"),
                )
            })
            .collect();

        let blueprint_parent = if let Some(parent) = &parent_blueprint {
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

        self.api.kernel_create_node(
            node_id,
            btreemap!(
                SysModuleId::User.into() => node_init,
                SysModuleId::TypeInfo.into() => ModuleInit::TypeInfo(
                    TypeInfoSubstate::Object(ObjectInfo {
                        blueprint: Blueprint::new(&package_address,blueprint_ident),
                        global:false,
                        blueprint_parent
                    })
                ).to_substates(),
            ),
        )?;

        Ok(node_id.into())
    }

    #[trace_resources]
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

    #[trace_resources]
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
        let mut node_substates = self.api.kernel_drop_node(&node_id)?;

        // Update the `global` flag of the type info substate.
        let type_info_module = node_substates
            .get_mut(&SysModuleId::TypeInfo.into())
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
            .get_mut(&SysModuleId::TypeInfo.into())
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

                    let mut access_rule_substates = self.api.kernel_drop_node(&node_id)?;
                    let access_rules = access_rule_substates
                        .remove(&SysModuleId::User.into())
                        .unwrap();
                    node_substates.insert(SysModuleId::AccessRules.into(), access_rules);
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

                    let mut metadata_substates = self.api.kernel_drop_node(&node_id)?;
                    let metadata = metadata_substates
                        .remove(&SysModuleId::User.into())
                        .unwrap();
                    node_substates.insert(SysModuleId::Metadata.into(), metadata);
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

                    let mut royalty_substates = self.api.kernel_drop_node(&node_id)?;
                    let royalty = royalty_substates
                        .remove(&SysModuleId::User.into())
                        .unwrap();
                    node_substates.insert(SysModuleId::Royalty.into(), royalty);
                }
            }
        }

        self.api
            .kernel_create_node(address.into(), node_substates)?;

        Ok(())
    }

    #[trace_resources]
    fn call_method(
        &mut self,
        receiver: &NodeId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        self.call_module_method(receiver, ObjectModuleId::SELF, method_name, args)
    }

    #[trace_resources]
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

                    TypeInfoSubstate::KeyValueStore(..)
                    | TypeInfoSubstate::SortedStore
                    | TypeInfoSubstate::IterableStore => {
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
                RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
            })?,
            payload_size,
        };

        self.api
            .kernel_invoke(Box::new(invocation))
            .map(|v| v.into())
    }

    #[trace_resources]
    fn get_object_info(&mut self, node_id: &NodeId) -> Result<ObjectInfo, RuntimeError> {
        let type_info = TypeInfoBlueprint::get_type(&node_id, self.api)?;
        let object_info = match type_info {
            TypeInfoSubstate::Object(info) => info,
            TypeInfoSubstate::KeyValueStore(..)
            | TypeInfoSubstate::SortedStore
            | TypeInfoSubstate::IterableStore => {
                return Err(RuntimeError::SystemError(SystemError::NotAnObject))
            }
        };

        Ok(object_info)
    }

    #[trace_resources]
    fn drop_object(&mut self, node_id: NodeId) -> Result<(), RuntimeError> {
        // TODO: Cleanup
        if let Some(actor) = self.api.kernel_get_current_actor() {
            let info = self.get_object_info(&node_id)?;
            if !info.blueprint.package_address.eq(actor.package_address()) {
                return Err(RuntimeError::KernelError(
                    KernelError::InvalidDropNodeAccess(Box::new(InvalidDropNodeAccess {
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

impl<'a, Y, V> ClientKeyValueStoreApi<RuntimeError> for SystemDownstream<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
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
                SysModuleId::User.into() => btreemap!(),
                SysModuleId::TypeInfo.into() => ModuleInit::TypeInfo(
                    TypeInfoSubstate::KeyValueStore(schema)
                ).to_substates(),
            ),
        )?;

        Ok(node_id)
    }

    #[trace_resources]
    fn get_key_value_store_info(
        &mut self,
        node_id: &NodeId,
    ) -> Result<KeyValueStoreSchema, RuntimeError> {
        let type_info = TypeInfoBlueprint::get_type(node_id, self.api)?;
        let schema = match type_info {
            TypeInfoSubstate::Object { .. }
            | TypeInfoSubstate::SortedStore
            | TypeInfoSubstate::IterableStore => {
                return Err(RuntimeError::SystemError(SystemError::NotAKeyValueStore))
            }
            TypeInfoSubstate::KeyValueStore(schema) => schema,
        };

        Ok(schema)
    }

    #[trace_resources]
    fn lock_key_value_store_entry(
        &mut self,
        node_id: &NodeId,
        key: &Vec<u8>,
        flags: LockFlags,
    ) -> Result<KeyValueEntryLockHandle, RuntimeError> {
        let type_info = TypeInfoBlueprint::get_type(&node_id, self.api)?;
        if flags.contains(LockFlags::UNMODIFIED_BASE) || flags.contains(LockFlags::FORCE_WRITE) {
            return Err(RuntimeError::SystemError(SystemError::InvalidLockFlags));
        }

        match type_info {
            TypeInfoSubstate::KeyValueStore(..) => {},
            TypeInfoSubstate::SortedStore | TypeInfoSubstate::IterableStore | TypeInfoSubstate::Object(..) => {
                return Err(RuntimeError::SystemError(SystemError::NotAKeyValueStore))
            }
        };

        self.api.kernel_lock_substate_with_default(
            &node_id,
            SysModuleId::User.into(),
            &SubstateKey::Map(key.clone()),
            flags,
            Some(|| IndexedScryptoValue::from_typed(&Option::<ScryptoValue>::None)),
            SystemLockData {
                is_kv_store: true,
            },
        )
    }

    #[trace_resources]
    fn key_value_entry_get(&mut self, handle: KeyValueEntryLockHandle) -> Result<Vec<u8>, RuntimeError> {
        let LockInfo { data, .. } = self.api.kernel_get_lock_info(handle)?;
        if !data.is_kv_store {
            return Err(RuntimeError::SystemError(SystemError::NotAKeyValueStore))
        }

        self.api
            .kernel_read_substate(handle)
            .map(|v| v.as_slice().to_vec())
    }

    #[trace_resources]
    fn key_value_entry_set(
        &mut self,
        handle: KeyValueEntryLockHandle,
        buffer: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        let LockInfo {
            node_id, data, ..
        } = self.api.kernel_get_lock_info(handle)?;

        if data.is_kv_store {
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
                },
                _ => {
                    // TODO: verify against schema
                },
            }
        } else {
            return Err(RuntimeError::SystemError(SystemError::NotAKeyValueStore))
        };

        /*
        let value: ScryptoValue = scrypto_decode(&buffer)
            .map_err(|_| {
                RuntimeError::SystemError(SystemError::InvalidSubstateWrite)
            }) ?;
        let buffer = scrypto_encode(&Option::Some(value)).unwrap();
         */



        let substate = IndexedScryptoValue::from_vec(buffer)
            .map_err(|_| RuntimeError::SystemError(SystemError::InvalidSubstateWrite))?;
        self.api.kernel_write_substate(handle, substate)?;

        Ok(())
    }
}

impl<'a, Y, V> ClientIndexApi<RuntimeError> for SystemDownstream<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    fn new_index(&mut self) -> Result<NodeId, RuntimeError> {
        let entity_type = EntityType::InternalIndex;
        let node_id = self.api.kernel_allocate_node_id(entity_type)?;

        self.api.kernel_create_node(
            node_id,
            btreemap!(
                SysModuleId::User.into() => btreemap!(),
                SysModuleId::TypeInfo.into() => ModuleInit::TypeInfo(
                    TypeInfoSubstate::IterableStore
                ).to_substates(),
            ),
        )?;

        Ok(node_id)
    }

    fn insert_into_index(
        &mut self,
        node_id: &NodeId,
        key: Vec<u8>,
        buffer: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        let type_info = TypeInfoBlueprint::get_type(&node_id, self.api)?;
        match type_info {
            TypeInfoSubstate::IterableStore => {}
            _ => {
                return Err(RuntimeError::SystemError(SystemError::NotAnIterableStore));
            }
        }

        let value = IndexedScryptoValue::from_vec(buffer).map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        if !value.owned_node_ids().is_empty() {
            return Err(RuntimeError::SystemError(
                SystemError::CannotStoreOwnedInIterable,
            ));
        }

        let module_id = SysModuleId::User.into();
        let substate_key = SubstateKey::Map(key);

        self.api
            .kernel_set_substate(node_id, module_id, substate_key, value)
    }

    fn remove_from_index(
        &mut self,
        node_id: &NodeId,
        key: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, RuntimeError> {
        let type_info = TypeInfoBlueprint::get_type(&node_id, self.api)?;
        match type_info {
            TypeInfoSubstate::IterableStore => {}
            _ => {
                return Err(RuntimeError::SystemError(SystemError::NotAnIterableStore));
            }
        }

        let module_id = SysModuleId::User.into();
        let substate_key = SubstateKey::Map(key);

        let rtn = self
            .api
            .kernel_remove_substate(node_id, module_id, &substate_key)?
            .map(|v| v.into());

        Ok(rtn)
    }

    fn scan_index(&mut self, node_id: &NodeId, count: u32) -> Result<Vec<Vec<u8>>, RuntimeError> {
        let type_info = TypeInfoBlueprint::get_type(&node_id, self.api)?;
        match type_info {
            TypeInfoSubstate::IterableStore => {}
            _ => {
                return Err(RuntimeError::SystemError(SystemError::NotAnIterableStore));
            }
        }

        let module_id = SysModuleId::User;
        let substates = self
            .api
            .kernel_scan_substates(node_id, module_id, count)?
            .into_iter()
            .map(|value| value.into())
            .collect();

        Ok(substates)
    }

    fn take(&mut self, node_id: &NodeId, count: u32) -> Result<Vec<Vec<u8>>, RuntimeError> {
        let type_info = TypeInfoBlueprint::get_type(&node_id, self.api)?;
        match type_info {
            TypeInfoSubstate::IterableStore => {}
            _ => {
                return Err(RuntimeError::SystemError(SystemError::NotAnIterableStore));
            }
        }

        let module_id = SysModuleId::User;
        let substates = self
            .api
            .kernel_take_substates(node_id, module_id, count)?
            .into_iter()
            .map(|value| value.into())
            .collect();

        Ok(substates)
    }
}

impl<'a, Y, V> ClientSortedIndexApi<RuntimeError> for SystemDownstream<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
    fn new_sorted_index(&mut self) -> Result<NodeId, RuntimeError> {
        let entity_type = EntityType::InternalSortedIndex;
        let node_id = self.api.kernel_allocate_node_id(entity_type)?;

        self.api.kernel_create_node(
            node_id,
            btreemap!(
                SysModuleId::User.into() => btreemap!(),
                SysModuleId::TypeInfo.into() => ModuleInit::TypeInfo(
                    TypeInfoSubstate::SortedStore
                ).to_substates(),
            ),
        )?;

        Ok(node_id)
    }

    #[trace_resources]
    fn insert_into_sorted_index(
        &mut self,
        node_id: &NodeId,
        sorted_key: SortedKey,
        buffer: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        let type_info = TypeInfoBlueprint::get_type(&node_id, self.api)?;
        match type_info {
            TypeInfoSubstate::SortedStore => {}
            _ => {
                return Err(RuntimeError::SystemError(SystemError::NotASortedStore));
            }
        }

        let value = IndexedScryptoValue::from_vec(buffer).map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        if !value.owned_node_ids().is_empty() {
            return Err(RuntimeError::SystemError(
                SystemError::CannotStoreOwnedInIterable,
            ));
        }

        let module_id = SysModuleId::User.into();
        let substate_key = SubstateKey::Sorted(sorted_key.0, sorted_key.1);
        self.api
            .kernel_set_substate(node_id, module_id, substate_key, value)
    }

    #[trace_resources]
    fn scan_sorted_index(
        &mut self,
        node_id: &NodeId,
        count: u32,
    ) -> Result<Vec<Vec<u8>>, RuntimeError> {
        let type_info = TypeInfoBlueprint::get_type(&node_id, self.api)?;
        match type_info {
            TypeInfoSubstate::SortedStore => {}
            _ => {
                return Err(RuntimeError::SystemError(SystemError::NotASortedStore));
            }
        }

        let substates = self
            .api
            .kernel_scan_sorted_substates(node_id, SysModuleId::User.into(), count)?
            .into_iter()
            .map(|value| value.into())
            .collect();

        Ok(substates)
    }

    #[trace_resources]
    fn remove_from_sorted_index(
        &mut self,
        node_id: &NodeId,
        sorted_key: &SortedKey,
    ) -> Result<Option<Vec<u8>>, RuntimeError> {
        let type_info = TypeInfoBlueprint::get_type(&node_id, self.api)?;
        match type_info {
            TypeInfoSubstate::SortedStore => {}
            _ => {
                return Err(RuntimeError::SystemError(SystemError::NotASortedStore));
            }
        }

        let module_id = SysModuleId::User.into();
        let substate_key = SubstateKey::Sorted(sorted_key.0, sorted_key.1.clone());

        let rtn = self
            .api
            .kernel_remove_substate(node_id, module_id, &substate_key)?
            .map(|v| v.into());

        Ok(rtn)
    }
}

impl<'a, Y, V> ClientBlueprintApi<RuntimeError> for SystemDownstream<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
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
                RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
            })?,
            sys_invocation: SystemInvocation {
                blueprint: identifier.0,
                ident: FnIdent::Application(identifier.1),
                receiver: None,
            },
            payload_size,
        };

        self.api
            .kernel_invoke(Box::new(invocation))
            .map(|v| v.into())
    }
}

impl<'a, Y, V> ClientCostingApi<RuntimeError> for SystemDownstream<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources(log=units, log=reason)]
    fn consume_cost_units(
        &mut self,
        units: u32,
        reason: ClientCostingReason,
    ) -> Result<(), RuntimeError> {
        // No costing applied

        self.api
            .kernel_get_callback()
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

    #[trace_resources]
    fn credit_cost_units(
        &mut self,
        vault_id: NodeId,
        locked_fee: LiquidFungibleResource,
        contingent: bool,
    ) -> Result<LiquidFungibleResource, RuntimeError> {
        // No costing applied

        self.api
            .kernel_get_callback()
            .modules
            .costing
            .credit_cost_units(vault_id, locked_fee, contingent)
    }
}

impl<'a, Y, V> ClientActorApi<RuntimeError> for SystemDownstream<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
    fn lock_field(&mut self, field: u8, flags: LockFlags) -> Result<LockHandle, RuntimeError> {
        let actor = self.api.kernel_get_current_actor().unwrap();
        let (node_id, object_module_id, blueprint) = match &actor {
            Actor::Function { .. } | Actor::VirtualLazyLoad { .. } => {
                return Err(RuntimeError::SystemError(SystemError::NotAMethod))
            }
            Actor::Method {
                node_id,
                module_id,
                blueprint,
                ..
            } => (node_id, module_id, blueprint),
        };

        // TODO: Remove
        if flags.contains(LockFlags::UNMODIFIED_BASE) || flags.contains(LockFlags::FORCE_WRITE) {
            if !(blueprint.package_address.eq(&RESOURCE_MANAGER_PACKAGE)
                && blueprint.blueprint_name.eq(FUNGIBLE_VAULT_BLUEPRINT))
            {
                return Err(RuntimeError::SystemError(SystemError::InvalidLockFlags));
            }
        }

        // Check if valid field_index
        let schema = self.get_blueprint_schema(blueprint)?;
        if !schema.has_field(field) {
            return Err(RuntimeError::SystemError(SystemError::FieldDoesNotExist(blueprint.clone(), field)));
        }

        let sys_module_id = match object_module_id {
            ObjectModuleId::Metadata => SysModuleId::Metadata,
            ObjectModuleId::Royalty => SysModuleId::Royalty,
            ObjectModuleId::AccessRules => SysModuleId::AccessRules,
            ObjectModuleId::SELF => SysModuleId::User,
        };
        let substate_key = SubstateKey::Tuple(field);

        self.api.kernel_lock_substate(
                &node_id,
                sys_module_id.into(),
                &substate_key,
                flags,
                SystemLockData::default(),
        )
    }

    #[trace_resources]
    fn lock_parent_field(
        &mut self,
        field: u8,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError> {
        let actor = self.api.kernel_get_current_actor().unwrap();
        let (node_id, object_module_id, blueprint) = match &actor {
            Actor::Function { .. } | Actor::VirtualLazyLoad { .. } => {
                return Err(RuntimeError::SystemError(SystemError::NotAMethod))
            }
            Actor::Method {
                node_id,
                module_id,
                blueprint,
                ..
            } => (node_id, module_id, blueprint),
        };

        let parent = self
            .get_info()?
            .blueprint_parent
            .ok_or(RuntimeError::SystemError(SystemError::NoParent))?;

        // TODO: Check if valid substate_key for node_id
        self.api.kernel_lock_substate(
            parent.as_node_id(),
            SysModuleId::User.into(),
            &SubstateKey::Tuple(field),
            flags,
            SystemLockData::default(),
        )
    }


    #[trace_resources]
    fn lock_key_value_entry(&mut self, key: &Vec<u8>, flags: LockFlags) -> Result<LockHandle, RuntimeError> {
        let actor = self.api.kernel_get_current_actor().unwrap();
        let (node_id, object_module_id, blueprint) = match &actor {
            Actor::Function { .. } | Actor::VirtualLazyLoad { .. } => {
                return Err(RuntimeError::SystemError(SystemError::NotAMethod))
            }
            Actor::Method {
                node_id,
                module_id,
                blueprint,
                ..
            } => (node_id, module_id, blueprint),
        };

        // TODO: Add check
        /*
        let schema = self.get_blueprint_schema(blueprint)?;

        if !schema.has_field(field) {
            return Err(RuntimeError::SystemError(SystemError::FieldDoesNotExist(blueprint.clone(), field)));
        }
         */

        let sys_module_id = match object_module_id {
            ObjectModuleId::Metadata => SysModuleId::Metadata,
            ObjectModuleId::Royalty => SysModuleId::Royalty,
            ObjectModuleId::AccessRules => SysModuleId::AccessRules,
            ObjectModuleId::SELF => SysModuleId::User,
        };

        self.api.kernel_lock_substate_with_default(
            &node_id,
            sys_module_id.into(),
            &SubstateKey::Map(key.clone()),
            flags,
            Some(|| IndexedScryptoValue::from_typed(&Option::<ScryptoValue>::None)),
            SystemLockData {
                is_kv_store: true,
            },
        )
    }

    #[trace_resources]
    fn get_info(&mut self) -> Result<ObjectInfo, RuntimeError> {
        let actor = self.api.kernel_get_current_actor().unwrap();
        let (node_id, module_id) = match &actor {
            Actor::Function { .. } | Actor::VirtualLazyLoad { .. } => {
                return Err(RuntimeError::SystemError(SystemError::NotAnObject))
            }
            Actor::Method {
                node_id, module_id, ..
            } => (node_id, module_id),
        };

        let info = match module_id {
            ObjectModuleId::SELF => self.get_object_info(node_id)?,
            ObjectModuleId::Metadata => ObjectInfo {
                blueprint: Blueprint::new(&METADATA_PACKAGE, METADATA_BLUEPRINT),
                global: true,
                blueprint_parent: None,
            },
            ObjectModuleId::AccessRules => ObjectInfo {
                blueprint: Blueprint::new(&ACCESS_RULES_PACKAGE, ACCESS_RULES_BLUEPRINT),
                global: true,
                blueprint_parent: None,
            },
            ObjectModuleId::Royalty => ObjectInfo {
                blueprint: Blueprint::new(&ROYALTY_PACKAGE, COMPONENT_ROYALTY_BLUEPRINT),
                global: true,
                blueprint_parent: None,
            },
        };

        Ok(info)
    }

    #[trace_resources]
    fn get_global_address(&mut self) -> Result<GlobalAddress, RuntimeError> {
        let actor = self.api.kernel_get_current_actor().unwrap();
        match actor {
            Actor::Method {
                global_address: Some(address),
                ..
            } => Ok(address),
            _ => Err(RuntimeError::SystemError(
                SystemError::GlobalAddressDoesNotExist,
            )),
        }
    }

    fn get_blueprint(&mut self) -> Result<Blueprint, RuntimeError> {
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        let actor = self.api.kernel_get_current_actor().unwrap();
        Ok(actor.blueprint().clone())
    }
}

impl<'a, Y, V> ClientAuthApi<RuntimeError> for SystemDownstream<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
    fn get_auth_zone(&mut self) -> Result<NodeId, RuntimeError> {
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        let auth_zone_id = self.api.kernel_get_callback().modules.auth.last_auth_zone();

        Ok(auth_zone_id.into())
    }

    #[trace_resources]
    fn assert_access_rule(&mut self, rule: AccessRule) -> Result<(), RuntimeError> {
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        // Decide `authorization`, `barrier_crossing_allowed`, and `tip_auth_zone_id`
        let authorization = convert_contextless(&rule);
        let barrier_crossings_required = 1;
        let barrier_crossings_allowed = 1;
        let auth_zone_id = self.api.kernel_get_callback().modules.auth.last_auth_zone();

        // Authenticate
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

        Ok(())
    }
}

impl<'a, Y, V> ClientTransactionLimitsApi<RuntimeError> for SystemDownstream<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
    fn update_wasm_memory_usage(&mut self, consumed_memory: usize) -> Result<(), RuntimeError> {
        // No costing applied

        let current_depth = self.api.kernel_get_current_depth();
        self.api
            .kernel_get_callback()
            .modules
            .transaction_limits
            .update_wasm_memory_usage(current_depth, consumed_memory)
    }
}

impl<'a, Y, V> ClientExecutionTraceApi<RuntimeError> for SystemDownstream<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
    fn update_instruction_index(&mut self, new_index: usize) -> Result<(), RuntimeError> {
        // No costing applied

        self.api
            .kernel_get_callback()
            .modules
            .execution_trace
            .update_instruction_index(new_index);
        Ok(())
    }
}

impl<'a, Y, V> ClientEventApi<RuntimeError> for SystemDownstream<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
    fn emit_event(&mut self, event_name: String, event_data: Vec<u8>) -> Result<(), RuntimeError> {
        // Costing event emission.
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        let actor = self.api.kernel_get_current_actor();

        // Locking the package info substate associated with the emitter's package
        let (blueprint_schema, local_type_index) = {
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

            let blueprint_schema = self.get_blueprint_schema(&blueprint)?;

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

            (blueprint_schema, local_type_index)
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
            .kernel_get_callback()
            .modules
            .events
            .add_event(event_type_identifier, event_data);

        Ok(())
    }
}

impl<'a, Y, V> ClientLoggerApi<RuntimeError> for SystemDownstream<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    fn log_message(&mut self, level: Level, message: String) -> Result<(), RuntimeError> {
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        self.api
            .kernel_get_callback()
            .modules
            .logger
            .add_log(level, message);
        Ok(())
    }
}

impl<'a, Y, V> ClientTransactionRuntimeApi<RuntimeError> for SystemDownstream<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
    fn get_transaction_hash(&mut self) -> Result<Hash, RuntimeError> {
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        Ok(self
            .api
            .kernel_get_callback()
            .modules
            .transaction_runtime
            .transaction_hash())
    }

    #[trace_resources]
    fn generate_uuid(&mut self) -> Result<u128, RuntimeError> {
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        Ok(self
            .api
            .kernel_get_callback()
            .modules
            .transaction_runtime
            .generate_uuid())
    }
}

impl<'a, Y, V> ClientApi<RuntimeError> for SystemDownstream<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
}

impl<'a, Y, V> KernelNodeApi for SystemDownstream<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    fn kernel_drop_node(&mut self, node_id: &NodeId) -> Result<NodeSubstates, RuntimeError> {
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
        node_substates: NodeSubstates,
    ) -> Result<(), RuntimeError> {
        self.api.kernel_create_node(node_id, node_substates)
    }
}

impl<'a, Y, V> KernelSubstateApi<SystemLockData> for SystemDownstream<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    fn kernel_lock_substate_with_default(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
        flags: LockFlags,
        default: Option<fn() -> IndexedScryptoValue>,
        data: SystemLockData,
    ) -> Result<LockHandle, RuntimeError> {
        self.api
            .kernel_lock_substate_with_default(node_id, module_id, substate_key, flags, default, data)
    }

    fn kernel_get_lock_info(&mut self, lock_handle: LockHandle) -> Result<LockInfo<SystemLockData>, RuntimeError> {
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

    fn kernel_set_substate(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: SubstateKey,
        value: IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        self.api
            .kernel_set_substate(node_id, module_id, substate_key, value)
    }

    fn kernel_remove_substate(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Result<Option<IndexedScryptoValue>, RuntimeError> {
        self.api
            .kernel_remove_substate(node_id, module_id, substate_key)
    }

    fn kernel_scan_sorted_substates(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        count: u32,
    ) -> Result<Vec<IndexedScryptoValue>, RuntimeError> {
        self.api
            .kernel_scan_sorted_substates(node_id, module_id, count)
    }

    fn kernel_scan_substates(
        &mut self,
        node_id: &NodeId,
        module_id: SysModuleId,
        count: u32,
    ) -> Result<Vec<IndexedScryptoValue>, RuntimeError> {
        self.api.kernel_scan_substates(node_id, module_id, count)
    }

    fn kernel_take_substates(
        &mut self,
        node_id: &NodeId,
        module_id: SysModuleId,
        count: u32,
    ) -> Result<Vec<IndexedScryptoValue>, RuntimeError> {
        self.api.kernel_take_substates(node_id, module_id, count)
    }
}

impl<'a, Y, V> KernelInternalApi<SystemConfig<V>> for SystemDownstream<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    fn kernel_get_callback(&mut self) -> &mut SystemConfig<V> {
        self.api.kernel_get_callback()
    }

    fn kernel_get_current_actor(&mut self) -> Option<Actor> {
        self.api.kernel_get_current_actor()
    }

    fn kernel_get_current_depth(&self) -> usize {
        self.api.kernel_get_current_depth()
    }

    fn kernel_get_node_info(&self, node_id: &NodeId) -> Option<(RefType, bool)> {
        self.api.kernel_get_node_info(node_id)
    }

    fn kernel_load_common(&mut self) {
        self.api.kernel_load_common()
    }

    fn kernel_load_package_package_dependencies(&mut self) {
        self.api.kernel_load_package_package_dependencies()
    }

    fn kernel_read_bucket(&mut self, bucket_id: &NodeId) -> Option<BucketSnapshot> {
        self.api.kernel_read_bucket(bucket_id)
    }

    fn kernel_read_proof(&mut self, proof_id: &NodeId) -> Option<ProofSnapshot> {
        self.api.kernel_read_proof(proof_id)
    }
}
