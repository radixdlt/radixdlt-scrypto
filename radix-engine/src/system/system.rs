use super::id_allocation::IDAllocation;
use super::system_modules::costing::ExecutionCostingEntry;
use crate::errors::{
    ApplicationError, CannotGlobalizeError, CreateObjectError, InvalidDropAccess,
    InvalidGlobalizeAccess, InvalidModuleType, RuntimeError, SystemError, SystemModuleError,
};
use crate::errors::{EventError, SystemUpstreamError};
use crate::kernel::actor::{Actor, FunctionActor, InstanceContext, MethodActor};
use crate::kernel::call_frame::{NodeVisibility, ReferenceOrigin};
use crate::kernel::kernel_api::*;
use crate::system::node_init::type_info_partition;
use crate::system::node_modules::type_info::{TypeInfoBlueprint, TypeInfoSubstate};
use crate::system::system_callback::{
    FieldLockData, KeyValueEntryLockData, SystemConfig, SystemLockData,
};
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_modules::execution_trace::{BucketSnapshot, ProofSnapshot};
use crate::system::system_modules::transaction_runtime::Event;
use crate::system::system_modules::SystemModuleMixer;
use crate::system::system_type_checker::{
    BlueprintTypeTarget, KVStoreValidationTarget, SchemaValidationMeta, SystemMapper,
};
use crate::track::interface::NodeSubstates;
use crate::types::*;
use radix_engine_interface::api::actor_index_api::ClientActorIndexApi;
use radix_engine_interface::api::field_api::{FieldHandle, LockFlags};
use radix_engine_interface::api::key_value_entry_api::{
    ClientKeyValueEntryApi, KeyValueEntryHandle,
};
use radix_engine_interface::api::key_value_store_api::{
    ClientKeyValueStoreApi, KeyValueStoreGenericArgs,
};
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::schema::{Condition, KeyValueStoreGenericSubstitutions};
use radix_engine_store_interface::db_key_mapper::SubstateKeyContent;
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

enum EmitterActor {
    CurrentActor,
    AsObject(NodeId, ObjectModuleId),
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

    fn validate_new_object(
        &mut self,
        blueprint_id: &BlueprintId,
        blueprint_interface: &BlueprintInterface,
        outer_obj_info: OuterObjectInfo,
        features: BTreeSet<String>,
        outer_blueprint_features: &BTreeSet<String>,
        generic_args: GenericArgs,
        fields: Vec<FieldValue>,
        mut kv_entries: BTreeMap<u8, BTreeMap<Vec<u8>, KVEntry>>,
    ) -> Result<(BlueprintInfo, NodeSubstates), RuntimeError> {
        // Validate generic arguments
        let (generic_substitutions, additional_schemas) = {
            let mut additional_schemas = index_map_new();

            if let Some(schema) = generic_args.additional_schema {
                validate_schema(&schema)
                    .map_err(|_| RuntimeError::SystemError(SystemError::InvalidGenericArgs))?;
                let schema_hash = schema.generate_schema_hash();
                additional_schemas.insert(schema_hash, schema);
            }

            self.validate_bp_generic_args(
                blueprint_interface,
                &additional_schemas,
                &generic_args.generic_substitutions,
            )
            .map_err(|e| RuntimeError::SystemError(SystemError::TypeCheckError(e)))?;

            (generic_args.generic_substitutions, additional_schemas)
        };

        let blueprint_info = BlueprintInfo {
            outer_obj_info,
            features: features.clone(),
            blueprint_id: blueprint_id.clone(),
            generic_substitutions: generic_substitutions.clone(),
        };

        let validation_target = BlueprintTypeTarget {
            blueprint_info,
            meta: SchemaValidationMeta::NewObject {
                additional_schemas: additional_schemas.clone().into_iter().collect(),
            },
        };

        // Fields
        let system_fields = {
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

            let mut system_fields = Vec::new();

            if let Some((_partition_description, field_schemas)) = &blueprint_interface.state.fields
            {
                for (i, field) in fields.into_iter().enumerate() {
                    // Check for any feature conditions
                    match &field_schemas[i].condition {
                        Condition::IfFeature(feature) => {
                            if !features.contains(feature) {
                                system_fields.push(None);
                                continue;
                            }
                        }
                        Condition::IfOuterFeature(feature) => {
                            if !outer_blueprint_features.contains(feature) {
                                system_fields.push(None);
                                continue;
                            }
                        }
                        Condition::Always => {}
                    }

                    self.validate_blueprint_payload(
                        &validation_target,
                        BlueprintPayloadIdentifier::Field(i as u8),
                        &field.value,
                    )?;

                    system_fields.push(Some(field));
                }
            }

            system_fields
        };

        // Collections
        {
            for (collection_index, entries) in &kv_entries {
                let payloads: Vec<(&Vec<u8>, &Vec<u8>)> = entries
                    .iter()
                    .filter_map(|(key, entry)| entry.value.as_ref().map(|e| (key, e)))
                    .collect();

                self.validate_blueprint_kv_collection(
                    &validation_target,
                    *collection_index,
                    &payloads,
                )?;
            }

            for (collection_index, ..) in blueprint_interface.state.collections.iter().enumerate() {
                let index = collection_index as u8;
                if !kv_entries.contains_key(&index) {
                    kv_entries.insert(index, BTreeMap::new());
                }
            }
        }

        let mut node_substates = SystemMapper::system_struct_to_node_substates(
            &blueprint_interface.state,
            (system_fields, kv_entries),
            MAIN_BASE_PARTITION,
        );

        let schema_partition = node_substates
            .entry(SCHEMAS_PARTITION)
            .or_insert(BTreeMap::new());

        for (schema_hash, schema) in additional_schemas {
            let key = SubstateKey::Map(scrypto_encode(&schema_hash).unwrap());
            let value =
                IndexedScryptoValue::from_typed(&KeyValueEntrySubstate::locked_entry(schema));
            schema_partition.insert(key, value);
        }

        Ok((validation_target.blueprint_info, node_substates))
    }

