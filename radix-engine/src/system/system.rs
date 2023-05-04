use super::system_modules::auth::{convert_contextless, Authentication};
use super::system_modules::costing::CostingReason;
use crate::errors::{
    ApplicationError, CreateObjectError, InvalidDropNodeAccess, InvalidModuleSet,
    InvalidModuleType, KernelError, RuntimeError,
};
use crate::errors::{SystemError, SystemUpstreamError};
use crate::kernel::actor::{Actor, InstanceContext, MethodActor};
use crate::kernel::call_frame::RefType;
use crate::kernel::kernel_api::*;
use crate::system::node_init::ModuleInit;
use crate::system::node_modules::type_info::{TypeInfoBlueprint, TypeInfoSubstate};
use crate::system::system_callback::{
    FieldLockData, KeyValueEntryLockData, SystemConfig, SystemInvocation, SystemLockData,
};
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_modules::costing::FIXED_LOW_FEE;
use crate::system::system_modules::events::EventError;
use crate::system::system_modules::execution_trace::{BucketSnapshot, ProofSnapshot};
use crate::track::interface::NodeSubstates;
use crate::types::*;
use radix_engine_interface::api::actor_index_api::ClientActorIndexApi;
use radix_engine_interface::api::actor_sorted_index_api::SortedKey;
use radix_engine_interface::api::field_lock_api::{FieldLockHandle, LockFlags};
use radix_engine_interface::api::key_value_entry_api::{
    ClientKeyValueEntryApi, KeyValueEntryHandle,
};
use radix_engine_interface::api::key_value_store_api::ClientKeyValueStoreApi;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::clock::CLOCK_BLUEPRINT;
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::schema::{BlueprintKeyValueStoreSchema, BlueprintPartitionSchema, IndexedBlueprintSchema, InstanceSchema, KeyValueStoreInfo, TypeSchema};
use resources_tracker_macro::trace_resources;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;

/// Provided to upper layer for invoking lower layer service
pub struct SystemService<'a, Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject> {
    pub api: &'a mut Y,
    pub phantom: PhantomData<V>,
}

enum ActorObjectType {
    SELF,
    OuterObject,
}

impl TryFrom<ObjectHandle> for ActorObjectType {
    type Error = RuntimeError;
    fn try_from(value: ObjectHandle) -> Result<Self, Self::Error> {
        match value {
            OBJECT_HANDLE_SELF => Ok(ActorObjectType::SELF),
            OBJECT_HANDLE_OUTER_OBJECT => Ok(ActorObjectType::OuterObject),
            _ => Err(RuntimeError::SystemError(SystemError::InvalidObjectHandle)),
        }
    }
}

