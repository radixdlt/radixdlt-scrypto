use super::id_allocation::IDAllocation;
use super::payload_validation::*;
use super::system_modules::auth::Authorization;
use super::system_modules::costing::CostingEntry;
use crate::errors::{
    ApplicationError, CannotGlobalizeError, CreateObjectError, InvalidDropNodeAccess,
    InvalidModuleType, PayloadValidationAgainstSchemaError, RuntimeError, SystemError,
    SystemModuleError,
};
use crate::errors::{EventError, SystemUpstreamError};
use crate::kernel::actor::{Actor, InstanceContext, MethodActor};
use crate::kernel::call_frame::{NodeVisibility, StableReferenceType, Visibility};
use crate::kernel::kernel_api::*;
use crate::system::node_init::type_info_partition;
use crate::system::node_modules::type_info::{TypeInfoBlueprint, TypeInfoSubstate};
use crate::system::system_callback::{
    FieldLockData, KeyValueEntryLockData, SystemConfig, SystemLockData,
};
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_modules::auth::{ActingLocation, AuthorizationCheckResult};
use crate::system::system_modules::execution_trace::{BucketSnapshot, ProofSnapshot};
use crate::track::interface::NodeSubstates;
use crate::types::*;
use radix_engine_interface::api::actor_index_api::ClientActorIndexApi;
use radix_engine_interface::api::actor_sorted_index_api::SortedKey;
use radix_engine_interface::api::field_api::{FieldHandle, LockFlags};
use radix_engine_interface::api::key_value_entry_api::{
    ClientKeyValueEntryApi, KeyValueEntryHandle,
};
use radix_engine_interface::api::key_value_store_api::ClientKeyValueStoreApi;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::schema::{
    BlueprintKeyValueStoreSchema, Condition, InstanceSchema, KeyValueStoreSchema,
};
use resources_tracker_macro::trace_resources;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum SubstateMutability {
    Mutable,
    Immutable,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct DynSubstate<E> {
    pub value: E,
    pub mutability: SubstateMutability,
}

impl<E> DynSubstate<E> {
    pub fn lock(&mut self) {
        self.mutability = SubstateMutability::Immutable;
    }

    pub fn is_mutable(&self) -> bool {
        matches!(self.mutability, SubstateMutability::Mutable)
    }
}

pub type FieldSubstate<V> = DynSubstate<(V,)>;

impl<V> FieldSubstate<V> {
    pub fn new_field(value: V) -> Self {
        Self {
            value: (value,),
            mutability: SubstateMutability::Mutable,
        }
    }
}

pub type KeyValueEntrySubstate<V> = DynSubstate<Option<V>>;

impl<V> KeyValueEntrySubstate<V> {
    pub fn entry(value: V) -> Self {
        Self {
            value: Some(value),
            mutability: SubstateMutability::Mutable,
        }
    }

    pub fn locked_entry(value: V) -> Self {
        Self {
            value: Some(value),
            mutability: SubstateMutability::Immutable,
        }
    }

    pub fn locked_empty_entry() -> Self {
        Self {
            value: None,
            mutability: SubstateMutability::Immutable,
        }
    }

    pub fn remove(&mut self) -> Option<V> {
        self.value.take()
    }
}

impl<V> Default for KeyValueEntrySubstate<V> {
    fn default() -> Self {
        Self {
            value: Option::None,
            mutability: SubstateMutability::Mutable,
        }
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum KeyValueStoreSchemaIdent {
    Key,
    Value,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum FunctionSchemaIdent {
    Input,
    Output,
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

    fn validate_payload<'s>(
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

    pub fn validate_payload_at_type_pointer(
        &mut self,
        blueprint_id: &BlueprintId,
        instance_schema: &Option<InstanceSchema>,
        type_pointer: TypePointer,
        payload: &[u8],
    ) -> Result<(), RuntimeError> {
        match type_pointer {
            TypePointer::Package(hash, index) => {
                let schema = self.get_schema(blueprint_id.package_address, &hash)?;

                self.validate_payload(
                    payload,
                    &schema,
                    index,
                    SchemaOrigin::Blueprint(blueprint_id.clone()),
                )
                .map_err(|err| {
                    RuntimeError::SystemError(SystemError::PayloadValidationAgainstSchemaError(
                        PayloadValidationAgainstSchemaError::PayloadValidationError(
                            err.error_message(&schema),
                        ),
                    ))
                })?;
            }
            TypePointer::Instance(instance_index) => {
                let instance_schema = match instance_schema.as_ref() {
                    Some(instance_schema) => instance_schema,
                    None => {
                        return Err(RuntimeError::SystemError(
                            SystemError::PayloadValidationAgainstSchemaError(
                                PayloadValidationAgainstSchemaError::InstanceSchemaDoesNotExist,
                            ),
                        ));
                    }
                };
                let index = instance_schema
                    .type_index
                    .get(instance_index as usize)
                    .unwrap()
                    .clone();

                self.validate_payload(
                    payload,
                    &instance_schema.schema,
                    index,
                    SchemaOrigin::Instance,
                )
                .map_err(|err| {
                    RuntimeError::SystemError(SystemError::PayloadValidationAgainstSchemaError(
                        PayloadValidationAgainstSchemaError::PayloadValidationError(
                            err.error_message(&instance_schema.schema),
                        ),
                    ))
                })?;
            }
        }

        Ok(())
    }

    pub fn validate_payload_against_blueprint_schema<'s>(
        &'s mut self,
        blueprint_id: &BlueprintId,
        instance_schema: &'s Option<InstanceSchema>,
        payloads: &[(&Vec<u8>, TypePointer)],
    ) -> Result<(), RuntimeError> {
        for (payload, type_pointer) in payloads {
            self.validate_payload_at_type_pointer(
                blueprint_id,
                instance_schema,
                type_pointer.clone(),
                payload,
            )?;
        }

        Ok(())
    }

    fn validate_instance_schema_and_state(
        &mut self,
        blueprint_id: &BlueprintId,
        blueprint_interface: &BlueprintInterface,
        blueprint_features: &BTreeSet<String>,
        outer_blueprint_features: &BTreeSet<String>,
        instance_schema: &Option<InstanceSchema>,
        fields: Vec<FieldValue>,
        kv_entries: BTreeMap<u8, BTreeMap<Vec<u8>, KVEntry>>,
    ) -> Result<BTreeMap<PartitionOffset, BTreeMap<SubstateKey, IndexedScryptoValue>>, RuntimeError>
    {
        // Validate instance schema
        {
            if let Some(instance_schema) = instance_schema {
                validate_schema(&instance_schema.schema)
                    .map_err(|_| RuntimeError::SystemError(SystemError::InvalidInstanceSchema))?;
            }
            if !blueprint_interface
                .state
                .validate_instance_schema(instance_schema)
            {
                return Err(RuntimeError::SystemError(
                    SystemError::InvalidInstanceSchema,
                ));
            }
        }

        let mut partitions = BTreeMap::new();

        // Fields
        {
            let expected_num_fields = blueprint_interface.state.num_fields();
            if expected_num_fields != fields.len() {
                return Err(RuntimeError::SystemError(SystemError::CreateObjectError(
                    Box::new(CreateObjectError::WrongNumberOfSubstates(
                        blueprint_id.clone(),
                        fields.len(),
                        expected_num_fields,
                    )),
                )));
            }

            if let Some((offset, field_schemas)) = &blueprint_interface.state.fields {
                let mut partition = BTreeMap::new();

                let mut fields_to_check = Vec::new();

                for (i, field) in fields.iter().enumerate() {
                    // Check for any feature conditions
                    match &field_schemas[i].condition {
                        Condition::IfFeature(feature) => {
                            if !blueprint_features.contains(feature) {
                                continue;
                            }
                        }
                        Condition::IfOuterFeature(feature) => {
                            if !outer_blueprint_features.contains(feature) {
                                continue;
                            }
                        }
                        Condition::Always => {}
                    }

                    let pointer = blueprint_interface
                        .get_field_type_pointer(i as u8)
                        .ok_or_else(|| {
                            RuntimeError::SystemError(
                                SystemError::PayloadValidationAgainstSchemaError(
                                    PayloadValidationAgainstSchemaError::FieldDoesNotExist(i as u8),
                                ),
                            )
                        })?;

                    fields_to_check.push((&field.value, pointer));
                }

                self.validate_payload_against_blueprint_schema(
                    &blueprint_id,
                    instance_schema,
                    &fields_to_check,
                )?;

                for (i, field) in fields.into_iter().enumerate() {
                    // Check for any feature conditions
                    match &field_schemas[i].condition {
                        Condition::IfFeature(feature) => {
                            if !blueprint_features.contains(feature) {
                                continue;
                            }
                        }
                        Condition::IfOuterFeature(feature) => {
                            if !outer_blueprint_features.contains(feature) {
                                continue;
                            }
                        }
                        Condition::Always => {}
                    }

                    let value: ScryptoValue =
                        scrypto_decode(&field.value).expect("Checked by payload-schema validation");

                    let substate = FieldSubstate {
                        value: (value,),
                        mutability: if field.locked {
                            SubstateMutability::Immutable
                        } else {
                            SubstateMutability::Mutable
                        },
                    };

                    partition.insert(
                        SubstateKey::Field(i as u8),
                        IndexedScryptoValue::from_typed(&substate),
                    );
                }

                partitions.insert(offset.clone(), partition);
            }
        }

        // Collections
        {
            for (collection_index, entries) in kv_entries {
                let mut partition = BTreeMap::new();

                for (key, kv_entry) in entries {
                    let (kv_entry, value_can_own) = if let Some(value) = kv_entry.value {
                        let key_type_pointer = blueprint_interface
                            .get_kv_key_type_pointer(collection_index)
                            .ok_or_else(|| {
                                RuntimeError::SystemError(
                                    SystemError::PayloadValidationAgainstSchemaError(
                                        PayloadValidationAgainstSchemaError::KeyValueStoreKeyDoesNotExist
                                    ),
                                )
                            })?;

                        let (value_type_pointer, value_can_own) = blueprint_interface
                            .get_kv_value_type_pointer(collection_index)
                            .ok_or_else(|| {
                                RuntimeError::SystemError(
                                    SystemError::PayloadValidationAgainstSchemaError(
                                        PayloadValidationAgainstSchemaError::KeyValueStoreValueDoesNotExist
                                    ),
                                )
                            })?;

                        self.validate_payload_against_blueprint_schema(
                            &blueprint_id,
                            instance_schema,
                            &[(&key, key_type_pointer), (&value, value_type_pointer)],
                        )?;

                        let value: ScryptoValue = scrypto_decode(&value).unwrap();
                        let kv_entry = if kv_entry.locked {
                            KeyValueEntrySubstate::locked_entry(value)
                        } else {
                            KeyValueEntrySubstate::entry(value)
                        };
                        (kv_entry, value_can_own)
                    } else {
                        if kv_entry.locked {
                            (KeyValueEntrySubstate::locked_empty_entry(), true)
                        } else {
                            continue;
                        }
                    };

                    let value = IndexedScryptoValue::from_typed(&kv_entry);
                    if !value_can_own {
                        if !value.owned_nodes().is_empty() {
                            return Err(RuntimeError::SystemError(
                                SystemError::InvalidKeyValueStoreOwnership,
                            ));
                        }
                    }

                    partition.insert(SubstateKey::Map(key), value);
                }

                let partition_offset = blueprint_interface
                    .state
                    .collections
                    .get(collection_index as usize)
                    .ok_or_else(|| {
                        RuntimeError::SystemError(SystemError::PayloadValidationAgainstSchemaError(
                            PayloadValidationAgainstSchemaError::CollectionDoesNotExist,
                        ))
                    })?
                    .0;

                partitions.insert(partition_offset, partition);
            }

            for (offset, _blueprint_partition_schema) in
                blueprint_interface.state.collections.iter()
            {
                if !partitions.contains_key(offset) {
                    partitions.insert(offset.clone(), BTreeMap::new());
                }
            }
        }

        Ok(partitions)
    }

    pub fn get_schema(
        &mut self,
        package_address: PackageAddress,
        schema_hash: &Hash,
    ) -> Result<ScryptoSchema, RuntimeError> {
        let def = self
            .api
            .kernel_get_system_state()
            .system
            .schema_cache
            .get(schema_hash);
        if let Some(schema) = def {
            return Ok(schema.clone());
        }

        let handle = self.api.kernel_open_substate_with_default(
            package_address.as_node_id(),
            MAIN_BASE_PARTITION
                .at_offset(PACKAGE_SCHEMAS_PARTITION_OFFSET)
                .unwrap(),
            &SubstateKey::Map(scrypto_encode(schema_hash).unwrap()),
            LockFlags::read_only(),
            Some(|| {
                let kv_entry = KeyValueEntrySubstate::<()>::default();
                IndexedScryptoValue::from_typed(&kv_entry)
            }),
            SystemLockData::default(),
        )?;

        let substate: KeyValueEntrySubstate<ScryptoSchema> =
            self.api.kernel_read_substate(handle)?.as_typed().unwrap();
        self.api.kernel_close_substate(handle)?;

        let schema = substate.value.unwrap();

        self.api
            .kernel_get_system_state()
            .system
            .schema_cache
            .insert(schema_hash.clone(), schema.clone());

        Ok(schema)
    }

    pub fn get_blueprint_default_interface(
        &mut self,
        blueprint_id: BlueprintId,
    ) -> Result<BlueprintInterface, RuntimeError> {
        let bp_version_key = BlueprintVersionKey::new_default(blueprint_id.blueprint_name);
        Ok(self
            .get_blueprint_definition(blueprint_id.package_address, &bp_version_key)?
            .interface)
    }

    pub fn get_blueprint_definition(
        &mut self,
        package_address: PackageAddress,
        bp_version_key: &BlueprintVersionKey,
    ) -> Result<BlueprintDefinition, RuntimeError> {
        let canonical_bp_id = CanonicalBlueprintId {
            address: package_address,
            blueprint: bp_version_key.blueprint.to_string(),
            version: bp_version_key.version.clone(),
        };

        let def = self
            .api
            .kernel_get_system_state()
            .system
            .blueprint_cache
            .get(&canonical_bp_id);
        if let Some(definition) = def {
            return Ok(definition.clone());
        }

        let handle = self.api.kernel_open_substate_with_default(
            package_address.as_node_id(),
            MAIN_BASE_PARTITION
                .at_offset(PACKAGE_BLUEPRINTS_PARTITION_OFFSET)
                .unwrap(),
            &SubstateKey::Map(scrypto_encode(bp_version_key).unwrap()),
            LockFlags::read_only(),
            Some(|| {
                let kv_entry = KeyValueEntrySubstate::<()>::default();
                IndexedScryptoValue::from_typed(&kv_entry)
            }),
            SystemLockData::default(),
        )?;

        let substate: KeyValueEntrySubstate<BlueprintDefinition> =
            self.api.kernel_read_substate(handle)?.as_typed().unwrap();
        self.api.kernel_close_substate(handle)?;

        let definition = match substate.value {
            Some(definition) => definition,
            None => {
                return Err(RuntimeError::SystemError(
                    SystemError::BlueprintDoesNotExist(canonical_bp_id),
                ))
            }
        };

        self.api
            .kernel_get_system_state()
            .system
            .blueprint_cache
            .insert(canonical_bp_id, definition.clone());

        Ok(definition)
    }

    pub fn prepare_global_address(
        &mut self,
        blueprint_id: BlueprintId,
        global_address: GlobalAddress,
    ) -> Result<GlobalAddressReservation, RuntimeError> {
        // Create global address phantom

        self.api.kernel_create_node(
            global_address.as_node_id().clone(),
            btreemap!(
                TYPE_INFO_FIELD_PARTITION => type_info_partition(
                    TypeInfoSubstate::GlobalAddressPhantom(GlobalAddressPhantom {
                        blueprint_id,
                    })
                )
            ),
        )?;

        // Create global address reservation
        let global_address_reservation = self
            .api
            .kernel_allocate_node_id(EntityType::InternalGenericComponent)?;
        self.api.kernel_create_node(
            global_address_reservation,
            btreemap!(
                TYPE_INFO_FIELD_PARTITION => type_info_partition(
                    TypeInfoSubstate::GlobalAddressReservation(global_address.clone())
                )
            ),
        )?;

        Ok(GlobalAddressReservation(Own(global_address_reservation)))
    }

    pub fn get_node_type_info(&mut self, node_id: &NodeId) -> Option<TypeInfoSubstate> {
        self.api
            .kernel_open_substate(
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
                            .kernel_close_substate(lock_handle)
                            .and_then(|_| Ok(substate))
                    })
            })
            .ok()
    }

    fn new_object_internal(
        &mut self,
        blueprint_id: &BlueprintId,
        features: Vec<&str>,
        instance_context: Option<InstanceContext>,
        instance_schema: Option<InstanceSchema>,
        fields: Vec<FieldValue>,
        kv_entries: BTreeMap<u8, BTreeMap<Vec<u8>, KVEntry>>,
    ) -> Result<NodeId, RuntimeError> {
        let blueprint_interface = self.get_blueprint_default_interface(blueprint_id.clone())?;
        let expected_outer_blueprint = blueprint_interface.blueprint_type.clone();

        let (blueprint_type, object_features, outer_object_features) =
            if let BlueprintType::Inner { outer_blueprint } = &expected_outer_blueprint {
                match instance_context {
                    Some(context) if context.outer_blueprint.eq(outer_blueprint) => {
                        let outer_object_info =
                            self.get_node_object_info(context.outer_object.as_node_id())?;

                        (
                            BlueprintObjectType::Inner {
                                outer_object: context.outer_object,
                            },
                            BTreeSet::new(),
                            // FIXME
                            outer_object_info.get_main_features(),
                        )
                    }
                    _ => {
                        return Err(RuntimeError::SystemError(
                            SystemError::InvalidChildObjectCreation,
                        ));
                    }
                }
            } else {
                let features: BTreeSet<String> =
                    features.into_iter().map(|s| s.to_string()).collect();

                // Validate features
                for feature in &features {
                    if !blueprint_interface.feature_set.contains(feature) {
                        return Err(RuntimeError::SystemError(SystemError::InvalidFeature(
                            feature.to_string(),
                        )));
                    }
                }

                (BlueprintObjectType::Outer, features, BTreeSet::new())
            };

        let user_substates = self.validate_instance_schema_and_state(
            blueprint_id,
            &blueprint_interface,
            &object_features,
            &outer_object_features,
            &instance_schema,
            fields,
            kv_entries,
        )?;

        let node_id = self.api.kernel_allocate_node_id(
            IDAllocation::Object {
                blueprint_id: blueprint_id.clone(),
                global: false,
            }
            .entity_type(),
        )?;

        let mut node_substates = btreemap!(
            TYPE_INFO_FIELD_PARTITION => type_info_partition(
                TypeInfoSubstate::Object(NodeObjectInfo {
                    global:false,
                    module_versions: btreemap!(
                        ObjectModuleId::Main => BlueprintVersion::default(),
                    ),

                    main_blueprint_info: BlueprintObjectInfo {
                        blueprint_id: blueprint_id.clone(),
                        blueprint_type,
                        features: object_features,
                        instance_schema,
                    }
                })
            ),
        );

        for (offset, partition) in user_substates.into_iter() {
            let partition_num = MAIN_BASE_PARTITION
                .at_offset(offset)
                .expect("Module number overflow");
            node_substates.insert(partition_num, partition);
        }

        self.api.kernel_create_node(node_id, node_substates)?;

        Ok(node_id.into())
    }

    fn key_value_entry_remove_and_close_substate(
        &mut self,
        handle: KeyValueEntryHandle,
    ) -> Result<Vec<u8>, RuntimeError> {
        // TODO: Replace with api::replace
        let current_value = self
            .api
            .kernel_read_substate(handle)
            .map(|v| v.as_slice().to_vec())?;

        let mut kv_entry: KeyValueEntrySubstate<ScryptoValue> =
            scrypto_decode(&current_value).unwrap();
        let value = kv_entry.remove();
        self.kernel_write_substate(handle, IndexedScryptoValue::from_typed(&kv_entry))?;

        self.kernel_close_substate(handle)?;

        let current_value = scrypto_encode(&value).unwrap();

        Ok(current_value)
    }

    pub fn get_blueprint_object_info(
        &mut self,
        node_id: &NodeId,
        module_id: ObjectModuleId,
    ) -> Result<BlueprintObjectInfo, RuntimeError> {
        let info = match module_id {
            ObjectModuleId::Main => self.get_node_object_info(node_id)?.main_blueprint_info,
            _ => BlueprintObjectInfo {
                blueprint_id: module_id.static_blueprint().unwrap(),
                blueprint_type: BlueprintObjectType::Outer,
                features: BTreeSet::default(),
                instance_schema: None,
            },
        };

        Ok(info)
    }

    fn get_actor_object_id(
        &mut self,
        actor_object_type: ActorObjectType,
    ) -> Result<(NodeId, ObjectModuleId), RuntimeError> {
        let actor = self.api.kernel_get_system_state().current;
        let method_actor = actor
            .try_as_method()
            .ok_or_else(|| RuntimeError::SystemError(SystemError::NotAMethod))?;

        let object_id = match actor_object_type {
            ActorObjectType::OuterObject => {
                let node_id = method_actor.node_id;
                let module_id = method_actor.module_id;
                let object_info = self.get_node_object_info(&node_id)?;
                let address = object_info.try_get_outer_object(module_id).ok_or_else(|| {
                    RuntimeError::SystemError(SystemError::OuterObjectDoesNotExist)
                })?;

                // TODO: This is currently assuming all outer objects are under the main module
                (address.into_node_id(), ObjectModuleId::Main)
            }
            ActorObjectType::SELF => {
                let node_id = method_actor.node_id;
                let object_module_id = method_actor.module_id;
                (node_id, object_module_id)
            }
        };

        Ok(object_id)
    }

    fn get_actor_schema(
        &mut self,
        actor_object_type: ActorObjectType,
    ) -> Result<
        (
            NodeId,
            PartitionNumber,
            BlueprintInterface,
            Option<InstanceSchema>,
            BlueprintId,
        ),
        RuntimeError,
    > {
        let (node_id, module_id) = self.get_actor_object_id(actor_object_type)?;
        let blueprint_info = self.get_blueprint_object_info(&node_id, module_id)?;
        let blueprint_interface =
            self.get_blueprint_default_interface(blueprint_info.blueprint_id.clone())?;

        Ok((
            node_id,
            module_id.base_partition_num(),
            blueprint_interface,
            blueprint_info.instance_schema,
            blueprint_info.blueprint_id,
        ))
    }

    fn get_actor_field(
        &mut self,
        actor_object_type: ActorObjectType,
        field_index: u8,
    ) -> Result<(NodeId, PartitionNumber, TypePointer, BlueprintId), RuntimeError> {
        let (node_id, base_partition, interface, _instance_schema, blueprint_id) =
            self.get_actor_schema(actor_object_type)?;

        let (partition_offset, field_schema) =
            interface.state.field(field_index).ok_or_else(|| {
                RuntimeError::SystemError(SystemError::FieldDoesNotExist(
                    blueprint_id.clone(),
                    field_index,
                ))
            })?;

        match field_schema.condition {
            Condition::IfFeature(feature) => {
                if !self.is_feature_enabled(&node_id, feature.as_str())? {
                    return Err(RuntimeError::SystemError(SystemError::FieldDoesNotExist(
                        blueprint_id.clone(),
                        field_index,
                    )));
                }
            }
            Condition::IfOuterFeature(feature) => {
                // FIXME: This currently assumes only main modules can have features
                let object_info = self.get_node_object_info(&node_id)?;

                if !self.is_feature_enabled(
                    object_info.get_main_outer_object().as_node_id(),
                    feature.as_str(),
                )? {
                    return Err(RuntimeError::SystemError(SystemError::FieldDoesNotExist(
                        blueprint_id.clone(),
                        field_index,
                    )));
                }
            }
            Condition::Always => {}
        }

        let pointer = field_schema.field;

        let partition_num = base_partition
            .at_offset(partition_offset)
            .expect("Module number overflow");

        Ok((node_id, partition_num, pointer, blueprint_id))
    }

    fn get_actor_kv_partition(
        &mut self,
        actor_object_type: ActorObjectType,
        collection_index: CollectionIndex,
    ) -> Result<
        (
            NodeId,
            PartitionNumber,
            BlueprintKeyValueStoreSchema<TypePointer>,
            Option<InstanceSchema>,
            BlueprintId,
        ),
        RuntimeError,
    > {
        let (node_id, base_partition, interface, instance_schema, blueprint_id) =
            self.get_actor_schema(actor_object_type)?;

        let (partition_offset, kv_schema) = interface
            .state
            .key_value_store_partition(collection_index)
            .ok_or_else(|| {
                RuntimeError::SystemError(SystemError::KeyValueStoreDoesNotExist(
                    blueprint_id.clone(),
                    collection_index,
                ))
            })?;

        let partition_num = base_partition
            .at_offset(partition_offset)
            .expect("Module number overflow");

        Ok((
            node_id,
            partition_num,
            kv_schema,
            instance_schema,
            blueprint_id,
        ))
    }

    fn get_actor_index(
        &mut self,
        actor_object_type: ActorObjectType,
        collection_index: CollectionIndex,
    ) -> Result<(NodeId, PartitionNumber), RuntimeError> {
        let (node_id, base_partition, interface, _instance_schema, blueprint_id) =
            self.get_actor_schema(actor_object_type)?;

        let (partition_offset, _) = interface
            .state
            .index_partition(collection_index)
            .ok_or_else(|| {
                RuntimeError::SystemError(SystemError::IndexDoesNotExist(
                    blueprint_id,
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
        let (node_id, base_partition, interface, _instance_schema, blueprint_id) =
            self.get_actor_schema(actor_object_type)?;

        let (partition_offset, _) = interface
            .state
            .sorted_index_partition(collection_index)
            .ok_or_else(|| {
                RuntimeError::SystemError(SystemError::SortedIndexDoesNotExist(
                    blueprint_id,
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

        Ok(self
            .get_node_object_info(node_id)?
            .main_blueprint_info
            .blueprint_id)
    }

    /// ASSUMPTIONS:
    /// Assumes the caller has already checked that the entity type on the GlobalAddress is valid
    /// against the given self module.
    fn globalize_with_address_internal(
        &mut self,
        mut modules: BTreeMap<ObjectModuleId, NodeId>,
        global_address_reservation: GlobalAddressReservation,
    ) -> Result<GlobalAddress, RuntimeError> {
        // Check global address reservation
        let global_address = {
            let substates = self.kernel_drop_node(global_address_reservation.0.as_node_id())?;

            let type_info: Option<TypeInfoSubstate> = substates
                .get(&TYPE_INFO_FIELD_PARTITION)
                .and_then(|x| x.get(&TypeInfoField::TypeInfo.into()))
                .and_then(|x| x.as_typed().ok());

            match type_info {
                Some(TypeInfoSubstate::GlobalAddressReservation(x)) => x,
                _ => {
                    return Err(RuntimeError::SystemError(
                        SystemError::InvalidGlobalAddressReservation,
                    ));
                }
            }
        };

        // Check blueprint id
        let reserved_blueprint_id = {
            let lock_handle = self.kernel_open_substate(
                global_address.as_node_id(),
                TYPE_INFO_FIELD_PARTITION,
                &TypeInfoField::TypeInfo.into(),
                LockFlags::MUTABLE, // This is to ensure the substate is lock free!
                SystemLockData::Default,
            )?;
            let type_info: TypeInfoSubstate =
                self.kernel_read_substate(lock_handle)?.as_typed().unwrap();
            self.kernel_close_substate(lock_handle)?;
            match type_info {
                TypeInfoSubstate::GlobalAddressPhantom(GlobalAddressPhantom { blueprint_id }) => {
                    blueprint_id
                }
                _ => unreachable!(),
            }
        };

        // Check for required modules
        if !modules.contains_key(&ObjectModuleId::AccessRules) {
            return Err(RuntimeError::SystemError(SystemError::MissingModule(
                ObjectModuleId::AccessRules,
            )));
        }
        if !modules.contains_key(&ObjectModuleId::Metadata) {
            return Err(RuntimeError::SystemError(SystemError::MissingModule(
                ObjectModuleId::Metadata,
            )));
        }

        let node_id = modules
            .remove(&ObjectModuleId::Main)
            .ok_or(RuntimeError::SystemError(SystemError::MissingModule(
                ObjectModuleId::Main,
            )))?;
        self.api
            .kernel_get_system_state()
            .system
            .modules
            .add_replacement(
                (node_id, ObjectModuleId::Main),
                (*global_address.as_node_id(), ObjectModuleId::Main),
            );

        // Read the type info
        let mut node_object_info = self.get_node_object_info(&node_id)?;

        // Verify can globalize with address
        {
            if node_object_info.global {
                return Err(RuntimeError::SystemError(SystemError::CannotGlobalize(
                    CannotGlobalizeError::AlreadyGlobalized,
                )));
            }
            if !node_object_info
                .main_blueprint_info
                .blueprint_id
                .eq(&reserved_blueprint_id)
            {
                return Err(RuntimeError::SystemError(SystemError::CannotGlobalize(
                    CannotGlobalizeError::InvalidBlueprintId,
                )));
            }
        }

        // Update Object Info
        {
            node_object_info.global = true;
            for module_id in modules.keys() {
                node_object_info
                    .module_versions
                    .insert(module_id.clone(), BlueprintVersion::default());
            }
        }

        let num_main_partitions = {
            let interface = self.get_blueprint_default_interface(
                node_object_info.main_blueprint_info.blueprint_id.clone(),
            )?;
            interface.state.num_partitions()
        };

        // Create a global node
        self.kernel_create_node(
            global_address.into(),
            btreemap!(
                TYPE_INFO_FIELD_PARTITION => type_info_partition(TypeInfoSubstate::Object(node_object_info))
            ),
        )?;

        // Move self modules to the newly created global node, and drop
        for offset in 0u8..num_main_partitions {
            let partition_number = MAIN_BASE_PARTITION
                .at_offset(PartitionOffset(offset))
                .unwrap();
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
                    let blueprint_id = self
                        .get_node_object_info(&node_id)?
                        .main_blueprint_info
                        .blueprint_id;
                    let expected_blueprint = module_id.static_blueprint().unwrap();
                    if !blueprint_id.eq(&expected_blueprint) {
                        return Err(RuntimeError::SystemError(SystemError::InvalidModuleType(
                            Box::new(InvalidModuleType {
                                expected_blueprint,
                                actual_blueprint: blueprint_id,
                            }),
                        )));
                    }

                    self.api
                        .kernel_get_system_state()
                        .system
                        .modules
                        .add_replacement(
                            (node_id, ObjectModuleId::Main),
                            (*global_address.as_node_id(), module_id),
                        );

                    // Move and drop
                    let interface = self.get_blueprint_default_interface(blueprint_id.clone())?;
                    let num_partitions = interface.state.num_partitions();

                    let module_base_partition = module_id.base_partition_num();
                    for offset in 0u8..num_partitions {
                        let src = MAIN_BASE_PARTITION
                            .at_offset(PartitionOffset(offset))
                            .unwrap();
                        let dest = module_base_partition
                            .at_offset(PartitionOffset(offset))
                            .unwrap();

                        self.kernel_move_module(&node_id, src, global_address.as_node_id(), dest)?;
                    }

                    self.kernel_drop_node(&node_id)?;
                }
            }
        }

        Ok(global_address)
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

    pub fn instance_context(&mut self) -> Option<InstanceContext> {
        let actor = self.api.kernel_get_system_state().current;

        let method_actor = match actor {
            Actor::Method(method_actor) => method_actor,
            _ => return None,
        };

        let instance_context = if method_actor.node_object_info.global {
            Some(InstanceContext {
                outer_object: GlobalAddress::new_or_panic(method_actor.node_id.0),
                outer_blueprint: method_actor.get_blueprint_id().blueprint_name,
            })
        } else {
            match method_actor.get_blueprint_info() {
                BlueprintObjectType::Inner { outer_object } => {
                    // TODO: do this recursively until global?
                    let outer_info = self
                        .get_node_object_info(outer_object.as_node_id())
                        .unwrap();
                    Some(InstanceContext {
                        outer_object,
                        outer_blueprint: outer_info.main_blueprint_info.blueprint_id.blueprint_name,
                    })
                }
                BlueprintObjectType::Outer { .. } => None,
            }
        };

        instance_context
    }

    pub fn is_feature_enabled(
        &mut self,
        node_id: &NodeId,
        feature: &str,
    ) -> Result<bool, RuntimeError> {
        let object_info = self.get_node_object_info(node_id)?;
        let enabled = object_info.main_blueprint_info.features.contains(feature);

        Ok(enabled)
    }
}

impl<'a, Y, V> ClientFieldApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    // Costing through kernel
    #[trace_resources]
    fn field_read(&mut self, handle: FieldHandle) -> Result<Vec<u8>, RuntimeError> {
        let LockInfo { data, .. } = self.api.kernel_get_lock_info(handle)?;
        match data {
            SystemLockData::Field(..) => {}
            _ => {
                return Err(RuntimeError::SystemError(SystemError::NotAFieldHandle));
            }
        }

        self.api.kernel_read_substate(handle).map(|v| {
            let wrapper: FieldSubstate<ScryptoValue> = v.as_typed().unwrap();
            scrypto_encode(&wrapper.value.0).unwrap()
        })
    }

    // Costing through kernel
    #[trace_resources]
    fn field_write(&mut self, handle: FieldHandle, buffer: Vec<u8>) -> Result<(), RuntimeError> {
        let LockInfo { data, .. } = self.api.kernel_get_lock_info(handle)?;

        match data {
            SystemLockData::Field(FieldLockData::Write {
                blueprint_id,
                type_pointer,
            }) => {
                self.validate_payload_at_type_pointer(
                    &blueprint_id,
                    &None, // TODO: Change to Some, once support for generic fields is implemented
                    type_pointer,
                    &buffer,
                )?;
            }
            _ => {
                return Err(RuntimeError::SystemError(SystemError::NotAFieldWriteHandle));
            }
        }

        let value: ScryptoValue =
            scrypto_decode(&buffer).expect("Should be valid due to payload check");

        let substate = IndexedScryptoValue::from_typed(&FieldSubstate::new_field(value));
        self.api.kernel_write_substate(handle, substate)?;

        Ok(())
    }

    // Costing through kernel
    #[trace_resources]
    fn field_lock(&mut self, handle: FieldHandle) -> Result<(), RuntimeError> {
        let LockInfo { data, .. } = self.api.kernel_get_lock_info(handle)?;

        match data {
            SystemLockData::Field(FieldLockData::Write { .. }) => {}
            _ => {
                return Err(RuntimeError::SystemError(SystemError::NotAFieldWriteHandle));
            }
        }

        let v = self.api.kernel_read_substate(handle)?;
        let mut substate: FieldSubstate<ScryptoValue> = v.as_typed().unwrap();
        substate.lock();
        let indexed = IndexedScryptoValue::from_typed(&substate);
        self.api.kernel_write_substate(handle, indexed)?;

        Ok(())
    }

    // Costing through kernel
    #[trace_resources]
    fn field_close(&mut self, handle: FieldHandle) -> Result<(), RuntimeError> {
        let LockInfo { data, .. } = self.api.kernel_get_lock_info(handle)?;
        match data {
            SystemLockData::Field(..) => {}
            _ => {
                return Err(RuntimeError::SystemError(SystemError::NotAFieldHandle));
            }
        }

        self.api.kernel_close_substate(handle)
    }
}

impl<'a, Y, V> ClientObjectApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    // Costing through kernel
    #[trace_resources]
    fn new_object(
        &mut self,
        blueprint_ident: &str,
        features: Vec<&str>,
        schema: Option<InstanceSchema>,
        fields: Vec<FieldValue>,
        kv_entries: BTreeMap<u8, BTreeMap<Vec<u8>, KVEntry>>,
    ) -> Result<NodeId, RuntimeError> {
        let actor = self.api.kernel_get_system_state().current;
        let package_address = actor.blueprint_id().package_address;
        let instance_context = self.instance_context();
        let blueprint = BlueprintId::new(&package_address, blueprint_ident);

        self.new_object_internal(
            &blueprint,
            features,
            instance_context,
            schema,
            fields,
            kv_entries,
        )
    }

    // Costing through kernel
    #[trace_resources]
    fn allocate_global_address(
        &mut self,
        blueprint_id: BlueprintId,
    ) -> Result<(GlobalAddressReservation, GlobalAddress), RuntimeError> {
        let global_address_node_id = self.api.kernel_allocate_node_id(
            IDAllocation::Object {
                blueprint_id: blueprint_id.clone(),
                global: true,
            }
            .entity_type(),
        )?;
        let global_address = GlobalAddress::try_from(global_address_node_id.0).unwrap();

        // Create global address reservation
        let global_address_reservation =
            self.prepare_global_address(blueprint_id, global_address)?;

        // NOTE: Because allocated global address is represented as an owned object and nobody is allowed
        // to drop it except the system during globalization, we don't track the lifecycle of
        // allocated addresses.

        Ok((global_address_reservation, global_address))
    }

    // Costing through kernel
    #[trace_resources]
    fn allocate_virtual_global_address(
        &mut self,
        blueprint_id: BlueprintId,
        global_address: GlobalAddress,
    ) -> Result<GlobalAddressReservation, RuntimeError> {
        let global_address_reservation =
            self.prepare_global_address(blueprint_id, global_address)?;

        Ok(global_address_reservation)
    }

    // Costing through kernel
    // FIXME: ensure that only the package actor can globalize its own blueprints
    #[trace_resources]
    fn globalize(
        &mut self,
        modules: BTreeMap<ObjectModuleId, NodeId>,
        address_reservation: Option<GlobalAddressReservation>,
    ) -> Result<GlobalAddress, RuntimeError> {
        // TODO: optimize by skipping address allocation
        let (global_address_reservation, global_address) =
            if let Some(reservation) = address_reservation {
                let address = self.get_reservation_address(reservation.0.as_node_id())?;
                (reservation, address)
            } else {
                let blueprint_id = self.resolve_blueprint_from_modules(&modules)?;
                self.allocate_global_address(blueprint_id)?
            };

        self.globalize_with_address_internal(modules, global_address_reservation)?;

        Ok(global_address)
    }

    // Costing through kernel
    #[trace_resources]
    fn globalize_with_address_and_create_inner_object(
        &mut self,
        modules: BTreeMap<ObjectModuleId, NodeId>,
        address_reservation: GlobalAddressReservation,
        inner_object_blueprint: &str,
        inner_object_fields: Vec<FieldValue>,
    ) -> Result<(GlobalAddress, NodeId), RuntimeError> {
        let actor_blueprint = self.resolve_blueprint_from_modules(&modules)?;

        let global_address = self.globalize_with_address_internal(modules, address_reservation)?;

        let blueprint = BlueprintId::new(&actor_blueprint.package_address, inner_object_blueprint);

        let inner_object = self.new_object_internal(
            &blueprint,
            vec![],
            Some(InstanceContext {
                outer_object: global_address,
                outer_blueprint: actor_blueprint.blueprint_name,
            }),
            None,
            inner_object_fields,
            btreemap!(),
        )?;

        Ok((global_address, inner_object))
    }

    // Costing through kernel
    #[trace_resources]
    fn call_method_advanced(
        &mut self,
        receiver: &NodeId,
        module_id: ObjectModuleId,
        direct_access: bool,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        // Direct access methods should never have access to a global address
        let global_address = if !direct_access {
            let node_visibility = self.api.kernel_get_node_visibility(receiver);

            // Retrieve the global address of the receiver node
            let mut get_global_address = |node_visibility: NodeVisibility| {
                for visibility in node_visibility.0 {
                    match visibility {
                        Visibility::StableReference(StableReferenceType::Global) => {
                            return Some(GlobalAddress::new_or_panic(receiver.clone().into()))
                        }

                        // Direct access references dont provide any info regarding global address so continue
                        Visibility::StableReference(StableReferenceType::DirectAccess) => {
                            continue;
                        }

                        // Anything frame owned does not have a global address
                        Visibility::FrameOwned => return None,

                        // If borrowed or actor then we just use the current actor's global address
                        // e.g. if the parent to the node is frame owned then the current actor's global
                        // address would be None
                        Visibility::Borrowed | Visibility::Actor => {
                            return self.api.kernel_get_system_state().current.global_address();
                        }
                    }
                }
                None
            };

            get_global_address(node_visibility)
        } else {
            None
        };

        // Key Value Stores do not have methods so we remove that possibility here
        let object_info = self.get_node_object_info(&receiver)?;
        if !object_info.module_versions.contains_key(&module_id) {
            return Err(RuntimeError::SystemError(
                SystemError::ObjectModuleDoesNotExist(module_id),
            ));
        }

        let invocation = KernelInvocation {
            actor: Actor::method(
                global_address,
                receiver.clone(),
                module_id,
                method_name.to_string(),
                object_info,
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

    // Costing through kernel
    #[trace_resources]
    fn get_node_object_info(&mut self, node_id: &NodeId) -> Result<NodeObjectInfo, RuntimeError> {
        let type_info = TypeInfoBlueprint::get_type(&node_id, self.api)?;
        let object_info = match type_info {
            TypeInfoSubstate::Object(info) => info,
            _ => return Err(RuntimeError::SystemError(SystemError::NotAnObject)),
        };

        Ok(object_info)
    }

    // Costing through kernel
    #[trace_resources]
    fn get_reservation_address(&mut self, node_id: &NodeId) -> Result<GlobalAddress, RuntimeError> {
        let type_info = TypeInfoBlueprint::get_type(&node_id, self.api)?;
        let address = match type_info {
            TypeInfoSubstate::GlobalAddressReservation(address) => address,
            _ => {
                return Err(RuntimeError::SystemError(
                    SystemError::NotAnAddressReservation,
                ))
            }
        };

        Ok(address)
    }

    // Costing through kernel
    #[trace_resources]
    fn drop_object(&mut self, node_id: &NodeId) -> Result<Vec<Vec<u8>>, RuntimeError> {
        let info = self.get_node_object_info(node_id)?;
        let mut is_drop_allowed = false;

        // FIXME: what's the right model, trading off between flexibility and security?
        // If the actor is the object's outer object
        match info.main_blueprint_info.blueprint_type {
            BlueprintObjectType::Inner { outer_object } => {
                if let Some(instance_context) = self.instance_context() {
                    if instance_context.outer_object.eq(&outer_object) {
                        is_drop_allowed = true;
                    }
                }
            }
            BlueprintObjectType::Outer { .. } => {}
        }

        // If the actor is a function within the same blueprint
        let actor = self.api.kernel_get_system_state().current;
        if let Actor::Function {
            blueprint_id: blueprint,
            ..
        } = actor
        {
            if blueprint.eq(&info.main_blueprint_info.blueprint_id) {
                is_drop_allowed = true;
            }
        }

        if !is_drop_allowed {
            return Err(RuntimeError::SystemError(
                SystemError::InvalidDropNodeAccess(Box::new(InvalidDropNodeAccess {
                    node_id: node_id.clone(),
                    package_address: info.main_blueprint_info.blueprint_id.package_address,
                    blueprint_name: info.main_blueprint_info.blueprint_id.blueprint_name,
                })),
            ));
        }

        let mut node_substates = self.api.kernel_drop_node(&node_id)?;
        let fields = if let Some(user_substates) = node_substates.remove(&MAIN_BASE_PARTITION) {
            user_substates
                .into_iter()
                .map(|(_key, v)| {
                    let substate: FieldSubstate<ScryptoValue> = v.as_typed().unwrap();
                    scrypto_encode(&substate.value.0).unwrap()
                })
                .collect()
        } else {
            vec![]
        };

        Ok(fields)
    }
}

impl<'a, Y, V> ClientKeyValueEntryApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    // Costing through kernel
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

        self.api.kernel_read_substate(handle).map(|v| {
            let wrapper: KeyValueEntrySubstate<ScryptoValue> = v.as_typed().unwrap();
            scrypto_encode(&wrapper.value).unwrap()
        })
    }

    // Costing through kernel
    // FIXME: Should this release lock or continue allow to mutate entry until lock released?
    fn key_value_entry_lock(&mut self, handle: KeyValueEntryHandle) -> Result<(), RuntimeError> {
        let LockInfo { data, .. } = self.api.kernel_get_lock_info(handle)?;
        match data {
            SystemLockData::KeyValueEntry(
                KeyValueEntryLockData::Write { .. } | KeyValueEntryLockData::BlueprintWrite { .. },
            ) => {}
            _ => {
                return Err(RuntimeError::SystemError(
                    SystemError::NotAKeyValueWriteLock,
                ));
            }
        };

        let v = self.api.kernel_read_substate(handle)?;
        let mut kv_entry: KeyValueEntrySubstate<ScryptoValue> = v.as_typed().unwrap();
        kv_entry.lock();
        let indexed = IndexedScryptoValue::from_typed(&kv_entry);
        self.api.kernel_write_substate(handle, indexed)?;
        Ok(())
    }

    // Costing through kernel
    fn key_value_entry_remove(
        &mut self,
        handle: KeyValueEntryHandle,
    ) -> Result<Vec<u8>, RuntimeError> {
        let current_value = self
            .api
            .kernel_read_substate(handle)
            .map(|v| v.as_slice().to_vec())?;

        let mut kv_entry: KeyValueEntrySubstate<ScryptoValue> =
            scrypto_decode(&current_value).unwrap();
        let value = kv_entry.remove();
        self.kernel_write_substate(handle, IndexedScryptoValue::from_typed(&kv_entry))?;

        let current_value = scrypto_encode(&value).unwrap();

        Ok(current_value)
    }

    // Costing through kernel
    #[trace_resources]
    fn key_value_entry_set(
        &mut self,
        handle: KeyValueEntryHandle,
        buffer: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        let LockInfo { data, .. } = self.api.kernel_get_lock_info(handle)?;

        let can_own = match data {
            SystemLockData::KeyValueEntry(KeyValueEntryLockData::BlueprintWrite {
                blueprint_id,
                instance_schema,
                type_pointer: schema_pointer,
                can_own,
            }) => {
                self.validate_payload_at_type_pointer(
                    &blueprint_id,
                    &instance_schema,
                    schema_pointer,
                    &buffer,
                )?;

                can_own
            }
            SystemLockData::KeyValueEntry(KeyValueEntryLockData::Write {
                schema,
                index,
                can_own,
            }) => {
                self.validate_payload(&buffer, &schema, index, SchemaOrigin::KeyValueStore {})
                    .map_err(|e| {
                        RuntimeError::SystemError(SystemError::InvalidSubstateWrite(
                            e.error_message(&schema),
                        ))
                    })?;

                can_own
            }
            _ => {
                return Err(RuntimeError::SystemError(
                    SystemError::NotAKeyValueWriteLock,
                ));
            }
        };

        let substate =
            IndexedScryptoValue::from_slice(&buffer).expect("Should be valid due to payload check");

        if !can_own {
            let own = substate.owned_nodes();
            if !own.is_empty() {
                return Err(RuntimeError::SystemError(
                    SystemError::InvalidKeyValueStoreOwnership,
                ));
            }
        }

        let value = substate.as_scrypto_value().clone();
        let kv_entry = KeyValueEntrySubstate::entry(value);
        let indexed = IndexedScryptoValue::from_typed(&kv_entry);

        self.api.kernel_write_substate(handle, indexed)?;

        Ok(())
    }

    // Costing through kernel
    fn key_value_entry_close(&mut self, handle: KeyValueEntryHandle) -> Result<(), RuntimeError> {
        let LockInfo { data, .. } = self.api.kernel_get_lock_info(handle)?;
        if !data.is_kv_entry() {
            return Err(RuntimeError::SystemError(SystemError::NotAKeyValueStore));
        }

        self.api.kernel_close_substate(handle)
    }
}

impl<'a, Y, V> ClientKeyValueStoreApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    // Costing through kernel
    #[trace_resources]
    fn key_value_store_new(&mut self, schema: KeyValueStoreSchema) -> Result<NodeId, RuntimeError> {
        schema
            .schema
            .validate()
            .map_err(|e| RuntimeError::SystemError(SystemError::InvalidKeyValueStoreSchema(e)))?;

        let node_id = self
            .api
            .kernel_allocate_node_id(IDAllocation::KeyValueStore.entity_type())?;

        self.api.kernel_create_node(
            node_id,
            btreemap!(
                MAIN_BASE_PARTITION => btreemap!(),
                TYPE_INFO_FIELD_PARTITION => type_info_partition(
                    TypeInfoSubstate::KeyValueStore(KeyValueStoreInfo {
                        schema,
                    })
                ),
            ),
        )?;

        Ok(node_id)
    }

    // Costing through kernel
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

    // Costing through kernel
    #[trace_resources]
    fn key_value_store_open_entry(
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
                schema: info.schema.schema,
                index: info.schema.value,
                can_own: info.schema.can_own,
            })
        } else {
            SystemLockData::KeyValueEntry(KeyValueEntryLockData::Read)
        };

        let handle = self.api.kernel_open_substate_with_default(
            &node_id,
            MAIN_BASE_PARTITION,
            &SubstateKey::Map(key.clone()),
            flags,
            Some(|| {
                let kv_entry = KeyValueEntrySubstate::<()>::default();
                IndexedScryptoValue::from_typed(&kv_entry)
            }),
            lock_data,
        )?;

        if flags.contains(LockFlags::MUTABLE) {
            let mutability = self.api.kernel_read_substate(handle).map(|v| {
                let kv_entry: KeyValueEntrySubstate<ScryptoValue> = v.as_typed().unwrap();
                kv_entry.mutability
            })?;

            if let SubstateMutability::Immutable = mutability {
                return Err(RuntimeError::SystemError(
                    SystemError::MutatingImmutableSubstate,
                ));
            }
        }

        Ok(handle)
    }

    // Costing through kernel
    fn key_value_store_remove_entry(
        &mut self,
        node_id: &NodeId,
        key: &Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let handle = self.key_value_store_open_entry(node_id, key, LockFlags::MUTABLE)?;
        self.key_value_entry_remove_and_close_substate(handle)
    }
}

impl<'a, Y, V> ClientActorIndexApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    // Costing through kernel
    fn actor_index_insert(
        &mut self,
        object_handle: ObjectHandle,
        collection_index: CollectionIndex,
        key: Vec<u8>,
        buffer: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, partition_num) = self.get_actor_index(actor_object_type, collection_index)?;

        let value = IndexedScryptoValue::from_vec(buffer)
            .map_err(|e| RuntimeError::SystemError(SystemError::InvalidScryptoValue(e)))?;

        if !value.owned_nodes().is_empty() {
            return Err(RuntimeError::SystemError(
                SystemError::CannotStoreOwnedInIterable,
            ));
        }

        self.api
            .kernel_set_substate(&node_id, partition_num, SubstateKey::Map(key), value)
    }

    // Costing through kernel
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

    // Costing through kernel
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

    // Costing through kernel
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
    // Costing through kernel
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

        let value = IndexedScryptoValue::from_vec(buffer)
            .map_err(|e| RuntimeError::SystemError(SystemError::InvalidScryptoValue(e)))?;

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

    // Costing through kernel
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

    // Costing through kernel
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
    // Costing through kernel
    fn call_function(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let invocation = KernelInvocation {
            actor: Actor::function(
                BlueprintId::new(&package_address, blueprint_name),
                function_name.to_string(),
            ),
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
    // No costing should be applied
    fn consume_cost_units(
        &mut self,
        costing_entry: ClientCostingEntry,
    ) -> Result<(), RuntimeError> {
        // Skip client-side costing requested by TransactionProcessor
        if self.api.kernel_get_current_depth() == 1 {
            return Ok(());
        }

        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(match costing_entry {
                ClientCostingEntry::RunNativeCode {
                    package_address,
                    export_name,
                    input_size,
                } => CostingEntry::RunNativeCode {
                    package_address,
                    export_name,
                    input_size,
                },
                ClientCostingEntry::RunWasmCode {
                    package_address,
                    export_name,
                    wasm_execution_units,
                } => CostingEntry::RunWasmCode {
                    package_address,
                    export_name,
                    wasm_execution_units,
                },
                ClientCostingEntry::PrepareWasmCode { size } => {
                    CostingEntry::PrepareWasmCode { size }
                }
            })
    }

    #[trace_resources]
    fn credit_cost_units(
        &mut self,
        vault_id: NodeId,
        locked_fee: LiquidFungibleResource,
        contingent: bool,
    ) -> Result<LiquidFungibleResource, RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(CostingEntry::LockFee)?;

        self.api
            .kernel_get_system()
            .modules
            .credit_cost_units(vault_id, locked_fee, contingent)
    }

    fn cost_unit_limit(&mut self) -> Result<u32, RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(CostingEntry::QueryFeeReserve)?;

        if let Some(fee_reserve) = self.api.kernel_get_system().modules.fee_reserve() {
            Ok(fee_reserve.cost_unit_limit())
        } else {
            Err(RuntimeError::SystemError(
                SystemError::CostingModuleNotEnabled,
            ))
        }
    }

    fn cost_unit_price(&mut self) -> Result<Decimal, RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(CostingEntry::QueryFeeReserve)?;

        if let Some(fee_reserve) = self.api.kernel_get_system().modules.fee_reserve() {
            Ok(fee_reserve.cost_unit_price())
        } else {
            Err(RuntimeError::SystemError(
                SystemError::CostingModuleNotEnabled,
            ))
        }
    }

    fn usd_price(&mut self) -> Result<Decimal, RuntimeError> {
        if let Some(fee_reserve) = self.api.kernel_get_system().modules.fee_reserve() {
            Ok(fee_reserve.usd_price())
        } else {
            Err(RuntimeError::SystemError(
                SystemError::CostingModuleNotEnabled,
            ))
        }
    }

    fn max_per_function_royalty_in_xrd(&mut self) -> Result<Decimal, RuntimeError> {
        if let Some(costing) = self.api.kernel_get_system().modules.costing() {
            Ok(costing.max_per_function_royalty_in_xrd)
        } else {
            Err(RuntimeError::SystemError(
                SystemError::CostingModuleNotEnabled,
            ))
        }
    }

    fn tip_percentage(&mut self) -> Result<u32, RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(CostingEntry::QueryFeeReserve)?;

        if let Some(fee_reserve) = self.api.kernel_get_system().modules.fee_reserve() {
            Ok(fee_reserve.tip_percentage())
        } else {
            Err(RuntimeError::SystemError(
                SystemError::CostingModuleNotEnabled,
            ))
        }
    }

    fn fee_balance(&mut self) -> Result<Decimal, RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(CostingEntry::QueryFeeReserve)?;

        if let Some(fee_reserve) = self.api.kernel_get_system().modules.fee_reserve() {
            Ok(fee_reserve.fee_balance())
        } else {
            Err(RuntimeError::SystemError(
                SystemError::CostingModuleNotEnabled,
            ))
        }
    }
}