    pub fn get_blueprint_default_interface(
        &mut self,
        blueprint_id: BlueprintId,
    ) -> Result<BlueprintInterface, RuntimeError> {
        let bp_version_key = BlueprintVersionKey::new_default(blueprint_id.blueprint_name);
        Ok(self
            .load_blueprint_definition(blueprint_id.package_address, &bp_version_key)?
            .interface)
    }

    pub fn load_blueprint_definition(
        &mut self,
        package_address: PackageAddress,
        bp_version_key: &BlueprintVersionKey,
    ) -> Result<BlueprintDefinition, RuntimeError> {
        let canonical_bp_id = CanonicalBlueprintId {
            address: package_address,
            blueprint: bp_version_key.blueprint.to_string(),
            version: bp_version_key.version.clone(),
        };

        // TODO: Use internment to cache blueprint interface rather than object cache?
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

        self.api.kernel_pin_node(global_address_reservation)?;

        Ok(GlobalAddressReservation(Own(global_address_reservation)))
    }

    pub fn get_node_type_info(
        &mut self,
        node_id: &NodeId,
    ) -> Result<TypeInfoSubstate, RuntimeError> {
        let handle = self.api.kernel_open_substate(
            node_id,
            TYPE_INFO_FIELD_PARTITION,
            &TypeInfoField::TypeInfo.into(),
            LockFlags::read_only(),
            SystemLockData::default(),
        )?;
        let value = self.api.kernel_read_substate(handle)?;
        let type_info = value.as_typed::<TypeInfoSubstate>().unwrap();
        self.api.kernel_close_substate(handle)?;
        Ok(type_info)
    }