impl<'a, Y, V> SystemService<'a, Y, V>
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

    fn validate_payload_against_blueprint_and_instance_schema<'s>(
        &'s mut self,
        payload: &Vec<u8>,
        type_schema: &TypeSchema,
        blueprint_schema: &'s ScryptoSchema,
        instance_schema: &'s Option<InstanceSchema>,
    ) -> Result<(), LocatedValidationError<ScryptoCustomTypeExtension>> {
        match type_schema {
            TypeSchema::Blueprint(index) => {
                validate_payload_against_schema(payload, blueprint_schema, index.clone(), self)?;
            }
            TypeSchema::Instance(instance_index) => {
                let instance_schema = instance_schema.as_ref().unwrap();
                let index = instance_schema
                    .type_index
                    .get(*instance_index as usize)
                    .unwrap()
                    .clone();

                validate_payload_against_schema(&payload, &instance_schema.schema, index, self)?;
            }
        }

        Ok(())
    }

    pub fn get_node_type_info(&mut self, node_id: &NodeId) -> Option<TypeInfoSubstate> {
        // This is to solve the bootstrapping problem.
        // TODO: Can be removed if we flush bootstrap state updates without transactional execution.
        if node_id.eq(RADIX_TOKEN.as_node_id()) {
            return Some(TypeInfoSubstate::Object(ObjectInfo {
                blueprint: Blueprint {
                    package_address: RESOURCE_PACKAGE,
                    blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                },
                global: true,
                outer_object: None,
                instance_schema: None,
            }));
        } else if node_id.eq(ECDSA_SECP256K1_SIGNATURE_VIRTUAL_BADGE.as_node_id())
            || node_id.eq(EDDSA_ED25519_SIGNATURE_VIRTUAL_BADGE.as_node_id())
            || node_id.eq(SYSTEM_TRANSACTION_BADGE.as_node_id())
            || node_id.eq(PACKAGE_VIRTUAL_BADGE.as_node_id())
            || node_id.eq(GLOBAL_ACTOR_VIRTUAL_BADGE.as_node_id())
            || node_id.eq(PACKAGE_OWNER_BADGE.as_node_id())
            || node_id.eq(VALIDATOR_OWNER_BADGE.as_node_id())
            || node_id.eq(IDENTITY_OWNER_BADGE.as_node_id())
            || node_id.eq(ACCOUNT_OWNER_BADGE.as_node_id())
        {
            return Some(TypeInfoSubstate::Object(ObjectInfo {
                blueprint: Blueprint {
                    package_address: RESOURCE_PACKAGE,
                    blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                },
                global: true,
                outer_object: None,
                instance_schema: None,
            }));
        }

        self.api
            .kernel_lock_substate(
                node_id,
                TYPE_INFO_BASE_PARTITION,
                &TypeInfoField::TypeInfo.into(),
                LockFlags::read_only(),
                SystemLockData::default(),
            )
            .and_then(|lock_handle| {
                self.api
                    .kernel_read_substate(lock_handle)
                    .and_then(|x| Ok(x.as_typed::<TypeInfoSubstate>().unwrap()))
                    .and_then(|substate| {
                        self.api
                            .kernel_drop_lock(lock_handle)
                            .and_then(|_| Ok(substate))
                    })
            })
            .ok()
    }

    fn new_object_internal(
        &mut self,
        blueprint: &Blueprint,
        instance_context: Option<InstanceContext>,
        instance_schema: Option<InstanceSchema>,
        fields: Vec<Vec<u8>>,
        kv_entries: BTreeMap<u8, BTreeMap<Vec<u8>, Vec<u8>>>,
    ) -> Result<NodeId, RuntimeError> {
        let (expected_blueprint_parent, user_substates) =
            self.verify_instance_schema_and_state(blueprint, &instance_schema, fields, kv_entries)?;

        let outer_object = if let Some(parent) = &expected_blueprint_parent {
            match instance_context {
                Some(context) if context.instance_blueprint.eq(parent) => Some(context.instance),
                _ => {
                    return Err(RuntimeError::SystemError(
                        SystemError::InvalidChildObjectCreation,
                    ));
                }
            }
        } else {
            None
        };

        let node_id = {
            let entity_type = match (blueprint.package_address, blueprint.blueprint_name.as_str()) {
                (RESOURCE_PACKAGE, FUNGIBLE_VAULT_BLUEPRINT) => EntityType::InternalFungibleVault,
                (RESOURCE_PACKAGE, NON_FUNGIBLE_VAULT_BLUEPRINT) => {
                    EntityType::InternalNonFungibleVault
                }
                (ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT) => EntityType::InternalAccount,
                _ => EntityType::InternalGenericComponent,
            };

            self.api.kernel_allocate_node_id(entity_type)?
        };

        let mut node_substates = btreemap!(
            TYPE_INFO_BASE_PARTITION => ModuleInit::TypeInfo(
                TypeInfoSubstate::Object(ObjectInfo {
                    blueprint: blueprint.clone(),
                    global:false,
                    outer_object,
                    instance_schema,
                })
            ).to_substates(),
        );

        for (i, partition) in user_substates.into_iter().enumerate() {
            let offset = PartitionOffset(i as u8);
            let partition_num = OBJECT_BASE_PARTITION
                .at_offset(offset)
                .expect("Module number overflow");
            node_substates.insert(partition_num, partition);
        }

        self.api.kernel_create_node(node_id, node_substates)?;

        Ok(node_id.into())
    }

    fn get_blueprint_schema(
        &mut self,
        blueprint: &Blueprint,
    ) -> Result<IndexedBlueprintSchema, RuntimeError> {
        let schema = self
            .api
            .kernel_get_system_state()
            .system
            .blueprint_schema_cache
            .get(blueprint);
        if let Some(schema) = schema {
            return Ok(schema.clone());
        } else {
            let handle = self.api.kernel_lock_substate(
                blueprint.package_address.as_node_id(),
                OBJECT_BASE_PARTITION,
                &PackageField::Info.into(),
                LockFlags::read_only(),
                SystemLockData::default(),
            )?;

            // TODO: We really need to split up PackageInfo into multiple substates
            let mut package: PackageInfoSubstate =
                self.api.kernel_read_substate(handle)?.as_typed().unwrap();
            let schema = package
                .schema
                .blueprints
                .remove(blueprint.blueprint_name.as_str())
                .ok_or(RuntimeError::SystemError(SystemError::CreateObjectError(
                    Box::new(CreateObjectError::BlueprintNotFound(
                        blueprint.blueprint_name.to_string(),
                    )),
                )))?;
            self.api
                .kernel_get_system_state()
                .system
                .blueprint_schema_cache
                .insert(blueprint.clone(), schema);
            self.api.kernel_drop_lock(handle)?;
            let schema = self
                .api
                .kernel_get_system_state()
                .system
                .blueprint_schema_cache
                .get(blueprint)
                .unwrap();
            Ok(schema.clone())
        }
    }

    fn verify_instance_schema_and_state(
        &mut self,
        blueprint: &Blueprint,
        instance_schema: &Option<InstanceSchema>,
        mut fields: Vec<Vec<u8>>,
        mut kv_entries: BTreeMap<u8, BTreeMap<Vec<u8>, Vec<u8>>>,
    ) -> Result<
        (
            Option<String>,
            Vec<BTreeMap<SubstateKey, IndexedScryptoValue>>,
        ),
        RuntimeError,
    > {
        let blueprint_schema = self.get_blueprint_schema(blueprint)?;

        // Validate instance schema
        {
            if let Some(instance_schema) = instance_schema {
                validate_schema(&instance_schema.schema)
                    .map_err(|_| RuntimeError::SystemError(SystemError::InvalidInstanceSchema))?;
            }
            if !blueprint_schema.validate_instance_schema(instance_schema) {
                return Err(RuntimeError::SystemError(
                    SystemError::InvalidInstanceSchema,
                ));
            }
        }

        let mut user_substates = Vec::new();

        {
            for (i, blueprint_partition_schema) in blueprint_schema.partitions.iter().enumerate() {
                let handle = i as u8;
                let mut partition = BTreeMap::new();
                match blueprint_partition_schema {
                    BlueprintPartitionSchema::Fields(field_type_index) => {
                        let entries = kv_entries.remove(&handle);
                        if entries.is_some() {
                            return Err(RuntimeError::SystemError(SystemError::CreateObjectError(
                                Box::new(CreateObjectError::InvalidModule),
                            )));
                        }

                        if field_type_index.len() != fields.len() {
                            return Err(RuntimeError::SystemError(SystemError::CreateObjectError(
                                Box::new(CreateObjectError::WrongNumberOfSubstates(
                                    blueprint.clone(),
                                    fields.len(),
                                    field_type_index.len(),
                                )),
                            )));
                        }
                        let field_type_index = field_type_index.clone();
                        for (i, field) in fields.drain(..).enumerate() {
                            validate_payload_against_schema(
                                &field,
                                &blueprint_schema.schema,
                                field_type_index[i],
                                self,
                            )
                            .map_err(|err| {
                                RuntimeError::SystemError(SystemError::CreateObjectError(Box::new(
                                    CreateObjectError::InvalidSubstateWrite(
                                        err.error_message(&blueprint_schema.schema),
                                    ),
                                )))
                            })?;

                            partition.insert(
                                SubstateKey::Tuple(i as u8),
                                IndexedScryptoValue::from_vec(field)
                                    .expect("Checked by payload-schema validation"),
                            );
                        }
                    }
                    BlueprintPartitionSchema::KeyValueStore(blueprint_kv_schema) => {
                        let entries = kv_entries.remove(&handle);
                        if let Some(entries) = entries {
                            for (key, value) in entries {
                                self.validate_payload_against_blueprint_and_instance_schema(
                                    &key,
                                    &blueprint_kv_schema.key,
                                    &blueprint_schema.schema,
                                    instance_schema,
                                )
                                .map_err(|err| {
                                    RuntimeError::SystemError(SystemError::CreateObjectError(
                                        Box::new(CreateObjectError::InvalidSubstateWrite(
                                            err.error_message(&blueprint_schema.schema),
                                        )),
                                    ))
                                })?;

                                self.validate_payload_against_blueprint_and_instance_schema(
                                    &value,
                                    &blueprint_kv_schema.value,
                                    &blueprint_schema.schema,
                                    instance_schema,
                                )
                                .map_err(|err| {
                                    RuntimeError::SystemError(SystemError::CreateObjectError(
                                        Box::new(CreateObjectError::InvalidSubstateWrite(
                                            err.error_message(&blueprint_schema.schema),
                                        )),
                                    ))
                                })?;

                                let value: ScryptoValue = scrypto_decode(&value).unwrap();
                                let value = IndexedScryptoValue::from_typed(&Some(value));

                                if !blueprint_kv_schema.can_own {
                                    if !value.owned_node_ids().is_empty() {
                                        return Err(RuntimeError::SystemError(
                                            SystemError::InvalidKeyValueStoreOwnership,
                                        ));
                                    }
                                }

                                partition.insert(SubstateKey::Map(key), value);
                            }
                        }
                    }
                    _ => {
                        let entries = kv_entries.remove(&handle);
                        if entries.is_some() {
                            return Err(RuntimeError::SystemError(SystemError::CreateObjectError(
                                Box::new(CreateObjectError::InvalidModule),
                            )));
                        }
                    }
                }

                user_substates.push(partition);
            }

            if !fields.is_empty() {
                return Err(RuntimeError::SystemError(SystemError::CreateObjectError(
                    Box::new(CreateObjectError::InvalidModule),
                )));
            }

            if !kv_entries.is_empty() {
                return Err(RuntimeError::SystemError(SystemError::CreateObjectError(
                    Box::new(CreateObjectError::InvalidModule),
                )));
            }
        }

        let parent_blueprint = blueprint_schema.outer_blueprint.clone();

        Ok((parent_blueprint, user_substates))
    }

    fn key_value_entry_remove_and_release_lock(
        &mut self,
        handle: KeyValueEntryHandle,
    ) -> Result<Vec<u8>, RuntimeError> {
        // TODO: Replace with api::replace
        let current_value = self
            .api
            .kernel_read_substate(handle)
            .map(|v| v.as_slice().to_vec())?;
        self.kernel_write_substate(
            handle,
            IndexedScryptoValue::from_typed(&None::<ScryptoValue>),
        )?;
        self.kernel_drop_lock(handle)?;
        Ok(current_value)
    }

    fn get_actor_schema(
        &mut self,
        actor_object_type: ActorObjectType,
    ) -> Result<(NodeId, PartitionNumber, ObjectInfo, IndexedBlueprintSchema), RuntimeError> {
        let actor = self.api.kernel_get_system_state().current.unwrap();
        let method = actor
            .try_as_method()
            .ok_or_else(|| RuntimeError::SystemError(SystemError::NotAMethod))?;
        match actor_object_type {
            ActorObjectType::OuterObject => {
                let address = method.object_info.outer_object.unwrap();
                let info = self.get_object_info(address.as_node_id())?;
                let schema = self.get_blueprint_schema(&info.blueprint)?;
                Ok((
                    address.into_node_id(),
                    OBJECT_BASE_PARTITION,
                    info,
                    schema,
                ))
            }
            ActorObjectType::SELF => {
                let node_id = method.node_id;
                let info = method.object_info.clone();
                let object_module_id = method.module_id;
                let schema = self.get_blueprint_schema(&info.blueprint)?;
                Ok((
                    node_id,
                    object_module_id.base_partition_num(),
                    info,
                    schema,
                ))
            }
        }
    }

    fn get_actor_kv_partition(
        &mut self,
        actor_object_type: ActorObjectType,
        partition_index: u8,
    ) -> Result<(NodeId, PartitionNumber, ScryptoSchema, BlueprintKeyValueStoreSchema, ObjectInfo), RuntimeError> {
        let (node_id, base_partition, info, schema) =
            self.get_actor_schema(actor_object_type)?;

        let (partition_offset, schema, kv_schema) = schema.key_value_store_partition(partition_index).ok_or_else(|| {
            RuntimeError::SystemError(SystemError::IndexDoesNotExist(info.blueprint.clone(), partition_index))
        })?;

        let partition_num = base_partition
            .at_offset(partition_offset)
            .expect("Module number overflow");

        Ok((node_id, partition_num, schema, kv_schema, info))
    }

    fn get_actor_index(
        &mut self,
        actor_object_type: ActorObjectType,
        handle: u8,
    ) -> Result<(NodeId, PartitionNumber), RuntimeError> {
        let (node_id, base_partition, object_info, schema) =
            self.get_actor_schema(actor_object_type)?;

        let (partition_offset, _) = schema.index_partition_offset(handle).ok_or_else(|| {
            RuntimeError::SystemError(SystemError::IndexDoesNotExist(object_info.blueprint, handle))
        })?;

        let partition_num = base_partition
            .at_offset(partition_offset)
            .expect("Module number overflow");

        Ok((node_id, partition_num))
    }

    fn get_actor_sorted_index(
        &mut self,
        actor_object_type: ActorObjectType,
        index_handle: u8,
    ) -> Result<(NodeId, PartitionNumber), RuntimeError> {
        let (node_id, base_partition, object_info, schema) =
            self.get_actor_schema(actor_object_type)?;

        let (partition_offset, _) = schema
            .sorted_index_partition_offset(index_handle)
            .ok_or_else(|| {
                RuntimeError::SystemError(SystemError::IndexDoesNotExist(object_info.blueprint, index_handle))
            })?;

        let partition_num = base_partition
            .at_offset(partition_offset)
            .expect("Module number overflow");

        Ok((node_id, partition_num))
    }
}