impl<'a, Y, V> ClientActorApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    // Costing through kernel
    #[trace_resources]
    fn actor_open_field(
        &mut self,
        object_handle: ObjectHandle,
        field_index: u8,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, partition_num, schema_pointer, blueprint_id) =
            self.get_actor_field(actor_object_type, field_index)?;

        // TODO: Remove
        if flags.contains(LockFlags::UNMODIFIED_BASE) || flags.contains(LockFlags::FORCE_WRITE) {
            if !(blueprint_id.eq(&BlueprintId::new(
                &RESOURCE_PACKAGE,
                FUNGIBLE_VAULT_BLUEPRINT,
            ))) {
                return Err(RuntimeError::SystemError(SystemError::InvalidLockFlags));
            }
        }

        let lock_data = if flags.contains(LockFlags::MUTABLE) {
            FieldLockData::Write {
                blueprint_id,
                type_pointer: schema_pointer,
            }
        } else {
            FieldLockData::Read
        };

        let handle = self.api.kernel_open_substate(
            &node_id,
            partition_num,
            &SubstateKey::Field(field_index),
            flags,
            SystemLockData::Field(lock_data),
        )?;

        if flags.contains(LockFlags::MUTABLE) {
            let mutability = self.api.kernel_read_substate(handle).map(|v| {
                let field: FieldSubstate<ScryptoValue> = v.as_typed().unwrap();
                field.mutability
            })?;

            if let SubstateMutability::Immutable = mutability {
                return Err(RuntimeError::SystemError(
                    SystemError::MutatingImmutableFieldSubstate(object_handle, field_index),
                ));
            }
        }

        Ok(handle)
    }

    #[trace_resources]
    fn actor_get_info(&mut self) -> Result<NodeObjectInfo, RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(CostingEntry::QueryActor)?;

        let actor = self.api.kernel_get_system_state().current;
        let object_info = actor
            .try_as_method()
            .map(|m| m.node_object_info.clone())
            .ok_or(RuntimeError::SystemError(SystemError::NotAMethod))?;

        Ok(object_info)
    }

    #[trace_resources]
    fn actor_get_node_id(&mut self) -> Result<NodeId, RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(CostingEntry::QueryActor)?;

        let actor = self.api.kernel_get_system_state().current;
        match actor {
            Actor::Method(MethodActor { node_id, .. }) => Ok(*node_id),
            _ => Err(RuntimeError::SystemError(SystemError::NotAMethod)),
        }
    }

    #[trace_resources]
    fn actor_get_global_address(&mut self) -> Result<GlobalAddress, RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(CostingEntry::QueryActor)?;

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
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(CostingEntry::QueryActor)?;

        let actor = self.api.kernel_get_system_state().current;
        Ok(actor.blueprint_id().clone())
    }

    // Costing through kernel
    #[trace_resources]
    fn actor_call_self_module_method(
        &mut self,
        module_id: ObjectModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let node_id = self.actor_get_node_id()?;
        self.call_method_advanced(&node_id, module_id, false, method_name, args)
    }

    #[trace_resources]
    fn actor_is_feature_enabled(
        &mut self,
        object_handle: ObjectHandle,
        feature: &str,
    ) -> Result<bool, RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(CostingEntry::QueryActor)?;

        let actor_object_type: ActorObjectType = object_handle.try_into()?;
        let (node_id, _module_id) = self.get_actor_object_id(actor_object_type)?;
        self.is_feature_enabled(&node_id, feature)
    }
}

