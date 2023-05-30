use super::payload_validation::*;
use super::system_modules::auth::Authorization;
use super::system_modules::costing::CostingReason;
use crate::errors::{
    ApplicationError, CannotGlobalizeError, CreateObjectError, InvalidDropNodeAccess,
    InvalidModuleSet, InvalidModuleType, KernelError, RuntimeError,
};
use crate::errors::{SystemError, SystemUpstreamError};
use crate::kernel::actor::{Actor, InstanceContext, MethodActor};
use crate::kernel::call_frame::{NodeVisibility, Visibility};
use crate::kernel::kernel_api::*;
use crate::system::node_init::ModuleInit;
use crate::system::node_modules::type_info::{TypeInfoBlueprint, TypeInfoSubstate};
use crate::system::system_callback::{
    FieldLockData, KeyValueEntryLockData, SystemConfig, SystemLockData,
};
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_modules::auth::{ActingLocation, AuthorizationCheckResult};
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
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::schema::{
    BlueprintCollectionSchema, BlueprintKeyValueStoreSchema, IndexedBlueprintSchema,
    InstanceSchema, KeyValueStoreSchema, TypeRef,
};
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

    pub fn validate_payload<'s>(
        &mut self,
        payload: &[u8],
        schema: &'s ScryptoSchema,
        type_index: LocalTypeIndex,
        schema_origin: SchemaOrigin,
    ) -> Result<(), LocatedValidationError<'s, ScryptoCustomExtension>> {
        let validation_context: Box<dyn TypeInfoLookup> =
            Box::new(SystemServiceTypeInfoLookup::new(self, schema_origin));
        validate_payload_against_schema::<ScryptoCustomExtension, _>(
            payload,
            schema,
            type_index,
            &validation_context,
        )
    }

    fn validate_payload_against_blueprint_or_instance_schema<'s>(
        &'s mut self,
        payload: &Vec<u8>,
        type_ref: &TypeRef,
        blueprint_schema: &'s ScryptoSchema,
        blueprint_id: BlueprintId,
        instance_schema: &'s Option<InstanceSchema>,
    ) -> Result<(), LocatedValidationError<ScryptoCustomExtension>> {
        match type_ref {
            TypeRef::Blueprint(index) => {
                self.validate_payload(
                    payload,
                    blueprint_schema,
                    *index,
                    SchemaOrigin::Blueprint(blueprint_id),
                )?;
            }
            TypeRef::Instance(instance_index) => {
                let instance_schema = instance_schema.as_ref().unwrap();
                let index = instance_schema
                    .type_index
                    .get(*instance_index as usize)
                    .unwrap()
                    .clone();

                self.validate_payload(
                    payload,
                    &instance_schema.schema,
                    index,
                    SchemaOrigin::Instance,
                )?;
            }
        }

        Ok(())
    }

    pub fn get_node_type_info(&mut self, node_id: &NodeId) -> Option<TypeInfoSubstate> {
        // This is to solve the bootstrapping problem.
        // TODO: Can be removed if we flush bootstrap state updates without transactional execution.
        if node_id.eq(RADIX_TOKEN.as_node_id()) {
            return Some(TypeInfoSubstate::Object(ObjectInfo {
                blueprint: BlueprintId {
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
            || node_id.eq(PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE.as_node_id())
            || node_id.eq(GLOBAL_CALLER_VIRTUAL_BADGE.as_node_id())
            || node_id.eq(PACKAGE_OWNER_BADGE.as_node_id())
            || node_id.eq(VALIDATOR_OWNER_BADGE.as_node_id())
            || node_id.eq(IDENTITY_OWNER_BADGE.as_node_id())
            || node_id.eq(ACCOUNT_OWNER_BADGE.as_node_id())
        {
            return Some(TypeInfoSubstate::Object(ObjectInfo {
                blueprint: BlueprintId {
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
                TYPE_INFO_FIELD_PARTITION,
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
        blueprint: &BlueprintId,
        instance_context: Option<InstanceContext>,
        instance_schema: Option<InstanceSchema>,
        fields: Vec<Vec<u8>>,
        kv_entries: BTreeMap<u8, BTreeMap<Vec<u8>, Vec<u8>>>,
    ) -> Result<NodeId, RuntimeError> {
        let (expected_blueprint_parent, user_substates) =
            self.verify_instance_schema_and_state(blueprint, &instance_schema, fields, kv_entries)?;

        let outer_object = if let Some(parent) = &expected_blueprint_parent {
            match instance_context {
                Some(context) if context.outer_blueprint.eq(parent) => Some(context.outer_object),
                _ => {
                    return Err(RuntimeError::SystemError(
                        SystemError::InvalidChildObjectCreation,
                    ));
                }
            }
        } else {
            None
        };

        let node_id = self.api.kernel_allocate_node_id(
            IDAllocationRequest::Object {
                blueprint_id: blueprint.clone(),
                global: false,
                virtual_node_id: None,
            }
            .entity_type(),
        )?;

        let mut node_substates = btreemap!(
            TYPE_INFO_FIELD_PARTITION => ModuleInit::TypeInfo(
                TypeInfoSubstate::Object(ObjectInfo {
                    blueprint: blueprint.clone(),
                    global:false,
                    outer_object,
                    instance_schema,
                })
            ).to_substates(),
        );

        for (offset, partition) in user_substates.into_iter() {
            let partition_num = OBJECT_BASE_PARTITION
                .at_offset(offset)
                .expect("Module number overflow");
            node_substates.insert(partition_num, partition);
        }

        self.api.kernel_create_node(node_id, node_substates)?;

        Ok(node_id.into())
    }

    pub fn get_blueprint_schema(
        &mut self,
        blueprint: &BlueprintId,
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
                .ok_or(RuntimeError::SystemError(
                    SystemError::BlueprintDoesNotExist(blueprint.clone()),
                ))?;
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
        blueprint: &BlueprintId,
        instance_schema: &Option<InstanceSchema>,
        fields: Vec<Vec<u8>>,
        mut kv_entries: BTreeMap<u8, BTreeMap<Vec<u8>, Vec<u8>>>,
    ) -> Result<
        (
            Option<String>,
            BTreeMap<PartitionOffset, BTreeMap<SubstateKey, IndexedScryptoValue>>,
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

        let mut partitions = BTreeMap::new();

        // Fields
        {
            let expected_num_fields = blueprint_schema.num_fields();
            if expected_num_fields != fields.len() {
                return Err(RuntimeError::SystemError(SystemError::CreateObjectError(
                    Box::new(CreateObjectError::WrongNumberOfSubstates(
                        blueprint.clone(),
                        fields.len(),
                        expected_num_fields,
                    )),
                )));
            }

            if let Some((offset, field_type_index)) = blueprint_schema.fields {
                let mut partition = BTreeMap::new();

                for (i, field) in fields.into_iter().enumerate() {
                    self.validate_payload(
                        &field,
                        &blueprint_schema.schema,
                        field_type_index[i],
                        SchemaOrigin::Blueprint(blueprint.clone()),
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

                partitions.insert(offset, partition);
            }
        }

        // Collections
        {
            for (index, (offset, blueprint_partition_schema)) in
                blueprint_schema.collections.iter().enumerate()
            {
                let index = index as u8;
                let mut partition = BTreeMap::new();
                match blueprint_partition_schema {
                    BlueprintCollectionSchema::KeyValueStore(blueprint_kv_schema) => {
                        let entries = kv_entries.remove(&index);
                        if let Some(entries) = entries {
                            for (key, value) in entries {
                                self.validate_payload_against_blueprint_or_instance_schema(
                                    &key,
                                    &blueprint_kv_schema.key,
                                    &blueprint_schema.schema,
                                    blueprint.clone(),
                                    instance_schema,
                                )
                                .map_err(|err| {
                                    RuntimeError::SystemError(SystemError::CreateObjectError(
                                        Box::new(CreateObjectError::InvalidSubstateWrite(
                                            err.error_message(&blueprint_schema.schema),
                                        )),
                                    ))
                                })?;

                                self.validate_payload_against_blueprint_or_instance_schema(
                                    &value,
                                    &blueprint_kv_schema.value,
                                    &blueprint_schema.schema,
                                    blueprint.clone(),
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
                                    if !value.owned_nodes().is_empty() {
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
                        let entries = kv_entries.remove(&index);
                        if entries.is_some() {
                            return Err(RuntimeError::SystemError(SystemError::CreateObjectError(
                                Box::new(CreateObjectError::InvalidModule),
                            )));
                        }
                    }
                }

                partitions.insert(offset.clone(), partition);
            }

            if !kv_entries.is_empty() {
                return Err(RuntimeError::SystemError(SystemError::CreateObjectError(
                    Box::new(CreateObjectError::InvalidModule),
                )));
            }
        }

        let parent_blueprint = blueprint_schema.outer_blueprint.clone();

        Ok((parent_blueprint, partitions))
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
        let actor = self.api.kernel_get_system_state().current;
        let method = actor
            .try_as_method()
            .ok_or_else(|| RuntimeError::SystemError(SystemError::NotAMethod))?;
        match actor_object_type {
            ActorObjectType::OuterObject => {
                let address = method.module_object_info.outer_object.unwrap();
                let info = self.get_object_info(address.as_node_id())?;
                let schema = self.get_blueprint_schema(&info.blueprint)?;
                Ok((address.into_node_id(), OBJECT_BASE_PARTITION, info, schema))
            }
            ActorObjectType::SELF => {
                let node_id = method.node_id;
                let info = method.module_object_info.clone();
                let object_module_id = method.module_id;
                let schema = self.get_blueprint_schema(&info.blueprint)?;
                Ok((node_id, object_module_id.base_partition_num(), info, schema))
            }
        }
    }

    fn get_actor_field(
        &mut self,
        actor_object_type: ActorObjectType,
        field_index: u8,
    ) -> Result<
        (
            NodeId,
            PartitionNumber,
            ScryptoSchema,
            LocalTypeIndex,
            ObjectInfo,
        ),
        RuntimeError,
    > {
        let (node_id, base_partition, info, schema) = self.get_actor_schema(actor_object_type)?;

        let (partition_offset, type_index) = schema.field(field_index).ok_or_else(|| {
            RuntimeError::SystemError(SystemError::FieldDoesNotExist(
                info.blueprint.clone(),
                field_index,
            ))
        })?;

        let partition_num = base_partition
            .at_offset(partition_offset)
            .expect("Module number overflow");

        Ok((node_id, partition_num, schema.schema, type_index, info))
    }

    fn get_actor_kv_partition(
        &mut self,
        actor_object_type: ActorObjectType,
        collection_index: CollectionIndex,
    ) -> Result<
        (
            NodeId,
            PartitionNumber,
            ScryptoSchema,
            BlueprintKeyValueStoreSchema,
            ObjectInfo,
        ),
        RuntimeError,
    > {
        let (node_id, base_partition, info, schema) = self.get_actor_schema(actor_object_type)?;

        let (partition_offset, schema, kv_schema) = schema
            .key_value_store_partition(collection_index)
            .ok_or_else(|| {
                RuntimeError::SystemError(SystemError::KeyValueStoreDoesNotExist(
                    info.blueprint.clone(),
                    collection_index,
                ))
            })?;

        let partition_num = base_partition
            .at_offset(partition_offset)
            .expect("Module number overflow");

        Ok((node_id, partition_num, schema, kv_schema, info))
    }

    fn get_actor_index(
        &mut self,
        actor_object_type: ActorObjectType,
        collection_index: CollectionIndex,
    ) -> Result<(NodeId, PartitionNumber), RuntimeError> {
        let (node_id, base_partition, object_info, schema) =
            self.get_actor_schema(actor_object_type)?;

        let (partition_offset, _) = schema.index_partition(collection_index).ok_or_else(|| {
            RuntimeError::SystemError(SystemError::IndexDoesNotExist(
                object_info.blueprint,
                collection_index,
            ))
        })?;

        let partition_num = base_partition
            .at_offset(partition_offset)
            .expect("Module number overflow");

        Ok((node_id, partition_num))
    }

    fn get_actor_sorted_index(
        &mut self,
        actor_object_type: ActorObjectType,
        collection_index: CollectionIndex,
    ) -> Result<(NodeId, PartitionNumber), RuntimeError> {
        let (node_id, base_partition, object_info, schema) =
            self.get_actor_schema(actor_object_type)?;

        let (partition_offset, _) =
            schema
                .sorted_index_partition(collection_index)
                .ok_or_else(|| {
                    RuntimeError::SystemError(SystemError::SortedIndexDoesNotExist(
                        object_info.blueprint,
                        collection_index,
                    ))
                })?;

        let partition_num = base_partition
            .at_offset(partition_offset)
            .expect("Module number overflow");

        Ok((node_id, partition_num))
    }

    fn resolve_blueprint_from_modules(
        &mut self,
        modules: &BTreeMap<ObjectModuleId, NodeId>,
    ) -> Result<BlueprintId, RuntimeError> {
        let node_id = modules
            .get(&ObjectModuleId::Main)
            .ok_or(RuntimeError::SystemError(SystemError::MissingModule(
                ObjectModuleId::Main,
            )))?;

        Ok(self.get_object_info(node_id)?.blueprint)
    }

    /// ASSUMPTIONS:
    /// Assumes the caller has already checked that the entity type on the GlobalAddress is valid
    /// against the given self module.
    fn globalize_with_address_internal(
        &mut self,
        mut modules: BTreeMap<ObjectModuleId, NodeId>,
        global_address: GlobalAddress,
    ) -> Result<(), RuntimeError> {
        // Check module configuration
        let module_ids = modules
            .keys()
            .cloned()
            .collect::<BTreeSet<ObjectModuleId>>();
        let standard_object = btreeset!(
            ObjectModuleId::Main,
            ObjectModuleId::Metadata,
            ObjectModuleId::Royalty,
            ObjectModuleId::AccessRules
        );
        if module_ids != standard_object {
            return Err(RuntimeError::SystemError(SystemError::InvalidModuleSet(
                Box::new(InvalidModuleSet(module_ids)),
            )));
        }

        // Read the type info
        let node_id = modules
            .remove(&ObjectModuleId::Main)
            .ok_or(RuntimeError::SystemError(SystemError::MissingModule(
                ObjectModuleId::Main,
            )))?;
        self.api
            .kernel_get_system_state()
            .system
            .modules
            .events
            .add_replacement(
                (node_id, ObjectModuleId::Main),
                (*global_address.as_node_id(), ObjectModuleId::Main),
            );
        let lock_handle = self.api.kernel_lock_substate(
            &node_id,
            TYPE_INFO_FIELD_PARTITION,
            &TypeInfoField::TypeInfo.into(),
            LockFlags::read_only(),
            SystemLockData::Default,
        )?;
        let mut type_info: TypeInfoSubstate = self
            .api
            .kernel_read_substate(lock_handle)?
            .as_typed()
            .unwrap();
        self.api.kernel_drop_lock(lock_handle)?;

        match type_info {
            TypeInfoSubstate::Object(ObjectInfo { ref mut global, .. }) if !*global => {
                *global = true;
            }
            _ => {
                return Err(RuntimeError::SystemError(SystemError::CannotGlobalize(
                    Box::new(CannotGlobalizeError::NotAnObject),
                )))
            }
        };

        // Create a global node
        self.kernel_create_node(
            global_address.into(),
            btreemap!(
                TYPE_INFO_FIELD_PARTITION => ModuleInit::TypeInfo(type_info).to_substates()
            ),
        )?;

        // Move self modules to the newly created global node, and drop
        let mut partition_numbers = self.kernel_list_modules(&node_id)?;
        partition_numbers.remove(&TYPE_INFO_FIELD_PARTITION);
        for partition_number in partition_numbers {
            self.kernel_move_module(
                &node_id,
                partition_number,
                global_address.as_node_id(),
                partition_number,
            )?;
        }
        self.kernel_drop_node(&node_id)?;

        // Move other modules, and drop
        for (module_id, node_id) in modules {
            match module_id {
                ObjectModuleId::Main => panic!("Should have been removed already"),
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
                            (node_id, ObjectModuleId::Main),
                            (*global_address.as_node_id(), module_id),
                        );

                    // Move and drop
                    self.kernel_move_module(
                        &node_id,
                        OBJECT_BASE_PARTITION,
                        global_address.as_node_id(),
                        module_id.base_partition_num(),
                    )?;
                    self.kernel_drop_node(&node_id)?;
                }
            }
        }

        Ok(())
    }

    pub fn actor_get_receiver_node_id(&mut self) -> Option<(NodeId, bool)> {
        let actor = self.api.kernel_get_system_state().current;
        actor
            .try_as_method()
            .map(|a| (a.node_id, a.is_direct_access))
    }

    pub fn actor_get_fn_identifier(&mut self) -> Result<FnIdentifier, RuntimeError> {
        let actor = self.api.kernel_get_system_state().current;
        Ok(actor.fn_identifier())
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
            SystemLockData::Field(FieldLockData::Write {
                index,
                schema,
                schema_origin,
            }) => {
                self.validate_payload(&buffer, &schema, index, schema_origin)
                    .map_err(|e| {
                        RuntimeError::SystemError(SystemError::InvalidSubstateWrite(
                            e.error_message(&schema),
                        ))
                    })?;
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
        let actor = self.api.kernel_get_system_state().current;
        let package_address = actor.package_address().clone();
        let instance_context = actor.instance_context();
        let blueprint = BlueprintId::new(&package_address, blueprint_ident);

        self.new_object_internal(&blueprint, instance_context, schema, fields, kv_entries)
    }

    #[trace_resources]
    fn allocate_global_address(
        &mut self,
        blueprint_id: BlueprintId,
    ) -> Result<GlobalAddress, RuntimeError> {
        let allocated_node_id = self.api.kernel_allocate_node_id(
            IDAllocationRequest::Object {
                blueprint_id,
                global: true,
                virtual_node_id: None,
            }
            .entity_type(),
        )?;
        Ok(GlobalAddress::new_or_panic(allocated_node_id.0))
    }

    #[trace_resources]
    fn globalize(
        &mut self,
        modules: BTreeMap<ObjectModuleId, NodeId>,
    ) -> Result<GlobalAddress, RuntimeError> {
        // FIXME ensure that only the package actor can globalize its own blueprints

        let blueprint_id = self.resolve_blueprint_from_modules(&modules)?;
        let global_node_id = self.api.kernel_allocate_node_id(
            IDAllocationRequest::Object {
                blueprint_id,
                global: true,
                virtual_node_id: None,
            }
            .entity_type(),
        )?;
        let global_address = GlobalAddress::new_or_panic(global_node_id.into());

        self.globalize_with_address_internal(modules, global_address)?;

        Ok(global_address)
    }

    #[trace_resources]
    fn globalize_with_address(
        &mut self,
        modules: BTreeMap<ObjectModuleId, NodeId>,
        address: GlobalAddress,
    ) -> Result<(), RuntimeError> {
        // FIXME ensure that only the package actor can globalize its own blueprints

        self.globalize_with_address_internal(modules, address)
    }

    #[trace_resources]
    fn globalize_with_address_and_create_inner_object(
        &mut self,
        modules: BTreeMap<ObjectModuleId, NodeId>,
        address: GlobalAddress,
        inner_object_blueprint: &str,
        inner_object_fields: Vec<Vec<u8>>,
    ) -> Result<NodeId, RuntimeError> {
        let actor_blueprint = self.resolve_blueprint_from_modules(&modules)?;

        self.globalize_with_address_internal(modules, address)?;

        let blueprint = BlueprintId::new(&actor_blueprint.package_address, inner_object_blueprint);

        self.new_object_internal(
            &blueprint,
            Some(InstanceContext {
                outer_object: address,
                outer_blueprint: actor_blueprint.blueprint_name,
            }),
            None,
            inner_object_fields,
            btreemap!(),
        )
    }

    #[trace_resources]
    fn call_method_advanced(
        &mut self,
        receiver: &NodeId,
        direct_access: bool,
        object_module_id: ObjectModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let receiver_info = self.get_object_info(receiver)?;

        let (object_info, global_address) = match object_module_id {
            ObjectModuleId::Main => {
                let global_address = if receiver_info.global {
                    Some(GlobalAddress::new_or_panic(receiver.clone().into()))
                } else {
                    // See if we have a parent

                    // TODO: Cleanup, this is a rather crude way of trying to figure out
                    // TODO: whether the node reference is a child of the current parent
                    // TODO: this should be cleaned up once call_frame is refactored
                    let node_visibility = self.api.kernel_get_node_visibility(receiver);
                    // FIXME I believe this logic is incorrect/inconsistent with design, it's
                    // to duplicate previous logic.
                    if node_visibility.0.iter().any(|v| v.is_normal())
                        && !node_visibility
                            .0
                            .iter()
                            .any(|v| matches!(v, Visibility::FrameOwned))
                    {
                        match self.api.kernel_get_system_state().current {
                            Actor::Method(MethodActor { global_address, .. }) => {
                                global_address.clone()
                            }
                            _ => None,
                        }
                    } else {
                        None
                    }
                };

                (receiver_info.clone(), global_address)
            }
            // TODO: Check if type has these object modules
            ObjectModuleId::Metadata | ObjectModuleId::Royalty | ObjectModuleId::AccessRules => (
                ObjectInfo {
                    blueprint: object_module_id.static_blueprint().unwrap(),
                    outer_object: None,
                    global: receiver_info.global,
                    instance_schema: None,
                },
                None,
            ),
        };

        let identifier =
            MethodIdentifier(receiver.clone(), object_module_id, method_name.to_string());

        // TODO: Can we load this lazily when needed?
        let instance_context = if object_info.global {
            match global_address {
                None => None,
                Some(address) => Some(InstanceContext {
                    outer_object: address,
                    outer_blueprint: object_info.blueprint.blueprint_name.clone(),
                }),
            }
        } else {
            match &object_info.outer_object {
                None => None,
                Some(blueprint_parent) => {
                    // TODO: do this recursively until global?
                    let parent_info = self.get_object_info(blueprint_parent.as_node_id()).unwrap();
                    Some(InstanceContext {
                        outer_object: blueprint_parent.clone(),
                        outer_blueprint: parent_info.blueprint.blueprint_name.clone(),
                    })
                }
            }
        };

        let invocation = KernelInvocation {
            actor: Actor::method(
                global_address,
                identifier,
                receiver_info,
                object_info,
                instance_context,
                direct_access,
            ),
            args: IndexedScryptoValue::from_vec(args).map_err(|e| {
                RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
            })?,
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
            _ => return Err(RuntimeError::SystemError(SystemError::NotAnObject)),
        };

        Ok(object_info)
    }

    #[trace_resources]
    fn drop_object(&mut self, node_id: &NodeId) -> Result<Vec<Vec<u8>>, RuntimeError> {
        let info = self.get_object_info(node_id)?;
        let actor = self.api.kernel_get_system_state().current;
        let mut is_drop_allowed = false;

        // TODO: what's the right model, trading off between flexibility and security?

        // If the actor is the object's outer object
        if let Some(outer_object) = info.outer_object {
            if let Some(instance_context) = actor.instance_context() {
                if instance_context.outer_object.eq(&outer_object) {
                    is_drop_allowed = true;
                }
            }
        }
        // If the actor is a function within the same blueprint
        if let Actor::Function { blueprint, .. } = actor {
            if blueprint.eq(&info.blueprint) {
                is_drop_allowed = true;
            }
        }

        if !is_drop_allowed {
            return Err(RuntimeError::KernelError(
                KernelError::InvalidDropNodeAccess(Box::new(InvalidDropNodeAccess {
                    node_id: node_id.clone(),
                    package_address: info.blueprint.package_address,
                    blueprint_name: info.blueprint.blueprint_name,
                })),
            ));
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
                schema_origin,
                schema,
                index,
                can_own,
            }) => {
                self.validate_payload(&buffer, &schema, index, schema_origin)
                    .map_err(|e| {
                        RuntimeError::SystemError(SystemError::InvalidSubstateWrite(
                            e.error_message(&schema),
                        ))
                    })?;

                let substate = IndexedScryptoValue::from_slice(&buffer)
                    .expect("Should be valid due to payload check");

                if !can_own {
                    let own = substate.owned_nodes();
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
    fn key_value_store_new(&mut self, schema: KeyValueStoreSchema) -> Result<NodeId, RuntimeError> {
        schema
            .schema
            .validate()
            .map_err(|e| RuntimeError::SystemError(SystemError::InvalidKeyValueStoreSchema(e)))?;

        let node_id = self
            .api
            .kernel_allocate_node_id(IDAllocationRequest::KeyValueStore.entity_type())?;

        self.api.kernel_create_node(
            node_id,
            btreemap!(
                OBJECT_BASE_PARTITION => btreemap!(),
                TYPE_INFO_FIELD_PARTITION => ModuleInit::TypeInfo(
                    TypeInfoSubstate::KeyValueStore(KeyValueStoreInfo {
                        schema,
                    })
                ).to_substates(),
            ),
        )?;

        Ok(node_id)
    }

    #[trace_resources]
    fn key_value_store_get_info(
        &mut self,
        node_id: &NodeId,
    ) -> Result<KeyValueStoreSchema, RuntimeError> {
        let type_info = TypeInfoBlueprint::get_type(node_id, self.api)?;
        let info = match type_info {
            TypeInfoSubstate::KeyValueStore(info) => info,
            _ => return Err(RuntimeError::SystemError(SystemError::NotAKeyValueStore)),
        };

        Ok(info.schema)
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
            _ => return Err(RuntimeError::SystemError(SystemError::NotAKeyValueStore)),
        };

        self.validate_payload(
            key,
            &info.schema.schema,
            info.schema.key,
            SchemaOrigin::KeyValueStore {},
        )
        .map_err(|e| {
            RuntimeError::SystemError(SystemError::InvalidKeyValueKey(
                e.error_message(&info.schema.schema),
            ))
        })?;

        let lock_data = if flags.contains(LockFlags::MUTABLE) {
            SystemLockData::KeyValueEntry(KeyValueEntryLockData::Write {
                schema_origin: SchemaOrigin::KeyValueStore {},
                schema: info.schema.schema,
                index: info.schema.value,
                can_own: info.schema.can_own,
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
        collection_index: CollectionIndex,
        key: Vec<u8>,
        buffer: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, partition_num) = self.get_actor_index(actor_object_type, collection_index)?;

        let value = IndexedScryptoValue::from_vec(buffer).map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        if !value.owned_nodes().is_empty() {
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
        collection_index: CollectionIndex,
        key: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, partition_num) = self.get_actor_index(actor_object_type, collection_index)?;

        let rtn = self
            .api
            .kernel_remove_substate(&node_id, partition_num, &SubstateKey::Map(key))?
            .map(|v| v.into());

        Ok(rtn)
    }

    fn actor_index_scan(
        &mut self,
        object_handle: ObjectHandle,
        collection_index: CollectionIndex,
        count: u32,
    ) -> Result<Vec<Vec<u8>>, RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, partition_num) = self.get_actor_index(actor_object_type, collection_index)?;

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
        collection_index: CollectionIndex,
        count: u32,
    ) -> Result<Vec<Vec<u8>>, RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, partition_num) = self.get_actor_index(actor_object_type, collection_index)?;

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
        collection_index: CollectionIndex,
        sorted_key: SortedKey,
        buffer: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, partition_num) =
            self.get_actor_sorted_index(actor_object_type, collection_index)?;

        let value = IndexedScryptoValue::from_vec(buffer).map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        if !value.owned_nodes().is_empty() {
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
        collection_index: CollectionIndex,
        sorted_key: &SortedKey,
    ) -> Result<Option<Vec<u8>>, RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, partition_num) =
            self.get_actor_sorted_index(actor_object_type, collection_index)?;

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
        collection_index: CollectionIndex,
        count: u32,
    ) -> Result<Vec<Vec<u8>>, RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, partition_num) =
            self.get_actor_sorted_index(actor_object_type, collection_index)?;

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
            BlueprintId::new(&package_address, blueprint_name),
            function_name.to_string(),
        );

        let invocation = KernelInvocation {
            actor: Actor::function(identifier.0, identifier.1),
            args: IndexedScryptoValue::from_vec(args).map_err(|e| {
                RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
            })?,
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

    fn cost_unit_limit(&mut self) -> Result<u32, RuntimeError> {
        Ok(self
            .api
            .kernel_get_system()
            .modules
            .costing
            .fee_reserve
            .cost_unit_limit())
    }

    fn cost_unit_price(&mut self) -> Result<Decimal, RuntimeError> {
        Ok(self
            .api
            .kernel_get_system()
            .modules
            .costing
            .fee_reserve
            .cost_unit_price())
    }

    fn tip_percentage(&mut self) -> Result<u32, RuntimeError> {
        Ok(self
            .api
            .kernel_get_system()
            .modules
            .costing
            .fee_reserve
            .tip_percentage())
    }

    fn fee_balance(&mut self) -> Result<Decimal, RuntimeError> {
        Ok(self
            .api
            .kernel_get_system()
            .modules
            .costing
            .fee_reserve
            .fee_balance())
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
        object_handle: ObjectHandle,
        field_index: u8,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, partition_num, schema, type_index, object_info) =
            self.get_actor_field(actor_object_type, field_index)?;

        // TODO: Remove
        if flags.contains(LockFlags::UNMODIFIED_BASE) || flags.contains(LockFlags::FORCE_WRITE) {
            if !(object_info.blueprint.package_address.eq(&RESOURCE_PACKAGE)
                && object_info
                    .blueprint
                    .blueprint_name
                    .eq(FUNGIBLE_VAULT_BLUEPRINT))
            {
                return Err(RuntimeError::SystemError(SystemError::InvalidLockFlags));
            }
        }

        let lock_data = if flags.contains(LockFlags::MUTABLE) {
            FieldLockData::Write {
                schema_origin: SchemaOrigin::Blueprint(object_info.blueprint),
                schema,
                index: type_index,
            }
        } else {
            FieldLockData::Read
        };

        self.api.kernel_lock_substate(
            &node_id,
            partition_num,
            &SubstateKey::Tuple(field_index),
            flags,
            SystemLockData::Field(lock_data),
        )
    }

    #[trace_resources]
    fn actor_get_info(&mut self) -> Result<ObjectInfo, RuntimeError> {
        let actor = self.api.kernel_get_system_state().current;
        let object_info = actor
            .try_as_method()
            .map(|m| m.module_object_info.clone())
            .ok_or(RuntimeError::SystemError(SystemError::NotAMethod))?;

        Ok(object_info)
    }

    #[trace_resources]
    fn actor_get_node_id(&mut self) -> Result<NodeId, RuntimeError> {
        let actor = self.api.kernel_get_system_state().current;
        match actor {
            Actor::Method(MethodActor { node_id, .. }) => Ok(*node_id),
            _ => Err(RuntimeError::SystemError(SystemError::NodeIdNotExist)),
        }
    }
    #[trace_resources]
    fn actor_get_global_address(&mut self) -> Result<GlobalAddress, RuntimeError> {
        let actor = self.api.kernel_get_system_state().current;
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

    #[trace_resources]
    fn actor_get_blueprint(&mut self) -> Result<BlueprintId, RuntimeError> {
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        let actor = self.api.kernel_get_system_state().current;
        Ok(actor.blueprint().clone())
    }

    #[trace_resources]
    fn actor_call_module_method(
        &mut self,
        object_handle: ObjectHandle,
        module_id: ObjectModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;
        let node_id = match actor_object_type {
            ActorObjectType::SELF => {
                self.actor_get_receiver_node_id()
                    .ok_or(RuntimeError::SystemError(SystemError::NotAMethod))?
                    .0
            }
            ActorObjectType::OuterObject => self
                .actor_get_info()?
                .outer_object
                .ok_or(RuntimeError::SystemError(
                    SystemError::OuterObjectDoesNotExist,
                ))?
                .into_node_id(),
        };

        self.call_method_advanced(&node_id, false, module_id, method_name, args)
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
        collection_index: CollectionIndex,
        key: &[u8],
        flags: LockFlags,
    ) -> Result<KeyValueEntryHandle, RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, partition_num, schema, kv_schema, object_info) =
            self.get_actor_kv_partition(actor_object_type, collection_index)?;

        let lock_data = if flags.contains(LockFlags::MUTABLE) {
            let can_own = kv_schema.can_own;
            match kv_schema.value {
                TypeRef::Instance(index) => {
                    let mut instance_schema = object_info.instance_schema.unwrap();
                    KeyValueEntryLockData::Write {
                        schema_origin: SchemaOrigin::Instance {},
                        schema: instance_schema.schema,
                        index: instance_schema.type_index.remove(index as usize),
                        can_own,
                    }
                }
                TypeRef::Blueprint(index) => KeyValueEntryLockData::Write {
                    schema_origin: SchemaOrigin::Blueprint(object_info.blueprint),
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
        collection_index: CollectionIndex,
        key: &Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let handle = self.actor_lock_key_value_entry(
            object_handle,
            collection_index,
            key,
            LockFlags::MUTABLE,
        )?;
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

        let auth_zone_id = self
            .api
            .kernel_get_system()
            .modules
            .auth
            .last_auth_zone()
            .expect("Auth zone missing");

        Ok(auth_zone_id.into())
    }

    #[trace_resources]
    fn assert_access_rule(&mut self, rule: AccessRule) -> Result<(), RuntimeError> {
        self.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunSystem)?;

        // Fetch the tip auth zone
        let auth_zone_id = self
            .api
            .kernel_get_system()
            .modules
            .auth
            .last_auth_zone()
            .expect("Missing auth zone");

        // Authorize
        let auth_result = Authorization::check_authorization_against_access_rule(
            ActingLocation::InCallFrame,
            auth_zone_id,
            &rule,
            self,
        )?;
        match auth_result {
            AuthorizationCheckResult::Authorized => Ok(()),
            AuthorizationCheckResult::Failed(..) => Err(RuntimeError::SystemError(
                SystemError::AssertAccessRuleFailed,
            )),
        }
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

        let actor = self.api.kernel_get_system_state().current;

        // Locking the package info substate associated with the emitter's package
        let (blueprint_id, blueprint_schema, local_type_index) = {
            // Getting the package address and blueprint name associated with the actor
            let blueprint_id = match actor {
                Actor::Method(MethodActor {
                    module_object_info: ref object_info,
                    ..
                }) => Ok(object_info.blueprint.clone()),
                Actor::Function { ref blueprint, .. } => Ok(blueprint.clone()),
                _ => Err(RuntimeError::ApplicationError(
                    ApplicationError::EventError(Box::new(EventError::InvalidActor)),
                )),
            }?;

            let blueprint_schema = self.get_blueprint_schema(&blueprint_id)?;

            // Translating the event name to it's local_type_index which is stored in the blueprint
            // schema
            let local_type_index =
                if let Some(index) = blueprint_schema.event_schema.get(&event_name).cloned() {
                    index
                } else {
                    return Err(RuntimeError::ApplicationError(
                        ApplicationError::EventError(Box::new(EventError::SchemaNotFoundError {
                            blueprint: blueprint_id.clone(),
                            event_name,
                        })),
                    ));
                };

            (blueprint_id, blueprint_schema, local_type_index)
        };

        // Construct the event type identifier based on the current actor
        let actor = self.api.kernel_get_system_state().current;
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
                    ObjectModuleId::Main,
                    blueprint.blueprint_name.to_string(),
                ),
                local_type_index,
            )),
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::EventError(Box::new(EventError::InvalidActor)),
            )),
        }?;
        self.validate_payload(
            &event_data,
            &blueprint_schema.schema,
            event_type_identifier.1,
            SchemaOrigin::Blueprint(blueprint_id),
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

    fn kernel_allocate_node_id(&mut self, entity_type: EntityType) -> Result<NodeId, RuntimeError> {
        self.api.kernel_allocate_node_id(entity_type)
    }

    fn kernel_create_node(
        &mut self,
        node_id: NodeId,
        node_substates: NodeSubstates,
    ) -> Result<(), RuntimeError> {
        self.api.kernel_create_node(node_id, node_substates)
    }

    fn kernel_move_module(
        &mut self,
        src_node_id: &NodeId,
        src_partition_number: PartitionNumber,
        dest_node_id: &NodeId,
        dest_partition_number: PartitionNumber,
    ) -> Result<(), RuntimeError> {
        self.api.kernel_move_module(
            src_node_id,
            src_partition_number,
            dest_node_id,
            dest_partition_number,
        )
    }

    fn kernel_list_modules(
        &mut self,
        node_id: &NodeId,
    ) -> Result<BTreeSet<PartitionNumber>, RuntimeError> {
        self.api.kernel_list_modules(node_id)
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

    fn kernel_get_node_visibility(&self, node_id: &NodeId) -> NodeVisibility {
        self.api.kernel_get_node_visibility(node_id)
    }

    fn kernel_read_bucket(&mut self, bucket_id: &NodeId) -> Option<BucketSnapshot> {
        self.api.kernel_read_bucket(bucket_id)
    }

    fn kernel_read_proof(&mut self, proof_id: &NodeId) -> Option<ProofSnapshot> {
        self.api.kernel_read_proof(proof_id)
    }
}