impl<'a, Y, V> ClientFieldLockApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
    fn field_lock_read(&mut self, lock_handle: FieldLockHandle) -> Result<Vec<u8>, RuntimeError> {
        let LockInfo { data, .. } = self.api.kernel_get_lock_info(lock_handle)?;
        match data {
            SystemLockData::Field(..) => {}
            _ => {
                return Err(RuntimeError::SystemError(SystemError::NotAFieldLock));
            }
        }

        self.api
            .kernel_read_substate(lock_handle)
            .map(|v| v.as_slice().to_vec())
    }

    #[trace_resources]
    fn field_lock_write(
        &mut self,
        lock_handle: FieldLockHandle,
        buffer: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        let LockInfo { data, .. } = self.api.kernel_get_lock_info(lock_handle)?;

        match data {
            SystemLockData::Field(FieldLockData::Write { index, schema }) => {
                if let Err(e) = validate_payload_against_schema(&buffer, &schema, index, self) {
                    return Err(RuntimeError::SystemError(
                        SystemError::InvalidSubstateWrite(e.error_message(&schema)),
                    ));
                };
            }
            _ => {
                return Err(RuntimeError::SystemError(SystemError::NotAFieldWriteLock));
            }
        }

        let substate =
            IndexedScryptoValue::from_vec(buffer).expect("Should be valid due to payload check");
        self.api.kernel_write_substate(lock_handle, substate)?;

        Ok(())
    }

    #[trace_resources]
    fn field_lock_release(&mut self, handle: FieldLockHandle) -> Result<(), RuntimeError> {
        let LockInfo { data, .. } = self.api.kernel_get_lock_info(handle)?;
        match data {
            SystemLockData::Field(..) => {}
            _ => {
                return Err(RuntimeError::SystemError(SystemError::NotAFieldLock));
            }
        }

        self.api.kernel_drop_lock(handle)
    }
}