impl<'a, Y, V> ClientActorKeyValueEntryApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    // Costing through kernel
    #[trace_resources]
    fn actor_open_key_value_entry(
        &mut self,
        object_handle: ObjectHandle,
        collection_index: CollectionIndex,
        key: &Vec<u8>,
        flags: LockFlags,
    ) -> Result<KeyValueEntryHandle, RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, partition_num, kv_schema, instance_schema, blueprint_id) =
            self.get_actor_kv_partition(actor_object_type, collection_index)?;

        self.validate_payload_against_blueprint_schema(
            &blueprint_id,
            &instance_schema,
            &[(key, kv_schema.key)],
        )?;

        let lock_data = if flags.contains(LockFlags::MUTABLE) {
            KeyValueEntryLockData::BlueprintWrite {
                blueprint_id,
                instance_schema,
                type_pointer: kv_schema.value,
                can_own: kv_schema.can_own,
            }
        } else {
            KeyValueEntryLockData::Read
        };

        let handle = self.api.kernel_open_substate_with_default(
            &node_id,
            partition_num,
            &SubstateKey::Map(key.to_vec()),
            flags,
            Some(|| {
                let kv_entry = KeyValueEntrySubstate::<()>::default();
                IndexedScryptoValue::from_typed(&kv_entry)
            }),
            SystemLockData::KeyValueEntry(lock_data),
        )?;

        if flags.contains(LockFlags::MUTABLE) {
            let substate: KeyValueEntrySubstate<ScryptoValue> =
                self.api.kernel_read_substate(handle)?.as_typed().unwrap();

            if !substate.is_mutable() {
                return Err(RuntimeError::SystemError(
                    SystemError::MutatingImmutableSubstate,
                ));
            }
        }

        Ok(handle)
    }

    // Costing through kernel
    fn actor_remove_key_value_entry(
        &mut self,
        object_handle: ObjectHandle,
        collection_index: CollectionIndex,
        key: &Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let handle = self.actor_open_key_value_entry(
            object_handle,
            collection_index,
            key,
            LockFlags::MUTABLE,
        )?;
        self.key_value_entry_remove_and_close_substate(handle)
    }
}