    fn new_object_internal(
        &mut self,
        blueprint_id: &BlueprintId,
        features: Vec<&str>,
        instance_context: Option<InstanceContext>,
        generic_args: GenericArgs,
        fields: Vec<FieldValue>,
        kv_entries: BTreeMap<u8, BTreeMap<Vec<u8>, KVEntry>>,
    ) -> Result<NodeId, RuntimeError> {
        let blueprint_interface = self.get_blueprint_default_interface(blueprint_id.clone())?;
        let expected_outer_blueprint = blueprint_interface.blueprint_type.clone();

        let object_features: BTreeSet<String> =
            features.into_iter().map(|s| s.to_string()).collect();

        let (outer_obj_info, outer_object_features) =
            if let BlueprintType::Inner { outer_blueprint } = &expected_outer_blueprint {
                match instance_context {
                    Some(context) => {
                        let info = self.get_object_info(context.outer_object.as_node_id())?;

                        if !info
                            .blueprint_info
                            .blueprint_id
                            .blueprint_name
                            .eq(outer_blueprint)
                        {
                            return Err(RuntimeError::SystemError(
                                SystemError::InvalidChildObjectCreation,
                            ));
                        }

                        (
                            OuterObjectInfo::Some {
                                outer_object: context.outer_object,
                            },
                            info.blueprint_info.features,
                        )
                    }
                    _ => {
                        return Err(RuntimeError::SystemError(
                            SystemError::InvalidChildObjectCreation,
                        ));
                    }
                }
            } else {
                // Validate features
                for feature in &object_features {
                    if !blueprint_interface.feature_set.contains(feature) {
                        return Err(RuntimeError::SystemError(SystemError::InvalidFeature(
                            feature.to_string(),
                        )));
                    }
                }

                (OuterObjectInfo::None, BTreeSet::new())
            };

        let (blueprint_info, mut node_substates) = self.validate_new_object(
            blueprint_id,
            &blueprint_interface,
            outer_obj_info,
            object_features,
            &outer_object_features,
            generic_args,
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

        node_substates.insert(
            TYPE_INFO_FIELD_PARTITION,
            type_info_partition(TypeInfoSubstate::Object(ObjectInfo {
                global: false,
                module_versions: btreemap!(
                    ObjectModuleId::Main => BlueprintVersion::default(),
                ),
                blueprint_info,
            })),
        );

        self.api.kernel_create_node(node_id, node_substates)?;

        if blueprint_interface.is_transient {
            self.api.kernel_pin_node(node_id)?;
        }

        if let Some((partition_offset, fields)) = blueprint_interface.state.fields {
            for (index, field) in fields.iter().enumerate() {
                if let FieldTransience::TransientStatic { .. } = field.transience {
                    let partition_number = match partition_offset {
                        PartitionDescription::Physical(partition_number) => partition_number,
                        PartitionDescription::Logical(offset) => {
                            MAIN_BASE_PARTITION.at_offset(offset).unwrap()
                        }
                    };
                    self.api.kernel_mark_substate_as_transient(
                        node_id,
                        partition_number,
                        SubstateKey::Field(index as u8),
                    )?;
                }
            }
        }

        Ok(node_id.into())
    }

    fn emit_event_internal(
        &mut self,
        actor: EmitterActor,
        event_name: String,
        event_data: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        self.api.kernel_get_system().modules.apply_execution_cost(
            ExecutionCostingEntry::EmitEvent {
                size: event_data.len(),
            },
        )?;

        // Locking the package info substate associated with the emitter's package
        // Getting the package address and blueprint name associated with the actor
        let validation_target = match &actor {
            EmitterActor::AsObject(node_id, module_id) => {
                let bp_info = self.get_blueprint_info(node_id, *module_id)?;

                BlueprintTypeTarget {
                    blueprint_info: bp_info,
                    meta: SchemaValidationMeta::ExistingObject {
                        additional_schemas: *node_id,
                    },
                }
            }
            EmitterActor::CurrentActor => self.get_actor_type_target()?,
        };

        self.validate_blueprint_payload(
            &validation_target,
            BlueprintPayloadIdentifier::Event(event_name.clone()),
            &event_data,
        )?;

        // Construct the event type identifier based on the current actor
        let event_type_identifier = match actor {
            EmitterActor::AsObject(node_id, module_id, ..) => Ok(EventTypeIdentifier(
                Emitter::Method(node_id, module_id),
                event_name,
            )),
            EmitterActor::CurrentActor => match self.current_actor() {
                Actor::Method(MethodActor {
                    node_id, module_id, ..
                }) => Ok(EventTypeIdentifier(
                    Emitter::Method(node_id, module_id),
                    event_name,
                )),
                Actor::Function(FunctionActor { blueprint_id, .. }) => Ok(EventTypeIdentifier(
                    Emitter::Function(blueprint_id.clone()),
                    event_name,
                )),
                _ => Err(RuntimeError::SystemModuleError(
                    SystemModuleError::EventError(Box::new(EventError::InvalidActor)),
                )),
            },
        }?;

        let event = Event {
            type_identifier: event_type_identifier,
            payload: event_data,
            discard_on_failure: true,
        };

        // Adding the event to the event store
        self.api.kernel_get_system().modules.add_event(event)?;

        Ok(())
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

    pub fn get_blueprint_info(
        &mut self,
        node_id: &NodeId,
        module_id: ObjectModuleId,
    ) -> Result<BlueprintInfo, RuntimeError> {
        let info = match module_id {
            ObjectModuleId::Main => self.get_object_info(node_id)?.blueprint_info,
            _ => BlueprintInfo {
                blueprint_id: module_id.static_blueprint().unwrap(),
                outer_obj_info: OuterObjectInfo::None,
                features: BTreeSet::default(),
                generic_substitutions: vec![],
            },
        };

        Ok(info)
    }

    pub fn get_actor_type_target(&mut self) -> Result<BlueprintTypeTarget, RuntimeError> {
        let actor = self.current_actor();
        match actor {
            Actor::Root => Err(RuntimeError::SystemError(SystemError::RootHasNoType)),
            Actor::BlueprintHook(actor) => Ok(BlueprintTypeTarget {
                blueprint_info: BlueprintInfo {
                    blueprint_id: actor.blueprint_id.clone(),
                    outer_obj_info: OuterObjectInfo::None,
                    features: btreeset!(),
                    generic_substitutions: vec![],
                },
                meta: SchemaValidationMeta::Blueprint,
            }),
            Actor::Function(actor) => Ok(BlueprintTypeTarget {
                blueprint_info: BlueprintInfo {
                    blueprint_id: actor.blueprint_id.clone(),
                    outer_obj_info: OuterObjectInfo::None,
                    features: btreeset!(),
                    generic_substitutions: vec![],
                },
                meta: SchemaValidationMeta::Blueprint,
            }),
            Actor::Method(actor) => {
                let blueprint_info = self.get_blueprint_info(&actor.node_id, actor.module_id)?;
                Ok(BlueprintTypeTarget {
                    blueprint_info,
                    meta: SchemaValidationMeta::ExistingObject {
                        additional_schemas: actor.node_id,
                    },
                })
            }
        }
    }

    fn get_actor_object_id(
        &mut self,
        actor_object_type: ActorObjectType,
    ) -> Result<(NodeId, ObjectModuleId), RuntimeError> {
        let actor = self.current_actor();
        let object_id = actor
            .get_object_id()
            .ok_or_else(|| RuntimeError::SystemError(SystemError::NotAnObject))?;

        let object_id = match actor_object_type {
            ActorObjectType::OuterObject => {
                let module_id = object_id.1;

                match module_id {
                    ObjectModuleId::Main => {
                        let node_id = object_id.0;
                        let address = self.get_outer_object(&node_id)?;

                        (address.into_node_id(), ObjectModuleId::Main)
                    }
                    _ => {
                        return Err(RuntimeError::SystemError(
                            SystemError::OuterObjectDoesNotExist,
                        ));
                    }
                }
            }
            ActorObjectType::SELF => object_id,
        };

        Ok(object_id)
    }

    fn get_actor_collection_partition_info(
        &mut self,
        actor_object_type: ActorObjectType,
        collection_index: u8,
        expected_type: &BlueprintPartitionType,
    ) -> Result<(NodeId, BlueprintInfo, PartitionNumber), RuntimeError> {
        let (node_id, module_id) = self.get_actor_object_id(actor_object_type)?;
        let blueprint_info = self.get_blueprint_info(&node_id, module_id)?;
        let blueprint_interface =
            self.get_blueprint_default_interface(blueprint_info.blueprint_id.clone())?;

        let partition_num = {
            let (partition_description, partition_type) = blueprint_interface
                .state
                .get_partition(collection_index)
                .ok_or_else(|| {
                    RuntimeError::SystemError(SystemError::CollectionIndexDoesNotExist(
                        blueprint_info.blueprint_id.clone(),
                        collection_index,
                    ))
                })?;

            if !partition_type.eq(expected_type) {
                // TODO: Implement different error
                return Err(RuntimeError::SystemError(
                    SystemError::CollectionIndexDoesNotExist(
                        blueprint_info.blueprint_id.clone(),
                        collection_index,
                    ),
                ));
            }

            match partition_description {
                PartitionDescription::Physical(partition_num) => partition_num,
                PartitionDescription::Logical(offset) => module_id
                    .base_partition_num()
                    .at_offset(offset)
                    .expect("Module number overflow"),
            }
        };

        Ok((node_id, blueprint_info, partition_num))
    }

    fn get_actor_info(
        &mut self,
        actor_object_type: ActorObjectType,
    ) -> Result<(NodeId, ObjectModuleId, BlueprintInterface, BlueprintInfo), RuntimeError> {
        let (node_id, module_id) = self.get_actor_object_id(actor_object_type)?;
        let blueprint_info = self.get_blueprint_info(&node_id, module_id)?;
        let blueprint_interface =
            self.get_blueprint_default_interface(blueprint_info.blueprint_id.clone())?;

        Ok((node_id, module_id, blueprint_interface, blueprint_info))
    }

    fn get_actor_field_info(
        &mut self,
        actor_object_type: ActorObjectType,
        field_index: u8,
    ) -> Result<(NodeId, BlueprintInfo, PartitionNumber, FieldTransience), RuntimeError> {
        let (node_id, module_id, interface, info) = self.get_actor_info(actor_object_type)?;

        let (partition_description, field_schema) =
            interface.state.field(field_index).ok_or_else(|| {
                RuntimeError::SystemError(SystemError::FieldDoesNotExist(
                    info.blueprint_id.clone(),
                    field_index,
                ))
            })?;

        match field_schema.condition {
            Condition::IfFeature(feature) => {
                if !self.is_feature_enabled(&node_id, module_id, feature.as_str())? {
                    return Err(RuntimeError::SystemError(SystemError::FieldDoesNotExist(
                        info.blueprint_id.clone(),
                        field_index,
                    )));
                }
            }
            Condition::IfOuterFeature(feature) => {
                let parent_id = match info.outer_obj_info {
                    OuterObjectInfo::Some { outer_object } => outer_object.into_node_id(),
                    OuterObjectInfo::None => {
                        panic!("Outer object should not have IfOuterFeature.")
                    }
                };
                let parent_module = ObjectModuleId::Main;

                if !self.is_feature_enabled(&parent_id, parent_module, feature.as_str())? {
                    return Err(RuntimeError::SystemError(SystemError::FieldDoesNotExist(
                        info.blueprint_id.clone(),
                        field_index,
                    )));
                }
            }
            Condition::Always => {}
        }

        let partition_num = match partition_description {
            PartitionDescription::Physical(partition_num) => partition_num,
            PartitionDescription::Logical(offset) => module_id
                .base_partition_num()
                .at_offset(offset)
                .expect("Module number overflow"),
        };

        Ok((node_id, info, partition_num, field_schema.transience))
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

        Ok(self.get_object_info(node_id)?.blueprint_info.blueprint_id)
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

        // For simplicity, a rule is enforced at system layer: only the package can globalize a node
        // In the future, we may consider allowing customization at blueprint level.
        let actor = self.current_actor();
        if Some(reserved_blueprint_id.package_address) != actor.package_address() {
            return Err(RuntimeError::SystemError(
                SystemError::InvalidGlobalizeAccess(Box::new(InvalidGlobalizeAccess {
                    package_address: reserved_blueprint_id.package_address,
                    blueprint_name: reserved_blueprint_id.blueprint_name,
                    actor_package: actor.package_address(),
                })),
            ));
        }

        // Check for required modules
        if !modules.contains_key(&ObjectModuleId::RoleAssignment) {
            return Err(RuntimeError::SystemError(SystemError::MissingModule(
                ObjectModuleId::RoleAssignment,
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
        let mut object_info = self.get_object_info(&node_id)?;

        // Verify can globalize with address
        {
            if object_info.global {
                return Err(RuntimeError::SystemError(SystemError::CannotGlobalize(
                    CannotGlobalizeError::AlreadyGlobalized,
                )));
            }
            if !object_info
                .blueprint_info
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
            object_info.global = true;
            for module_id in modules.keys() {
                object_info
                    .module_versions
                    .insert(module_id.clone(), BlueprintVersion::default());
            }
        }

        let num_main_partitions = {
            let interface = self
                .get_blueprint_default_interface(object_info.blueprint_info.blueprint_id.clone())?;
            interface.state.num_logical_partitions()
        };

        // Create a global node
        self.kernel_create_node(
            global_address.into(),
            btreemap!(
                TYPE_INFO_FIELD_PARTITION => type_info_partition(TypeInfoSubstate::Object(object_info))
            ),
        )?;

        self.kernel_move_partition(
            &node_id,
            SCHEMAS_PARTITION,
            global_address.as_node_id(),
            SCHEMAS_PARTITION,
        )?;

        // Move self modules to the newly created global node, and drop
        for offset in 0u8..num_main_partitions {
            let partition_number = MAIN_BASE_PARTITION
                .at_offset(PartitionOffset(offset))
                .unwrap();
            self.kernel_move_partition(
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
                ObjectModuleId::RoleAssignment
                | ObjectModuleId::Metadata
                | ObjectModuleId::Royalty => {
                    let blueprint_id = self.get_object_info(&node_id)?.blueprint_info.blueprint_id;
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
                    let num_logical_partitions = interface.state.num_logical_partitions();

                    let module_base_partition = module_id.base_partition_num();
                    for offset in 0u8..num_logical_partitions {
                        let src = MAIN_BASE_PARTITION
                            .at_offset(PartitionOffset(offset))
                            .unwrap();
                        let dest = module_base_partition
                            .at_offset(PartitionOffset(offset))
                            .unwrap();

                        self.kernel_move_partition(
                            &node_id,
                            src,
                            global_address.as_node_id(),
                            dest,
                        )?;
                    }

                    self.kernel_drop_node(&node_id)?;
                }
            }
        }

        Ok(global_address)
    }

    pub fn current_actor(&mut self) -> Actor {
        self.api
            .kernel_get_system_state()
            .current_call_frame
            .clone()
    }

    pub fn get_object_info(&mut self, node_id: &NodeId) -> Result<ObjectInfo, RuntimeError> {
        let type_info = TypeInfoBlueprint::get_type(&node_id, self.api)?;
        let object_info = match type_info {
            TypeInfoSubstate::Object(info) => info,
            _ => return Err(RuntimeError::SystemError(SystemError::NotAnObject)),
        };

        Ok(object_info)
    }

    pub fn is_feature_enabled(
        &mut self,
        node_id: &NodeId,
        module_id: ObjectModuleId,
        feature: &str,
    ) -> Result<bool, RuntimeError> {
        match module_id {
            ObjectModuleId::Main => {
                let object_info = self.get_object_info(node_id)?;
                let enabled = object_info.blueprint_info.features.contains(feature);
                Ok(enabled)
            }
            _ => Ok(false),
        }
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
        let data = self.api.kernel_get_lock_data(handle)?;
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
        let data = self.api.kernel_get_lock_data(handle)?;

        match data {
            SystemLockData::Field(FieldLockData::Write {
                target,
                field_index,
            }) => {
                self.validate_blueprint_payload(
                    &target,
                    BlueprintPayloadIdentifier::Field(field_index),
                    &buffer,
                )?;
            }
            _ => {
                return Err(RuntimeError::SystemError(SystemError::NotAFieldWriteHandle));
            }
        };

        let value: ScryptoValue =
            scrypto_decode(&buffer).expect("Should be valid due to payload check");

        let substate = IndexedScryptoValue::from_typed(&FieldSubstate::new_field(value));

        self.api.kernel_write_substate(handle, substate)?;

        Ok(())
    }

    // Costing through kernel
    #[trace_resources]
    fn field_lock(&mut self, handle: FieldHandle) -> Result<(), RuntimeError> {
        let data = self.api.kernel_get_lock_data(handle)?;

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
        let data = self.api.kernel_get_lock_data(handle)?;
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
        generic_args: GenericArgs,
        fields: Vec<FieldValue>,
        kv_entries: BTreeMap<u8, BTreeMap<Vec<u8>, KVEntry>>,
    ) -> Result<NodeId, RuntimeError> {
        let actor = self.current_actor();
        let package_address = actor
            .blueprint_id()
            .map(|b| b.package_address)
            .ok_or(RuntimeError::SystemError(SystemError::NoPackageAddress))?;
        let blueprint_id = BlueprintId::new(&package_address, blueprint_ident);
        let instance_context = actor.instance_context();

        self.new_object_internal(
            &blueprint_id,
            features,
            instance_context,
            generic_args,
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
    fn globalize_with_address_and_create_inner_object_and_emit_event(
        &mut self,
        modules: BTreeMap<ObjectModuleId, NodeId>,
        address_reservation: GlobalAddressReservation,
        inner_object_blueprint: &str,
        inner_object_fields: Vec<FieldValue>,
        event_name: String,
        event_data: Vec<u8>,
    ) -> Result<(GlobalAddress, NodeId), RuntimeError> {
        let actor_blueprint = self.resolve_blueprint_from_modules(&modules)?;

        let global_address = self.globalize_with_address_internal(modules, address_reservation)?;

        let blueprint_id =
            BlueprintId::new(&actor_blueprint.package_address, inner_object_blueprint);

        let inner_object = self.new_object_internal(
            &blueprint_id,
            vec![],
            Some(InstanceContext {
                outer_object: global_address,
            }),
            GenericArgs::default(),
            inner_object_fields,
            btreemap!(),
        )?;

        self.emit_event_internal(
            EmitterActor::AsObject(global_address.as_node_id().clone(), ObjectModuleId::Main),
            event_name,
            event_data,
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
        // Key Value Stores do not have methods so we remove that possibility here
        let object_info = self.get_object_info(&receiver)?;
        if !object_info.module_versions.contains_key(&module_id) {
            return Err(RuntimeError::SystemError(
                SystemError::ObjectModuleDoesNotExist(module_id),
            ));
        }

        let args = IndexedScryptoValue::from_vec(args).map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        let auth_actor_info = SystemModuleMixer::on_call_method(
            self,
            receiver,
            module_id,
            direct_access,
            method_name,
            &args,
        )?;

        let rtn = self
            .api
            .kernel_invoke(Box::new(KernelInvocation {
                call_frame_data: Actor::Method(MethodActor {
                    direct_access,
                    node_id: receiver.clone(),
                    module_id,
                    ident: method_name.to_string(),

                    auth_zone: auth_actor_info.clone(),
                    object_info,
                }),
                args,
            }))
            .map(|v| v.into())?;

        SystemModuleMixer::on_call_method_finish(self, auth_actor_info)?;

        Ok(rtn)
    }

    // Costing through kernel
    #[trace_resources]
    fn get_blueprint_id(&mut self, node_id: &NodeId) -> Result<BlueprintId, RuntimeError> {
        let blueprint_id = self.get_object_info(node_id)?.blueprint_info.blueprint_id;
        Ok(blueprint_id)
    }

    // Costing through kernel
    #[trace_resources]
    fn get_outer_object(&mut self, node_id: &NodeId) -> Result<GlobalAddress, RuntimeError> {
        match self.get_object_info(node_id)?.try_get_outer_object() {
            None => Err(RuntimeError::SystemError(
                SystemError::OuterObjectDoesNotExist,
            )),
            Some(address) => Ok(address),
        }
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
        // For simplicity, a rule is enforced at system layer: only the package can drop a node
        // In the future, we may consider allowing customization at blueprint level.
        let info = self.get_object_info(node_id)?;
        let actor = self.current_actor();
        if Some(info.blueprint_info.blueprint_id.package_address) != actor.package_address() {
            return Err(RuntimeError::SystemError(SystemError::InvalidDropAccess(
                Box::new(InvalidDropAccess {
                    node_id: node_id.clone(),
                    package_address: info.blueprint_info.blueprint_id.package_address,
                    blueprint_name: info.blueprint_info.blueprint_id.blueprint_name,
                    actor_package: actor.package_address(),
                }),
            )));
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
        let data = self.api.kernel_get_lock_data(handle)?;
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
    fn key_value_entry_lock(&mut self, handle: KeyValueEntryHandle) -> Result<(), RuntimeError> {
        let data = self.api.kernel_get_lock_data(handle)?;
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
        let data = self.api.kernel_get_lock_data(handle)?;

        match data {
            SystemLockData::KeyValueEntry(KeyValueEntryLockData::BlueprintWrite {
                collection_index,
                target,
            }) => {
                self.validate_blueprint_payload(
                    &target,
                    BlueprintPayloadIdentifier::KeyValueEntry(collection_index, KeyOrValue::Value),
                    &buffer,
                )?;
            }
            SystemLockData::KeyValueEntry(KeyValueEntryLockData::Write {
                kv_store_validation_target,
            }) => {
                self.validate_kv_store_payload(
                    &kv_store_validation_target,
                    KeyOrValue::Value,
                    &buffer,
                )?;
            }
            _ => {
                return Err(RuntimeError::SystemError(
                    SystemError::NotAKeyValueWriteLock,
                ));
            }
        }

        let substate =
            IndexedScryptoValue::from_slice(&buffer).expect("Should be valid due to payload check");

        let value = substate.as_scrypto_value().clone();
        let kv_entry = KeyValueEntrySubstate::entry(value);
        let indexed = IndexedScryptoValue::from_typed(&kv_entry);

        self.api.kernel_write_substate(handle, indexed)?;

        Ok(())
    }

    // Costing through kernel
    fn key_value_entry_close(&mut self, handle: KeyValueEntryHandle) -> Result<(), RuntimeError> {
        let data = self.api.kernel_get_lock_data(handle)?;
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
    fn key_value_store_new(
        &mut self,
        generic_args: KeyValueStoreGenericArgs,
    ) -> Result<NodeId, RuntimeError> {
        let mut additional_schemas = index_map_new();
        if let Some(schema) = generic_args.additional_schema {
            validate_schema(&schema)
                .map_err(|_| RuntimeError::SystemError(SystemError::InvalidGenericArgs))?;
            let schema_hash = schema.generate_schema_hash();
            additional_schemas.insert(schema_hash, schema);
        }

        self.validate_kv_store_generic_args(
            &additional_schemas,
            &generic_args.key_type,
            &generic_args.value_type,
        )
        .map_err(|e| RuntimeError::SystemError(SystemError::TypeCheckError(e)))?;

        let schema_partition = additional_schemas
            .into_iter()
            .map(|(schema_hash, schema)| {
                let key = SubstateKey::Map(scrypto_encode(&schema_hash).unwrap());
                let substate = KeyValueEntrySubstate::locked_entry(schema);
                let value = IndexedScryptoValue::from_typed(&substate);
                (key, value)
            })
            .collect();

        let generic_substitutions = KeyValueStoreGenericSubstitutions {
            key_generic_substitutions: generic_args.key_type,
            value_generic_substitutions: generic_args.value_type,
            allow_ownership: generic_args.allow_ownership,
        };

        let node_id = self
            .api
            .kernel_allocate_node_id(IDAllocation::KeyValueStore.entity_type())?;

        self.api.kernel_create_node(
            node_id,
            btreemap!(
                MAIN_BASE_PARTITION => btreemap!(),
                TYPE_INFO_FIELD_PARTITION => type_info_partition(
                    TypeInfoSubstate::KeyValueStore(KeyValueStoreInfo {
                        generic_substitutions,
                    })
                ),
                SCHEMAS_PARTITION => schema_partition,
            ),
        )?;

        Ok(node_id)
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

        let target = KVStoreValidationTarget {
            kv_store_type: info.generic_substitutions,
            meta: *node_id,
        };

        self.validate_kv_store_payload(&target, KeyOrValue::Key, &key)?;

        let lock_data = if flags.contains(LockFlags::MUTABLE) {
            SystemLockData::KeyValueEntry(KeyValueEntryLockData::Write {
                kv_store_validation_target: target,
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

        let (node_id, info, partition_num) = self.get_actor_collection_partition_info(
            actor_object_type,
            collection_index,
            &BlueprintPartitionType::IndexCollection,
        )?;

        let target = BlueprintTypeTarget {
            blueprint_info: info,
            meta: SchemaValidationMeta::ExistingObject {
                additional_schemas: node_id,
            },
        };

        self.validate_blueprint_payload(
            &target,
            BlueprintPayloadIdentifier::IndexEntry(collection_index, KeyOrValue::Key),
            &key,
        )?;

        self.validate_blueprint_payload(
            &target,
            BlueprintPayloadIdentifier::IndexEntry(collection_index, KeyOrValue::Value),
            &buffer,
        )?;

        let value = IndexedScryptoValue::from_vec(buffer)
            .map_err(|e| RuntimeError::SystemError(SystemError::InvalidScryptoValue(e)))?;

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

        let (node_id, _info, partition_num) = self.get_actor_collection_partition_info(
            actor_object_type,
            collection_index,
            &BlueprintPartitionType::IndexCollection,
        )?;

        let rtn = self
            .api
            .kernel_remove_substate(&node_id, partition_num, &SubstateKey::Map(key))?
            .map(|v| v.into());

        Ok(rtn)
    }

    // Costing through kernel
    fn actor_index_scan_keys(
        &mut self,
        object_handle: ObjectHandle,
        collection_index: CollectionIndex,
        limit: u32,
    ) -> Result<Vec<Vec<u8>>, RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, _info, partition_num) = self.get_actor_collection_partition_info(
            actor_object_type,
            collection_index,
            &BlueprintPartitionType::IndexCollection,
        )?;

        let substates = self
            .api
            .kernel_scan_keys::<MapKey>(&node_id, partition_num, limit)?
            .into_iter()
            .map(|key| key.into_map())
            .collect();

        Ok(substates)
    }

    // Costing through kernel
    fn actor_index_drain(
        &mut self,
        object_handle: ObjectHandle,
        collection_index: CollectionIndex,
        limit: u32,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, _info, partition_num) = self.get_actor_collection_partition_info(
            actor_object_type,
            collection_index,
            &BlueprintPartitionType::IndexCollection,
        )?;

        let substates = self
            .api
            .kernel_drain_substates::<MapKey>(&node_id, partition_num, limit)?
            .into_iter()
            .map(|(key, value)| (key.into_map(), value.into()))
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

        let (node_id, info, partition_num) = self.get_actor_collection_partition_info(
            actor_object_type,
            collection_index,
            &BlueprintPartitionType::SortedIndexCollection,
        )?;

        let target = BlueprintTypeTarget {
            blueprint_info: info,
            meta: SchemaValidationMeta::ExistingObject {
                additional_schemas: node_id,
            },
        };

        self.validate_blueprint_payload(
            &target,
            BlueprintPayloadIdentifier::SortedIndexEntry(collection_index, KeyOrValue::Key),
            &sorted_key.1,
        )?;

        self.validate_blueprint_payload(
            &target,
            BlueprintPayloadIdentifier::SortedIndexEntry(collection_index, KeyOrValue::Value),
            &buffer,
        )?;

        let value = IndexedScryptoValue::from_vec(buffer)
            .map_err(|e| RuntimeError::SystemError(SystemError::InvalidScryptoValue(e)))?;

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

        let (node_id, _info, partition_num) = self.get_actor_collection_partition_info(
            actor_object_type,
            collection_index,
            &BlueprintPartitionType::SortedIndexCollection,
        )?;

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
        limit: u32,
    ) -> Result<Vec<(SortedKey, Vec<u8>)>, RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, _info, partition_num) = self.get_actor_collection_partition_info(
            actor_object_type,
            collection_index,
            &BlueprintPartitionType::SortedIndexCollection,
        )?;

        let substates = self
            .api
            .kernel_scan_sorted_substates(&node_id, partition_num, limit)?
            .into_iter()
            .map(|(key, value)| (key, value.into()))
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
        let args = IndexedScryptoValue::from_vec(args).map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;
        let blueprint_id = BlueprintId::new(&package_address, blueprint_name);
        let auth_zone = SystemModuleMixer::on_call_function(self, &blueprint_id, function_name)?;

        let rtn = self
            .api
            .kernel_invoke(Box::new(KernelInvocation {
                call_frame_data: Actor::Function(FunctionActor {
                    blueprint_id,
                    ident: function_name.to_string(),
                    auth_zone: auth_zone.clone(),
                }),
                args,
            }))
            .map(|v| v.into())?;

        SystemModuleMixer::on_call_function_finish(self, auth_zone)?;

        Ok(rtn)
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
                } => ExecutionCostingEntry::RunNativeCode {
                    package_address,
                    export_name,
                    input_size,
                },
                ClientCostingEntry::RunWasmCode {
                    package_address,
                    export_name,
                    wasm_execution_units,
                } => ExecutionCostingEntry::RunWasmCode {
                    package_address,
                    export_name,
                    wasm_execution_units,
                },
                ClientCostingEntry::PrepareWasmCode { size } => {
                    ExecutionCostingEntry::PrepareWasmCode { size }
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
            .apply_execution_cost(ExecutionCostingEntry::LockFee)?;

        self.api
            .kernel_get_system()
            .modules
            .credit_cost_units(vault_id, locked_fee, contingent)
    }

    fn execution_cost_unit_limit(&mut self) -> Result<u32, RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(ExecutionCostingEntry::QueryFeeReserve)?;

        if let Some(fee_reserve) = self.api.kernel_get_system().modules.fee_reserve() {
            Ok(fee_reserve.execution_cost_unit_limit())
        } else {
            Err(RuntimeError::SystemError(
                SystemError::CostingModuleNotEnabled,
            ))
        }
    }

    fn execution_cost_unit_price(&mut self) -> Result<Decimal, RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(ExecutionCostingEntry::QueryFeeReserve)?;

        if let Some(fee_reserve) = self.api.kernel_get_system().modules.fee_reserve() {
            Ok(fee_reserve.execution_cost_unit_price())
        } else {
            Err(RuntimeError::SystemError(
                SystemError::CostingModuleNotEnabled,
            ))
        }
    }

    fn finalization_cost_unit_limit(&mut self) -> Result<u32, RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(ExecutionCostingEntry::QueryFeeReserve)?;

        if let Some(fee_reserve) = self.api.kernel_get_system().modules.fee_reserve() {
            Ok(fee_reserve.finalization_cost_unit_limit())
        } else {
            Err(RuntimeError::SystemError(
                SystemError::CostingModuleNotEnabled,
            ))
        }
    }

    fn finalization_cost_unit_price(&mut self) -> Result<Decimal, RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(ExecutionCostingEntry::QueryFeeReserve)?;

        if let Some(fee_reserve) = self.api.kernel_get_system().modules.fee_reserve() {
            Ok(fee_reserve.finalization_cost_unit_price())
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
            .apply_execution_cost(ExecutionCostingEntry::QueryFeeReserve)?;

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
            .apply_execution_cost(ExecutionCostingEntry::QueryFeeReserve)?;

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
    #[trace_resources]
    fn actor_get_blueprint_id(&mut self) -> Result<BlueprintId, RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(ExecutionCostingEntry::QueryActor)?;

        self.current_actor()
            .blueprint_id()
            .ok_or(RuntimeError::SystemError(SystemError::NoBlueprintId))
    }

    // Costing through kernel
    #[trace_resources]
    fn actor_open_field(
        &mut self,
        object_handle: ObjectHandle,
        field_index: u8,
        flags: LockFlags,
    ) -> Result<SubstateHandle, RuntimeError> {
        let actor_object_type: ActorObjectType = object_handle.try_into()?;

        let (node_id, blueprint_info, partition_num, transient) =
            self.get_actor_field_info(actor_object_type, field_index)?;

        // TODO: Remove
        if flags.contains(LockFlags::UNMODIFIED_BASE) || flags.contains(LockFlags::FORCE_WRITE) {
            if !(blueprint_info.blueprint_id.eq(&BlueprintId::new(
                &RESOURCE_PACKAGE,
                FUNGIBLE_VAULT_BLUEPRINT,
            ))) {
                return Err(RuntimeError::SystemError(SystemError::InvalidLockFlags));
            }
        }

        let lock_data = if flags.contains(LockFlags::MUTABLE) {
            let target = BlueprintTypeTarget {
                blueprint_info,
                meta: SchemaValidationMeta::ExistingObject {
                    additional_schemas: node_id,
                },
            };

            FieldLockData::Write {
                target,
                field_index,
            }
        } else {
            FieldLockData::Read
        };

        let handle = match transient {
            FieldTransience::NotTransient => self.api.kernel_open_substate(
                &node_id,
                partition_num,
                &SubstateKey::Field(field_index),
                flags,
                SystemLockData::Field(lock_data),
            )?,
            FieldTransience::TransientStatic { default_value } => {
                let default_value: ScryptoValue = scrypto_decode(&default_value).unwrap();
                self.api.kernel_mark_substate_as_transient(
                    node_id,
                    partition_num,
                    SubstateKey::Field(field_index),
                )?;
                self.api.kernel_open_substate_with_default(
                    &node_id,
                    partition_num,
                    &SubstateKey::Field(field_index),
                    flags,
                    Some(|| {
                        IndexedScryptoValue::from_typed(&FieldSubstate::new_field(default_value))
                    }),
                    SystemLockData::Field(lock_data),
                )?
            }
        };

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
    fn actor_get_node_id(&mut self) -> Result<NodeId, RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(ExecutionCostingEntry::QueryActor)?;

        let node_id = self
            .current_actor()
            .node_id()
            .ok_or(RuntimeError::SystemError(
                SystemError::ActorNodeIdDoesNotExist,
            ))?;

        Ok(node_id)
    }

    #[trace_resources]
    fn actor_get_global_address(&mut self) -> Result<GlobalAddress, RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(ExecutionCostingEntry::QueryActor)?;

        let actor = self.current_actor();
        if !actor.is_direct_access() {
            if let Some(node_id) = actor.node_id() {
                let visibility = self.kernel_get_node_visibility(&node_id);
                if let ReferenceOrigin::Global(address) =
                    visibility.reference_origin(node_id).unwrap()
                {
                    return Ok(address);
                }
            }
        }

        Err(RuntimeError::SystemError(
            SystemError::GlobalAddressDoesNotExist,
        ))
    }

    #[trace_resources]
    fn actor_get_outer_object(&mut self) -> Result<GlobalAddress, RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(ExecutionCostingEntry::QueryActor)?;

        let (node_id, module_id) = self.get_actor_object_id(ActorObjectType::SELF)?;
        match module_id {
            ObjectModuleId::Main => {
                let info = self.get_object_info(&node_id)?;
                match info.blueprint_info.outer_obj_info {
                    OuterObjectInfo::Some { outer_object } => Ok(outer_object),
                    OuterObjectInfo::None => Err(RuntimeError::SystemError(
                        SystemError::OuterObjectDoesNotExist,
                    )),
                }
            }
            _ => Err(RuntimeError::SystemError(
                SystemError::ModulesDontHaveOuterObjects,
            )),
        }
    }

    // Costing through kernel
    #[trace_resources]
    fn actor_call_module(
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
            .apply_execution_cost(ExecutionCostingEntry::QueryActor)?;

        let actor_object_type: ActorObjectType = object_handle.try_into()?;
        let (node_id, module_id) = self.get_actor_object_id(actor_object_type)?;
        self.is_feature_enabled(&node_id, module_id, feature)
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

        let (node_id, info, partition_num) = self.get_actor_collection_partition_info(
            actor_object_type,
            collection_index,
            &BlueprintPartitionType::KeyValueCollection,
        )?;

        let target = BlueprintTypeTarget {
            blueprint_info: info,
            meta: SchemaValidationMeta::ExistingObject {
                additional_schemas: node_id,
            },
        };

        self.validate_blueprint_payload(
            &target,
            BlueprintPayloadIdentifier::KeyValueEntry(collection_index, KeyOrValue::Key),
            &key,
        )?;

        let lock_data = if flags.contains(LockFlags::MUTABLE) {
            KeyValueEntryLockData::BlueprintWrite {
                collection_index,
                target,
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
            .apply_execution_cost(ExecutionCostingEntry::QueryAuthZone)?;

        if let Some(auth_zone_id) = self.current_actor().self_auth_zone() {
            Ok(auth_zone_id.into())
        } else {
            Err(RuntimeError::SystemError(SystemError::AuthModuleNotEnabled))
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
        self.emit_event_internal(EmitterActor::CurrentActor, event_name, event_data)
    }

    #[trace_resources]
    fn emit_log(&mut self, level: Level, message: String) -> Result<(), RuntimeError> {
        self.api.kernel_get_system().modules.apply_execution_cost(
            ExecutionCostingEntry::EmitLog {
                size: message.len(),
            },
        )?;

        self.api
            .kernel_get_system()
            .modules
            .add_log(level, message)?;

        Ok(())
    }

    fn panic(&mut self, message: String) -> Result<(), RuntimeError> {
        self.api.kernel_get_system().modules.apply_execution_cost(
            ExecutionCostingEntry::Panic {
                size: message.len(),
            },
        )?;

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
            .apply_execution_cost(ExecutionCostingEntry::QueryTransactionHash)?;

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
            .apply_execution_cost(ExecutionCostingEntry::GenerateRuid)?;

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
    fn kernel_pin_node(&mut self, node_id: NodeId) -> Result<(), RuntimeError> {
        self.api.kernel_pin_node(node_id)
    }

    fn kernel_mark_substate_as_transient(
        &mut self,
        node_id: NodeId,
        partition_num: PartitionNumber,
        key: SubstateKey,
    ) -> Result<(), RuntimeError> {
        self.api
            .kernel_mark_substate_as_transient(node_id, partition_num, key)
    }

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

    fn kernel_move_partition(
        &mut self,
        src_node_id: &NodeId,
        src_partition_number: PartitionNumber,
        dest_node_id: &NodeId,
        dest_partition_number: PartitionNumber,
    ) -> Result<(), RuntimeError> {
        self.api.kernel_move_partition(
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
    fn kernel_open_substate_with_default<F: FnOnce() -> IndexedScryptoValue>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        default: Option<F>,
        data: SystemLockData,
    ) -> Result<SubstateHandle, RuntimeError> {
        self.api.kernel_open_substate_with_default(
            node_id,
            partition_num,
            substate_key,
            flags,
            default,
            data,
        )
    }

    fn kernel_get_lock_data(
        &mut self,
        lock_handle: SubstateHandle,
    ) -> Result<SystemLockData, RuntimeError> {
        self.api.kernel_get_lock_data(lock_handle)
    }

    fn kernel_close_substate(&mut self, lock_handle: SubstateHandle) -> Result<(), RuntimeError> {
        self.api.kernel_close_substate(lock_handle)
    }

    fn kernel_read_substate(
        &mut self,
        lock_handle: SubstateHandle,
    ) -> Result<&IndexedScryptoValue, RuntimeError> {
        self.api.kernel_read_substate(lock_handle)
    }

    fn kernel_write_substate(
        &mut self,
        lock_handle: SubstateHandle,
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
        limit: u32,
    ) -> Result<Vec<(SortedKey, IndexedScryptoValue)>, RuntimeError> {
        self.api
            .kernel_scan_sorted_substates(node_id, partition_num, limit)
    }

    fn kernel_scan_keys<K: SubstateKeyContent + 'static>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        limit: u32,
    ) -> Result<Vec<SubstateKey>, RuntimeError> {
        self.api
            .kernel_scan_keys::<K>(node_id, partition_num, limit)
    }

    fn kernel_drain_substates<K: SubstateKeyContent + 'static>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        limit: u32,
    ) -> Result<Vec<(SubstateKey, IndexedScryptoValue)>, RuntimeError> {
        self.api
            .kernel_drain_substates::<K>(node_id, partition_num, limit)
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