impl<'a, Y, V> ClientObjectApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
    fn new_object(
        &mut self,
        blueprint_ident: &str,
        schema: Option<InstanceSchema>,
        fields: Vec<Vec<u8>>,
        kv_entries: BTreeMap<u8, BTreeMap<Vec<u8>, Vec<u8>>>,
    ) -> Result<NodeId, RuntimeError> {
        let actor = self.api.kernel_get_system_state().current.unwrap();
        let package_address = actor.package_address().clone();
        let instance_context = actor.instance_context();
        let blueprint = Blueprint::new(&package_address, blueprint_ident);

        self.new_object_internal(&blueprint, instance_context, schema, fields, kv_entries)
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
            (RESOURCE_PACKAGE, FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT) => {
                EntityType::GlobalFungibleResource
            }
            (RESOURCE_PACKAGE, NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT) => {
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
        let global_address = GlobalAddress::new_or_panic(global_node_id.into());

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
        self.api
            .kernel_get_system_state()
            .system
            .modules
            .events
            .add_replacement(
                (node_id, ObjectModuleId::SELF),
                (*address.as_node_id(), ObjectModuleId::SELF),
            );
        let mut node_substates = self.api.kernel_drop_node(&node_id)?;

        // Update the `global` flag of the type info substate.
        let type_info_module = node_substates
            .get_mut(&TYPE_INFO_BASE_PARTITION)
            .unwrap()
            .remove(&TypeInfoField::TypeInfo.into())
            .unwrap();
        let mut type_info: TypeInfoSubstate = type_info_module.as_typed().unwrap();
        match type_info {
            TypeInfoSubstate::Object(ObjectInfo { ref mut global, .. }) if !*global => {
                *global = true
            }
            _ => return Err(RuntimeError::SystemError(SystemError::CannotGlobalize)),
        };
        node_substates
            .get_mut(&TYPE_INFO_BASE_PARTITION)
            .unwrap()
            .insert(
                TypeInfoField::TypeInfo.into(),
                IndexedScryptoValue::from_typed(&type_info),
            );

        //  Drop the module nodes and move the substates to the designated module ID.
        for (module_id, node_id) in modules {
            match module_id {
                ObjectModuleId::SELF => panic!("Should have been removed already"),
                ObjectModuleId::AccessRules
                | ObjectModuleId::Metadata
                | ObjectModuleId::Royalty => {
                    let blueprint = self.get_object_info(&node_id)?.blueprint;
                    let expected_blueprint = module_id.static_blueprint().unwrap();
                    if !blueprint.eq(&expected_blueprint) {
                        return Err(RuntimeError::SystemError(SystemError::InvalidModuleType(
                            Box::new(InvalidModuleType {
                                expected_blueprint,
                                actual_blueprint: blueprint,
                            }),
                        )));
                    }

                    self.api
                        .kernel_get_system_state()
                        .system
                        .modules
                        .events
                        .add_replacement(
                            (node_id, ObjectModuleId::SELF),
                            (*address.as_node_id(), module_id),
                        );

                    let mut cur_node_substates = self.api.kernel_drop_node(&node_id)?;
                    let self_substates = cur_node_substates.remove(&OBJECT_BASE_PARTITION).unwrap();
                    node_substates.insert(module_id.base_partition_num(), self_substates);
                }
            }
        }

        self.api
            .kernel_create_node(address.into(), node_substates)?;

        Ok(())
    }

    fn globalize_with_address_and_create_inner_object(
        &mut self,
        modules: BTreeMap<ObjectModuleId, NodeId>,
        address: GlobalAddress,
        inner_object_blueprint: &str,
        inner_object_fields: Vec<Vec<u8>>,
    ) -> Result<NodeId, RuntimeError> {
        let node_id = modules
            .get(&ObjectModuleId::SELF)
            .ok_or(RuntimeError::SystemError(SystemError::MissingModule(
                ObjectModuleId::SELF,
            )))?;
        let actor_blueprint = self.get_object_info(node_id)?.blueprint;

        self.globalize_with_address(modules, address)?;

        let blueprint = Blueprint::new(&actor_blueprint.package_address, inner_object_blueprint);

        self.new_object_internal(
            &blueprint,
            Some(InstanceContext {
                instance: address,
                instance_blueprint: actor_blueprint.blueprint_name,
            }),
            None,
            inner_object_fields,
            btreemap!(),
        )
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
        object_module_id: ObjectModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let (object_info, global_address) = match object_module_id {
            ObjectModuleId::SELF => {
                let type_info = TypeInfoBlueprint::get_type(receiver, self.api)?;
                match type_info {
                    TypeInfoSubstate::Object(info @ ObjectInfo { global, .. }) => {
                        let global_address = if global {
                            Some(GlobalAddress::new_or_panic(receiver.clone().into()))
                        } else {
                            // See if we have a parent

                            // TODO: Cleanup, this is a rather crude way of trying to figure out
                            // TODO: whether the node reference is a child of the current parent
                            // TODO: this should be cleaned up once call_frame is refactored
                            let (visibility, on_heap) =
                                self.api.kernel_get_node_info(receiver).unwrap();
                            match (visibility, on_heap) {
                                (RefType::Normal, false) => self
                                    .api
                                    .kernel_get_system_state()
                                    .current
                                    .and_then(|a| match a {
                                        Actor::Method(MethodActor { global_address, .. }) => {
                                            global_address.clone()
                                        }
                                        _ => None,
                                    }),
                                _ => None,
                            }
                        };

                        (info, global_address)
                    }

                    TypeInfoSubstate::KeyValueStore(..) => {
                        return Err(RuntimeError::SystemError(
                            SystemError::CallMethodOnKeyValueStore,
                        ))
                    }
                }
            }
            // TODO: Check if type has these object modules
            ObjectModuleId::Metadata | ObjectModuleId::Royalty | ObjectModuleId::AccessRules => (
                ObjectInfo {
                    blueprint: object_module_id.static_blueprint().unwrap(),
                    outer_object: None,
                    global: true,
                    instance_schema: None,
                },
                None,
            ),
        };

        let identifier =
            MethodIdentifier(receiver.clone(), object_module_id, method_name.to_string());
        let payload_size = args.len() + identifier.2.len();
        let blueprint = object_info.blueprint.clone();

        // TODO: Can we load this lazily when needed?
        let instance_context = if object_info.global {
            match global_address {
                None => None,
                Some(address) => Some(InstanceContext {
                    instance: address,
                    instance_blueprint: object_info.blueprint.blueprint_name.clone(),
                }),
            }
        } else {
            match &object_info.outer_object {
                None => None,
                Some(blueprint_parent) => {
                    // TODO: do this recursively until global?
                    let parent_info = self.get_object_info(blueprint_parent.as_node_id()).unwrap();
                    Some(InstanceContext {
                        instance: blueprint_parent.clone(),
                        instance_blueprint: parent_info.blueprint.blueprint_name.clone(),
                    })
                }
            }
        };

        let invocation = KernelInvocation {
            resolved_actor: Actor::method(
                global_address,
                identifier.clone(),
                object_info,
                instance_context,
            ),
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
            TypeInfoSubstate::KeyValueStore(..) => {
                return Err(RuntimeError::SystemError(SystemError::NotAnObject))
            }
        };

        Ok(object_info)
    }

    #[trace_resources]
    fn drop_object(&mut self, node_id: &NodeId) -> Result<Vec<Vec<u8>>, RuntimeError> {
        let info = self.get_object_info(node_id)?;
        if let Some(blueprint_parent) = info.outer_object {
            let actor = self.api.kernel_get_system_state().current.unwrap();
            let instance_context = actor.instance_context();
            match instance_context {
                Some(instance_context) if instance_context.instance.eq(&blueprint_parent) => {}
                _ => {
                    return Err(RuntimeError::KernelError(
                        KernelError::InvalidDropNodeAccess(Box::new(InvalidDropNodeAccess {
                            node_id: node_id.clone(),
                            package_address: info.blueprint.package_address,
                            blueprint_name: info.blueprint.blueprint_name,
                        })),
                    ));
                }
            }
        }

        let mut node_substates = self.api.kernel_drop_node(&node_id)?;
        let user_substates = node_substates.remove(&OBJECT_BASE_PARTITION).unwrap();
        let fields = user_substates
            .into_iter()
            .map(|(_key, v)| v.into())
            .collect();

        Ok(fields)
    }
}