impl<'a, Y, V> ClientAuthApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
    fn get_auth_zone(&mut self) -> Result<NodeId, RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(CostingEntry::QueryAuthZone)?;

        if let Some(auth_zone_id) = self.api.kernel_get_system().modules.auth_zone_id() {
            Ok(auth_zone_id.into())
        } else {
            Err(RuntimeError::SystemError(SystemError::AuthModuleNotEnabled))
        }
    }

    #[trace_resources]
    fn assert_access_rule(&mut self, rule: AccessRule) -> Result<(), RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(CostingEntry::AssertAccessRule)?;

        // Fetch the tip auth zone
        let auth_zone_id = self.get_auth_zone()?;

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

impl<'a, Y, V> ClientExecutionTraceApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    // No costing should be applied
    #[trace_resources]
    fn update_instruction_index(&mut self, new_index: usize) -> Result<(), RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .update_instruction_index(new_index);
        Ok(())
    }
}

impl<'a, Y, V> ClientTransactionRuntimeApi<RuntimeError> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    #[trace_resources]
    fn emit_event(&mut self, event_name: String, event_data: Vec<u8>) -> Result<(), RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(CostingEntry::EmitEvent {
                size: event_data.len(),
            })?;

        // Locking the package info substate associated with the emitter's package
        let type_pointer = {
            let actor = self.api.kernel_get_system_state().current;

            // Getting the package address and blueprint name associated with the actor
            let (instance_schema, blueprint_id) = match actor {
                Actor::Method(MethodActor {
                    node_id, module_id, ..
                }) => {
                    let module_id = *module_id;
                    let node_id = *node_id;
                    let blueprint_obj_info = self.get_blueprint_object_info(&node_id, module_id)?;
                    (
                        blueprint_obj_info.instance_schema,
                        blueprint_obj_info.blueprint_id,
                    )
                }
                Actor::Function {
                    blueprint_id: ref blueprint,
                    ..
                } => (None, blueprint.clone()),
                _ => {
                    return Err(RuntimeError::SystemError(SystemError::EventError(
                        EventError::InvalidActor,
                    )))
                }
            };

            let blueprint_interface = self.get_blueprint_default_interface(blueprint_id.clone())?;

            let type_pointer = blueprint_interface
                .get_event_type_pointer(event_name.as_str())
                .ok_or_else(|| {
                    RuntimeError::SystemError(SystemError::PayloadValidationAgainstSchemaError(
                        PayloadValidationAgainstSchemaError::EventDoesNotExist(event_name.clone()),
                    ))
                })?;

            self.validate_payload_against_blueprint_schema(
                &blueprint_id,
                &instance_schema,
                &[(&event_data, type_pointer.clone())],
            )?;

            type_pointer
        };

        // Construct the event type identifier based on the current actor
        let actor = self.api.kernel_get_system_state().current;
        let event_type_identifier = match actor {
            Actor::Method(MethodActor {
                node_id, module_id, ..
            }) => Ok(EventTypeIdentifier(
                Emitter::Method(node_id.clone(), module_id.clone()),
                type_pointer,
            )),
            Actor::Function {
                blueprint_id: ref blueprint,
                ..
            } => Ok(EventTypeIdentifier(
                Emitter::Function(
                    blueprint.package_address.into(),
                    ObjectModuleId::Main,
                    blueprint.blueprint_name.to_string(),
                ),
                type_pointer,
            )),
            _ => Err(RuntimeError::SystemModuleError(
                SystemModuleError::EventError(Box::new(EventError::InvalidActor)),
            )),
        }?;

        // Adding the event to the event store
        self.api
            .kernel_get_system()
            .modules
            .add_event(event_type_identifier, event_data)?;

        Ok(())
    }

    #[trace_resources]
    fn emit_log(&mut self, level: Level, message: String) -> Result<(), RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(CostingEntry::EmitLog {
                size: message.len(),
            })?;

        self.api
            .kernel_get_system()
            .modules
            .add_log(level, message)?;

        Ok(())
    }

    fn panic(&mut self, message: String) -> Result<(), RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(CostingEntry::Panic {
                size: message.len(),
            })?;

        self.api
            .kernel_get_system()
            .modules
            .set_panic_message(message.clone())?;

        Err(RuntimeError::ApplicationError(ApplicationError::Panic(
            message,
        )))
    }

    #[trace_resources]
    fn get_transaction_hash(&mut self) -> Result<Hash, RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(CostingEntry::QueryTransactionHash)?;

        if let Some(hash) = self.api.kernel_get_system().modules.transaction_hash() {
            Ok(hash)
        } else {
            Err(RuntimeError::SystemError(
                SystemError::TransactionRuntimeModuleNotEnabled,
            ))
        }
    }

    #[trace_resources]
    fn generate_ruid(&mut self) -> Result<[u8; 32], RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(CostingEntry::GenerateRuid)?;

        if let Some(ruid) = self.api.kernel_get_system().modules.generate_ruid() {
            Ok(ruid)
        } else {
            Err(RuntimeError::SystemError(
                SystemError::TransactionRuntimeModuleNotEnabled,
            ))
        }
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
}

impl<'a, Y, V> KernelSubstateApi<SystemLockData> for SystemService<'a, Y, V>
where
    Y: KernelApi<SystemConfig<V>>,
    V: SystemCallbackObject,
{
    fn kernel_open_substate_with_default(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        default: Option<fn() -> IndexedScryptoValue>,
        data: SystemLockData,
    ) -> Result<LockHandle, RuntimeError> {
        self.api.kernel_open_substate_with_default(
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

    fn kernel_close_substate(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError> {
        self.api.kernel_close_substate(lock_handle)
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