impl<'a, Y, V> ClientKeyValueEntryApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
    fn key_value_entry_get(
        &mut self,
        handle: KeyValueEntryHandle,
    ) -> Result<Vec<u8>, RuntimeError> {
        let LockInfo { data, .. } = self.api.kernel_get_lock_info(handle)?;
        match data {
            SystemLockData::KeyValueEntry(..) => {}
            _ => {
                return Err(RuntimeError::SystemError(SystemError::NotAKeyValueStore));
            }
        }

        self.api
            .kernel_read_substate(handle)
            .map(|v| v.as_slice().to_vec())
    }

    #[trace_resources]
    fn key_value_entry_set(
        &mut self,
        handle: KeyValueEntryHandle,
        buffer: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        let LockInfo { data, .. } = self.api.kernel_get_lock_info(handle)?;

        let substate = match data {
            SystemLockData::KeyValueEntry(KeyValueEntryLockData::Write {
                schema,
                index,
                can_own,
            }) => {
                if let Err(e) = validate_payload_against_schema(&buffer, &schema, index, self) {
                    return Err(RuntimeError::SystemError(
                        SystemError::InvalidSubstateWrite(e.error_message(&schema)),
                    ));
                };

                let substate = IndexedScryptoValue::from_slice(&buffer)
                    .expect("Should be valid due to payload check");

                if !can_own {
                    let own = substate.owned_node_ids();
                    if !own.is_empty() {
                        return Err(RuntimeError::SystemError(
                            SystemError::InvalidKeyValueStoreOwnership,
                        ));
                    }
                }

                substate
            }
            _ => {
                return Err(RuntimeError::SystemError(
                    SystemError::NotAKeyValueWriteLock,
                ));
            }
        };

        let value = substate.as_scrypto_value().clone();
        let indexed =
            IndexedScryptoValue::from_vec(scrypto_encode(&Option::Some(value)).unwrap()).unwrap();

        self.api.kernel_write_substate(handle, indexed)?;

        Ok(())
    }

    fn key_value_entry_release(&mut self, handle: KeyValueEntryHandle) -> Result<(), RuntimeError> {
        let LockInfo { data, .. } = self.api.kernel_get_lock_info(handle)?;
        if !data.is_kv_entry() {
            return Err(RuntimeError::SystemError(SystemError::NotAKeyValueStore));
        }

        self.api.kernel_drop_lock(handle)
    }
}

impl<'a, Y, V> ClientKeyValueStoreApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
    fn key_value_store_new(&mut self, schema: KeyValueStoreInfo) -> Result<NodeId, RuntimeError> {
        schema
            .schema
            .validate()
            .map_err(|e| RuntimeError::SystemError(SystemError::InvalidKeyValueStoreSchema(e)))?;

        let entity_type = EntityType::InternalKeyValueStore;
        let node_id = self.api.kernel_allocate_node_id(entity_type)?;

        self.api.kernel_create_node(
            node_id,
            btreemap!(
                OBJECT_BASE_PARTITION => btreemap!(),
                TYPE_INFO_BASE_PARTITION => ModuleInit::TypeInfo(
                    TypeInfoSubstate::KeyValueStore(schema)
                ).to_substates(),
            ),
        )?;

        Ok(node_id)
    }

    #[trace_resources]
    fn key_value_store_get_info(
        &mut self,
        node_id: &NodeId,
    ) -> Result<KeyValueStoreInfo, RuntimeError> {
        let type_info = TypeInfoBlueprint::get_type(node_id, self.api)?;
        let schema = match type_info {
            TypeInfoSubstate::Object { .. } => {
                return Err(RuntimeError::SystemError(SystemError::NotAKeyValueStore))
            }
            TypeInfoSubstate::KeyValueStore(schema) => schema,
        };

        Ok(schema)
    }

    #[trace_resources]
    fn key_value_store_lock_entry(
        &mut self,
        node_id: &NodeId,
        key: &Vec<u8>,
        flags: LockFlags,
    ) -> Result<KeyValueEntryHandle, RuntimeError> {
        let type_info = TypeInfoBlueprint::get_type(&node_id, self.api)?;
        if flags.contains(LockFlags::UNMODIFIED_BASE) || flags.contains(LockFlags::FORCE_WRITE) {
            return Err(RuntimeError::SystemError(SystemError::InvalidLockFlags));
        }

        let info = match type_info {
            TypeInfoSubstate::KeyValueStore(info) => info,
            TypeInfoSubstate::Object(..) => {
                return Err(RuntimeError::SystemError(SystemError::NotAKeyValueStore))
            }
        };

        if let Err(e) =
            validate_payload_against_schema(key, &info.schema, info.kv_store_schema.key, self)
        {
            return Err(RuntimeError::SystemError(SystemError::InvalidKeyValueKey(
                e.error_message(&info.schema),
            )));
        };

        let lock_data = if flags.contains(LockFlags::MUTABLE) {
            SystemLockData::KeyValueEntry(KeyValueEntryLockData::Write {
                schema: info.schema,
                index: info.kv_store_schema.value,
                can_own: info.kv_store_schema.can_own,
            })
        } else {
            SystemLockData::KeyValueEntry(KeyValueEntryLockData::Read)
        };

        self.api.kernel_lock_substate_with_default(
            &node_id,
            OBJECT_BASE_PARTITION,
            &SubstateKey::Map(key.clone()),
            flags,
            Some(|| IndexedScryptoValue::from_typed(&Option::<ScryptoValue>::None)),
            lock_data,
        )
    }

    fn key_value_store_remove_entry(
        &mut self,
        node_id: &NodeId,
        key: &Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let handle = self.key_value_store_lock_entry(node_id, key, LockFlags::MUTABLE)?;
        self.key_value_entry_remove_and_release_lock(handle)
    }
}

impl<'a, Y, V> ClientActorIndexApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    fn actor_index_insert(
        &mut self,
        object_handle: ObjectHandle,
        index_id: u8,
        key: Vec<u8>,
        buffer: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, partition_num) = self.get_actor_index(actor_object_type, index_id)?;

        let value = IndexedScryptoValue::from_vec(buffer).map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        if !value.owned_node_ids().is_empty() {
            return Err(RuntimeError::SystemError(
                SystemError::CannotStoreOwnedInIterable,
            ));
        }

        self.api
            .kernel_set_substate(&node_id, partition_num, SubstateKey::Map(key), value)
    }

    fn actor_index_remove(
        &mut self,
        object_handle: ObjectHandle,
        index_id: u8,
        key: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, partition_num) = self.get_actor_index(actor_object_type, index_id)?;

        let rtn = self
            .api
            .kernel_remove_substate(&node_id, partition_num, &SubstateKey::Map(key))?
            .map(|v| v.into());

        Ok(rtn)
    }

    fn actor_index_scan(
        &mut self,
        object_handle: ObjectHandle,
        index_id: u8,
        count: u32,
    ) -> Result<Vec<Vec<u8>>, RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, partition_num) = self.get_actor_index(actor_object_type, index_id)?;

        let substates = self
            .api
            .kernel_scan_substates(&node_id, partition_num, count)?
            .into_iter()
            .map(|value| value.into())
            .collect();

        Ok(substates)
    }

    fn actor_index_take(
        &mut self,
        object_handle: ObjectHandle,
        index_id: u8,
        count: u32,
    ) -> Result<Vec<Vec<u8>>, RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, partition_num) = self.get_actor_index(actor_object_type, index_id)?;

        let substates = self
            .api
            .kernel_take_substates(&node_id, partition_num, count)?
            .into_iter()
            .map(|value| value.into())
            .collect();

        Ok(substates)
    }
}

impl<'a, Y, V> ClientActorSortedIndexApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
    fn actor_sorted_index_insert(
        &mut self,
        object_handle: ObjectHandle,
        partition_index: u8,
        sorted_key: SortedKey,
        buffer: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, partition_num) =
            self.get_actor_sorted_index(actor_object_type, partition_index)?;

        let value = IndexedScryptoValue::from_vec(buffer).map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        if !value.owned_node_ids().is_empty() {
            return Err(RuntimeError::SystemError(
                SystemError::CannotStoreOwnedInIterable,
            ));
        }

        self.api.kernel_set_substate(
            &node_id,
            partition_num,
            SubstateKey::Sorted((sorted_key.0, sorted_key.1)),
            value,
        )
    }

    #[trace_resources]
    fn actor_sorted_index_remove(
        &mut self,
        object_handle: ObjectHandle,
        partition_index: u8,
        sorted_key: &SortedKey,
    ) -> Result<Option<Vec<u8>>, RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, partition_num) =
            self.get_actor_sorted_index(actor_object_type, partition_index)?;

        let rtn = self
            .api
            .kernel_remove_substate(
                &node_id,
                partition_num,
                &SubstateKey::Sorted((sorted_key.0, sorted_key.1.clone())),
            )?
            .map(|v| v.into());

        Ok(rtn)
    }

    #[trace_resources]
    fn actor_sorted_index_scan(
        &mut self,
        object_handle: ObjectHandle,
        partition_index: u8,
        count: u32,
    ) -> Result<Vec<Vec<u8>>, RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, partition_num) =
            self.get_actor_sorted_index(actor_object_type, partition_index)?;

        let substates = self
            .api
            .kernel_scan_sorted_substates(&node_id, partition_num, count)?
            .into_iter()
            .map(|value| value.into())
            .collect();

        Ok(substates)
    }
}

impl<'a, Y, V> ClientBlueprintApi<RuntimeError> for SystemService<'a, Y, V>
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

impl<'a, Y, V> ClientCostingApi<RuntimeError> for SystemService<'a, Y, V>
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

    #[trace_resources]
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

impl<'a, Y, V> ClientActorApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
    fn actor_lock_field(
        &mut self,
        field: u8,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError> {
        let system_state = self.api.kernel_get_system_state();
        let actor = system_state.current.unwrap();
        let method_actor = actor
            .try_as_method()
            .ok_or_else(|| RuntimeError::SystemError(SystemError::NotAMethod))?;

        // TODO: Remove
        if flags.contains(LockFlags::UNMODIFIED_BASE) || flags.contains(LockFlags::FORCE_WRITE) {
            if !(method_actor
                .object_info
                .blueprint
                .package_address
                .eq(&RESOURCE_PACKAGE)
                && method_actor
                    .object_info
                    .blueprint
                    .blueprint_name
                    .eq(FUNGIBLE_VAULT_BLUEPRINT))
            {
                return Err(RuntimeError::SystemError(SystemError::InvalidLockFlags));
            }
        }

        let node_id = method_actor.node_id;
        let base_partition = method_actor.module_id.base_partition_num();
        let blueprint = method_actor.object_info.blueprint.clone();
        let schema = self.get_blueprint_schema(&blueprint)?;

        let (partition_num, field_type_index) =
            if let Some((offset, field_type_index)) = schema.field(field) {
                let partition_num = base_partition
                    .at_offset(offset)
                    .expect("Module number overflow");
                (partition_num, field_type_index)
            } else {
                return Err(RuntimeError::SystemError(SystemError::FieldDoesNotExist(
                    blueprint, field,
                )));
            };

        let lock_data = if flags.contains(LockFlags::MUTABLE) {
            FieldLockData::Write {
                schema: schema.schema.clone(),
                index: field_type_index,
            }
        } else {
            FieldLockData::Read
        };

        self.api.kernel_lock_substate(
            &node_id,
            partition_num,
            &SubstateKey::Tuple(field),
            flags,
            SystemLockData::Field(lock_data),
        )
    }

    #[trace_resources]
    fn actor_lock_outer_object_field(
        &mut self,
        field: u8,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError> {
        let parent = self
            .actor_get_info()?
            .outer_object
            .ok_or(RuntimeError::SystemError(SystemError::NoParent))?;

        let parent_info = self.get_object_info(parent.as_node_id())?;
        let schema = self.get_blueprint_schema(&parent_info.blueprint)?;
        let (partition_offset, field_type_index) = if let Some(field_info) = schema.field(field) {
            field_info
        } else {
            return Err(RuntimeError::SystemError(SystemError::FieldDoesNotExist(
                parent_info.blueprint.clone(),
                field,
            )));
        };

        let partition_num = OBJECT_BASE_PARTITION
            .at_offset(partition_offset)
            .expect("Module number overflow");

        // TODO: Check if valid substate_key for node_id
        let lock_data = if flags.contains(LockFlags::MUTABLE) {
            FieldLockData::Write {
                schema: schema.schema,
                index: field_type_index,
            }
        } else {
            FieldLockData::Read
        };

        self.api.kernel_lock_substate(
            parent.as_node_id(),
            partition_num,
            &SubstateKey::Tuple(field),
            flags,
            SystemLockData::Field(lock_data),
        )
    }

    #[trace_resources]
    fn actor_get_info(&mut self) -> Result<ObjectInfo, RuntimeError> {
        let actor = self.api.kernel_get_system_state().current.unwrap();
        let object_info = actor
            .try_as_method()
            .map(|m| m.object_info.clone())
            .ok_or(RuntimeError::SystemError(SystemError::NotAMethod))?;

        Ok(object_info)
    }

    #[trace_resources]
    fn actor_get_global_address(&mut self) -> Result<GlobalAddress, RuntimeError> {
        let actor = self.api.kernel_get_system_state().current.unwrap();
        match actor {
            Actor::Method(MethodActor {
                global_address: Some(address),
                ..
            }) => Ok(address.clone()),
            _ => Err(RuntimeError::SystemError(
                SystemError::GlobalAddressDoesNotExist,
            )),
        }
    }

    fn actor_get_blueprint(&mut self) -> Result<Blueprint, RuntimeError> {
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        let actor = self.api.kernel_get_system_state().current.unwrap();
        Ok(actor.blueprint().clone())
    }
}

impl<'a, Y, V> ClientActorKeyValueEntryApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
    fn actor_lock_key_value_entry(
        &mut self,
        object_handle: ObjectHandle,
        partition_index: u8,
        key: &[u8],
        flags: LockFlags,
    ) -> Result<KeyValueEntryHandle, RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, partition_num, schema, kv_schema, object_info) =
            self.get_actor_kv_partition(actor_object_type, partition_index)?;

        let lock_data = if flags.contains(LockFlags::MUTABLE) {
            let can_own = kv_schema.can_own;
            match kv_schema.value {
                TypeSchema::Instance(index) => {
                    let mut instance_schema = object_info.instance_schema.unwrap();
                    KeyValueEntryLockData::Write {
                        schema: instance_schema.schema,
                        index: instance_schema.type_index.remove(index as usize),
                        can_own,
                    }
                }
                TypeSchema::Blueprint(index) => KeyValueEntryLockData::Write {
                    schema,
                    index,
                    can_own,
                },
            }
        } else {
            KeyValueEntryLockData::Read
        };

        self.api.kernel_lock_substate_with_default(
            &node_id,
            partition_num,
            &SubstateKey::Map(key.to_vec()),
            flags,
            Some(|| IndexedScryptoValue::from_typed(&Option::<ScryptoValue>::None)),
            SystemLockData::KeyValueEntry(lock_data),
        )
    }

    fn actor_remove_key_value_entry(
        &mut self,
        object_handle: ObjectHandle,
        partition_index: u8,
        key: &Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let handle = self.actor_lock_key_value_entry(object_handle, partition_index, key, LockFlags::MUTABLE)?;
        self.key_value_entry_remove_and_release_lock(handle)
    }
}

impl<'a, Y, V> ClientAuthApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
    fn get_auth_zone(&mut self) -> Result<NodeId, RuntimeError> {
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        let auth_zone_id = self.api.kernel_get_system().modules.auth.last_auth_zone();

        Ok(auth_zone_id.into())
    }

    #[trace_resources]
    fn assert_access_rule(&mut self, rule: AccessRule) -> Result<(), RuntimeError> {
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        // Decide `authorization`, `barrier_crossing_allowed`, and `tip_auth_zone_id`
        let authorization = convert_contextless(&rule);
        let barrier_crossings_required = 1;
        let barrier_crossings_allowed = 1;
        let auth_zone_id = self.api.kernel_get_system().modules.auth.last_auth_zone();

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

impl<'a, Y, V> ClientTransactionLimitsApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
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

impl<'a, Y, V> ClientExecutionTraceApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
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

impl<'a, Y, V> ClientEventApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
    fn emit_event(&mut self, event_name: String, event_data: Vec<u8>) -> Result<(), RuntimeError> {
        // Costing event emission.
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        let actor = self.api.kernel_get_system_state().current.unwrap();

        // Locking the package info substate associated with the emitter's package
        let (blueprint_schema, local_type_index) = {
            // Getting the package address and blueprint name associated with the actor
            let blueprint = match actor {
                Actor::Method(MethodActor {
                    ref object_info, ..
                }) => Ok(object_info.blueprint.clone()),
                Actor::Function { ref blueprint, .. } => Ok(blueprint.clone()),
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
        let actor = self.api.kernel_get_system_state().current.unwrap();
        let event_type_identifier = match actor {
            Actor::Method(MethodActor {
                node_id, module_id, ..
            }) => Ok(EventTypeIdentifier(
                Emitter::Method(node_id.clone(), module_id.clone()),
                local_type_index,
            )),
            Actor::Function { ref blueprint, .. } => Ok(EventTypeIdentifier(
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
            self,
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

        Ok(())
    }
}

impl<'a, Y, V> ClientLoggerApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
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

impl<'a, Y, V> ClientTransactionRuntimeApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
    fn get_transaction_hash(&mut self) -> Result<Hash, RuntimeError> {
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        Ok(self
            .api
            .kernel_get_system()
            .modules
            .transaction_runtime
            .transaction_hash())
    }

    #[trace_resources]
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

impl<'a, Y, V> ClientApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
}

impl<'a, Y, V> KernelNodeApi for SystemService<'a, Y, V>
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

impl<'a, Y, V> KernelSubstateApi<SystemLockData> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    fn kernel_lock_substate_with_default(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        default: Option<fn() -> IndexedScryptoValue>,
        data: SystemLockData,
    ) -> Result<LockHandle, RuntimeError> {
        self.api.kernel_lock_substate_with_default(
            node_id,
            partition_num,
            substate_key,
            flags,
            default,
            data,
        )
    }

    fn kernel_get_lock_info(
        &mut self,
        lock_handle: LockHandle,
    ) -> Result<LockInfo<SystemLockData>, RuntimeError> {
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
        partition_num: PartitionNumber,
        substate_key: SubstateKey,
        value: IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        self.api
            .kernel_set_substate(node_id, partition_num, substate_key, value)
    }

    fn kernel_remove_substate(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Result<Option<IndexedScryptoValue>, RuntimeError> {
        self.api
            .kernel_remove_substate(node_id, partition_num, substate_key)
    }

    fn kernel_scan_sorted_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> Result<Vec<IndexedScryptoValue>, RuntimeError> {
        self.api
            .kernel_scan_sorted_substates(node_id, partition_num, count)
    }

    fn kernel_scan_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> Result<Vec<IndexedScryptoValue>, RuntimeError> {
        self.api
            .kernel_scan_substates(node_id, partition_num, count)
    }

    fn kernel_take_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> Result<Vec<IndexedScryptoValue>, RuntimeError> {
        self.api
            .kernel_take_substates(node_id, partition_num, count)
    }
}

impl<'a, Y, V> KernelInternalApi<SystemConfig<V>> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    fn kernel_get_system_state(&mut self) -> SystemState<'_, SystemConfig<V>> {
        self.api.kernel_get_system_state()
    }

    fn kernel_get_current_depth(&self) -> usize {
        self.api.kernel_get_current_depth()
    }

    fn kernel_get_node_info(&self, node_id: &NodeId) -> Option<(RefType, bool)> {
        self.api.kernel_get_node_info(node_id)
    }

    fn kernel_read_bucket(&mut self, bucket_id: &NodeId) -> Option<BucketSnapshot> {
        self.api.kernel_read_bucket(bucket_id)
    }

    fn kernel_read_proof(&mut self, proof_id: &NodeId) -> Option<ProofSnapshot> {
        self.api.kernel_read_proof(proof_id)
    }
}
