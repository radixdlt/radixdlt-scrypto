use super::id_allocation::IDAllocation;
use super::system_modules::costing::ExecutionCostingEntry;
use crate::blueprints::package::PackageBlueprintVersionDefinitionEntrySubstate;
use crate::blueprints::resource::fungible_vault::LockFeeEvent;
use crate::errors::*;
use crate::errors::{EventError, SystemUpstreamError};
use crate::internal_prelude::*;
use crate::kernel::call_frame::{NodeVisibility, ReferenceOrigin};
use crate::kernel::kernel_api::*;
use crate::system::actor::{Actor, FunctionActor, InstanceContext, MethodActor, MethodType};
use crate::system::node_init::type_info_partition;
use crate::system::system_callback::*;
use crate::system::system_modules::transaction_runtime::Event;
use crate::system::system_modules::{EnabledModules, SystemModuleMixer};
use crate::system::system_substates::{KeyValueEntrySubstate, LockStatus};
use crate::system::system_type_checker::{
    BlueprintTypeTarget, KVStoreTypeTarget, SchemaValidationMeta, SystemMapper,
};
use crate::system::type_info::{TypeInfoBlueprint, TypeInfoSubstate};
use crate::track::interface::NodeSubstates;
use radix_blueprint_schema_init::{Condition, KeyValueStoreGenericSubstitutions};
#[cfg(not(feature = "alloc"))]
use radix_common_derive::*;
use radix_engine_interface::api::actor_api::EventFlags;
use radix_engine_interface::api::actor_index_api::SystemActorIndexApi;
use radix_engine_interface::api::field_api::{FieldHandle, LockFlags};
use radix_engine_interface::api::key_value_entry_api::{
    KeyValueEntryHandle, SystemKeyValueEntryApi,
};
use radix_engine_interface::api::key_value_store_api::{
    KeyValueStoreDataSchema, SystemKeyValueStoreApi,
};
use radix_engine_interface::api::object_api::ModuleId;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_profiling_derive::trace_resources;
use radix_substate_store_interface::db_key_mapper::SubstateKeyContent;

enum ActorStateRef {
    SELF,
    OuterObject,
}

impl TryFrom<ActorStateHandle> for ActorStateRef {
    type Error = RuntimeError;
    fn try_from(value: ActorStateHandle) -> Result<Self, Self::Error> {
        match value {
            ACTOR_STATE_SELF => Ok(ActorStateRef::SELF),
            ACTOR_STATE_OUTER_OBJECT => Ok(ActorStateRef::OuterObject),
            _ => Err(RuntimeError::SystemError(
                SystemError::InvalidActorStateHandle,
            )),
        }
    }
}

enum ActorObjectRef {
    SELF,
    Outer,
    Global,
    AuthZone,
}

impl TryFrom<ActorRefHandle> for ActorObjectRef {
    type Error = RuntimeError;
    fn try_from(value: ActorStateHandle) -> Result<Self, Self::Error> {
        match value {
            ACTOR_REF_SELF => Ok(ActorObjectRef::SELF),
            ACTOR_REF_OUTER => Ok(ActorObjectRef::Outer),
            ACTOR_REF_GLOBAL => Ok(ActorObjectRef::Global),
            ACTOR_REF_AUTH_ZONE => Ok(ActorObjectRef::AuthZone),
            _ => Err(RuntimeError::SystemError(
                SystemError::InvalidActorRefHandle,
            )),
        }
    }
}

enum EmitterActor {
    CurrentActor,
    AsObject(NodeId, Option<AttachedModuleId>),
}

/// A wrapper offering a comprehensive system api to callers. It is built on top
/// of a [`SystemBasedKernelApi`], and you are free to access it.
/// You can also construct a [`SystemService`] from the api with `api.system_service()`.
///
/// Like [`SystemModuleApiImpl`], we use a wrapper type rather than implementing this functionality
/// directly on a `SystemBasedKernelApi` for a few reasons:
/// * Trait coherence - the System traits aren't defined in this crate, so it prevents us
///   from implementing them on any type implementing `SystemBasedKernelApi`.
/// * Separation of APIs - we avoid exposing the methods of a [`SystemServiceApi`] directly
///   if someone happens to have a [`SystemBasedKernelApi`], which prevents some
///   possible confusion.
pub struct SystemService<'a, Y: SystemBasedKernelApi + ?Sized> {
    api: &'a mut Y,
}

impl<'a, Y: SystemBasedKernelApi + ?Sized> SystemService<'a, Y> {
    pub fn new(api: &'a mut Y) -> Self {
        Self { api }
    }

    pub fn api(&mut self) -> &mut Y {
        self.api
    }

    pub fn system(&mut self) -> &mut System<Y::SystemCallback> {
        self.api.kernel_get_system()
    }
}

#[cfg_attr(
    feature = "std",
    catch_unwind(crate::utils::catch_unwind_system_panic_transformer)
)]
impl<'a, Y: SystemBasedKernelApi> SystemService<'a, Y> {
    fn validate_new_object(
        &mut self,
        blueprint_id: &BlueprintId,
        blueprint_interface: &BlueprintInterface,
        outer_obj_info: OuterObjectInfo,
        features: IndexSet<String>,
        outer_object_features: &IndexSet<String>,
        generic_args: GenericArgs,
        fields: IndexMap<u8, FieldValue>,
        mut kv_entries: IndexMap<u8, IndexMap<Vec<u8>, KVEntry>>,
    ) -> Result<(BlueprintInfo, NodeSubstates), RuntimeError> {
        // Validate generic arguments
        let (generic_substitutions, additional_schemas) = {
            let mut additional_schemas = index_map_new();

            if let Some(schema) = generic_args.additional_schema {
                validate_schema(schema.v1())
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
            blueprint_id: blueprint_id.clone(),
            blueprint_version: BlueprintVersion::default(),
            outer_obj_info,
            features: features.clone(),
            generic_substitutions: generic_substitutions.clone(),
        };

        let validation_target = BlueprintTypeTarget {
            blueprint_info,
            meta: SchemaValidationMeta::NewObject {
                additional_schemas: additional_schemas.clone().into_iter().collect(),
            },
        };

        // Fields
        {
            let expected_num_fields = blueprint_interface.state.num_fields();
            for field_index in fields.keys() {
                let field_index: usize = (*field_index) as usize;
                if field_index >= expected_num_fields {
                    return Err(RuntimeError::SystemError(SystemError::CreateObjectError(
                        Box::new(CreateObjectError::InvalidFieldIndex(
                            blueprint_id.clone(),
                            field_index as u8,
                        )),
                    )));
                }
            }

            if let Some((_partition, field_schemas)) = &blueprint_interface.state.fields {
                for (i, field) in field_schemas.iter().enumerate() {
                    let index = i as u8;

                    let maybe_field = fields.get(&index);

                    let field_value = match &field.condition {
                        Condition::IfFeature(feature) => {
                            match (features.contains(feature), maybe_field) {
                                (false, Some(..)) => {
                                    return Err(RuntimeError::SystemError(
                                        SystemError::CreateObjectError(Box::new(
                                            CreateObjectError::InvalidFieldDueToFeature(
                                                blueprint_id.clone(),
                                                index,
                                            ),
                                        )),
                                    ));
                                }
                                (true, None) => {
                                    return Err(RuntimeError::SystemError(
                                        SystemError::CreateObjectError(Box::new(
                                            CreateObjectError::MissingField(
                                                blueprint_id.clone(),
                                                index,
                                            ),
                                        )),
                                    ));
                                }
                                (false, None) => continue,
                                (true, Some(field_value)) => field_value,
                            }
                        }
                        Condition::IfOuterFeature(feature) => {
                            match (outer_object_features.contains(feature), maybe_field) {
                                (false, Some(..)) => {
                                    return Err(RuntimeError::SystemError(
                                        SystemError::CreateObjectError(Box::new(
                                            CreateObjectError::InvalidFieldDueToFeature(
                                                blueprint_id.clone(),
                                                index,
                                            ),
                                        )),
                                    ));
                                }
                                (true, None) => {
                                    return Err(RuntimeError::SystemError(
                                        SystemError::CreateObjectError(Box::new(
                                            CreateObjectError::MissingField(
                                                blueprint_id.clone(),
                                                index,
                                            ),
                                        )),
                                    ));
                                }
                                (false, None) => continue,
                                (true, Some(field_value)) => field_value,
                            }
                        }
                        Condition::Always => match maybe_field {
                            None => {
                                return Err(RuntimeError::SystemError(
                                    SystemError::CreateObjectError(Box::new(
                                        CreateObjectError::MissingField(
                                            blueprint_id.clone(),
                                            index,
                                        ),
                                    )),
                                ));
                            }
                            Some(field_value) => field_value,
                        },
                    };

                    self.validate_blueprint_payload(
                        &validation_target,
                        BlueprintPayloadIdentifier::Field(i as u8),
                        &field_value.value,
                    )?;
                }
            }
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
                    kv_entries.insert(index, index_map_new());
                }
            }
        }

        let mut node_substates = SystemMapper::system_struct_to_node_substates(
            &blueprint_interface.state,
            (fields, kv_entries),
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

    pub fn get_blueprint_default_definition(
        &mut self,
        blueprint_id: BlueprintId,
    ) -> Result<Rc<BlueprintDefinition>, RuntimeError> {
        let bp_version_key = BlueprintVersionKey::new_default(blueprint_id.blueprint_name);
        Ok(self.load_blueprint_definition(blueprint_id.package_address, &bp_version_key)?)
    }

    pub fn load_blueprint_definition(
        &mut self,
        package_address: PackageAddress,
        bp_version_key: &BlueprintVersionKey,
    ) -> Result<Rc<BlueprintDefinition>, RuntimeError> {
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

        let substate: PackageBlueprintVersionDefinitionEntrySubstate =
            self.api.kernel_read_substate(handle)?.as_typed().unwrap();
        self.api.kernel_close_substate(handle)?;

        let definition = Rc::new(match substate.into_value() {
            Some(definition) => definition.fully_update_and_into_latest_version(),
            None => {
                return Err(RuntimeError::SystemError(
                    SystemError::BlueprintDoesNotExist(canonical_bp_id),
                ))
            }
        });

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
        fields: IndexMap<u8, FieldValue>,
        kv_entries: IndexMap<u8, IndexMap<Vec<u8>, KVEntry>>,
    ) -> Result<NodeId, RuntimeError> {
        let blueprint_definition = self.get_blueprint_default_definition(blueprint_id.clone())?;
        let blueprint_type = blueprint_definition.interface.blueprint_type.clone();

        let object_features: IndexSet<String> =
            features.into_iter().map(|s| s.to_string()).collect();
        // Validate features
        for feature in &object_features {
            if !blueprint_definition.interface.feature_set.contains(feature) {
                return Err(RuntimeError::SystemError(SystemError::InvalidFeature(
                    feature.to_string(),
                )));
            }
        }

        let (outer_obj_info, outer_object_features) =
            if let BlueprintType::Inner { outer_blueprint } = &blueprint_type {
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
                (OuterObjectInfo::None, index_set_new())
            };

        let (blueprint_info, mut node_substates) = self.validate_new_object(
            blueprint_id,
            &blueprint_definition.interface,
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
                blueprint_info,
                object_type: ObjectType::Owned,
            })),
        );

        self.api.kernel_create_node(node_id, node_substates)?;

        if blueprint_definition.interface.is_transient {
            self.api.kernel_pin_node(node_id)?;
        }

        if let Some((partition_offset, fields)) = &blueprint_definition.interface.state.fields {
            for (index, field) in fields.iter().enumerate() {
                if let FieldTransience::TransientStatic { .. } = field.transience {
                    let partition_number = match partition_offset {
                        PartitionDescription::Physical(partition_number) => *partition_number,
                        PartitionDescription::Logical(offset) => {
                            MAIN_BASE_PARTITION.at_offset(*offset).unwrap()
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
        event_flags: EventFlags,
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
                Emitter::Method(node_id, module_id.into()),
                event_name,
            )),
            EmitterActor::CurrentActor => match self.current_actor() {
                Actor::Method(MethodActor {
                    method_type,
                    node_id,
                    ..
                }) => Ok(EventTypeIdentifier(
                    Emitter::Method(node_id, method_type.module_id()),
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
            flags: event_flags,
        };

        // Adding the event to the event store
        self.api
            .kernel_get_system()
            .modules
            .checked_add_event(event)?;

        Ok(())
    }

    /// Internal, handle must be checked or from trusted sources
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
        module_id: Option<AttachedModuleId>,
    ) -> Result<BlueprintInfo, RuntimeError> {
        let info = match module_id {
            None => self.get_object_info(node_id)?.blueprint_info,
            Some(module_id) => BlueprintInfo {
                blueprint_id: module_id.static_blueprint(),
                blueprint_version: BlueprintVersion::default(),
                outer_obj_info: OuterObjectInfo::None,
                features: indexset!(),
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
                    blueprint_version: BlueprintVersion::default(),
                    outer_obj_info: OuterObjectInfo::None,
                    features: indexset!(),
                    generic_substitutions: vec![],
                },
                meta: SchemaValidationMeta::Blueprint,
            }),
            Actor::Function(actor) => Ok(BlueprintTypeTarget {
                blueprint_info: BlueprintInfo {
                    blueprint_id: actor.blueprint_id.clone(),
                    blueprint_version: BlueprintVersion::default(),
                    outer_obj_info: OuterObjectInfo::None,
                    features: indexset!(),
                    generic_substitutions: vec![],
                },
                meta: SchemaValidationMeta::Blueprint,
            }),
            Actor::Method(actor) => {
                let blueprint_info =
                    self.get_blueprint_info(&actor.node_id, actor.method_type.module_id().into())?;
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
        actor_object_type: ActorStateRef,
    ) -> Result<(NodeId, Option<AttachedModuleId>), RuntimeError> {
        let actor = self.current_actor();
        let object_id = actor
            .get_object_id()
            .ok_or_else(|| RuntimeError::SystemError(SystemError::NotAnObject))?;

        let object_id = match actor_object_type {
            ActorStateRef::OuterObject => {
                let module_id = object_id.1;

                match module_id {
                    None => {
                        let node_id = object_id.0;
                        let address = self.get_outer_object(&node_id)?;

                        (address.into_node_id(), None)
                    }
                    _ => {
                        return Err(RuntimeError::SystemError(
                            SystemError::OuterObjectDoesNotExist,
                        ));
                    }
                }
            }
            ActorStateRef::SELF => object_id,
        };

        Ok(object_id)
    }

    fn get_actor_collection_partition_info(
        &mut self,
        actor_object_type: ActorStateRef,
        collection_index: u8,
        expected_type: &BlueprintPartitionType,
    ) -> Result<(NodeId, BlueprintInfo, PartitionNumber), RuntimeError> {
        let (node_id, module_id) = self.get_actor_object_id(actor_object_type)?;
        let blueprint_info = self.get_blueprint_info(&node_id, module_id)?;
        let blueprint_definition =
            self.get_blueprint_default_definition(blueprint_info.blueprint_id.clone())?;

        let partition_num = {
            let (partition_description, partition_type) = blueprint_definition
                .interface
                .state
                .get_partition(collection_index)
                .ok_or_else(|| {
                    RuntimeError::SystemError(SystemError::CollectionIndexDoesNotExist(
                        blueprint_info.blueprint_id.clone(),
                        collection_index,
                    ))
                })?;

            if !partition_type.eq(expected_type) {
                return Err(RuntimeError::SystemError(
                    SystemError::CollectionIndexIsOfWrongType(
                        blueprint_info.blueprint_id.clone(),
                        collection_index,
                        expected_type.to_owned(),
                        partition_type,
                    ),
                ));
            }

            match partition_description {
                PartitionDescription::Physical(partition_num) => partition_num,
                PartitionDescription::Logical(offset) => {
                    let base = match module_id {
                        None => MAIN_BASE_PARTITION,
                        Some(module_id) => {
                            let object_module: ModuleId = module_id.into();
                            object_module.base_partition_num()
                        }
                    };
                    base.at_offset(offset).expect("Module number overflow")
                }
            }
        };

        Ok((node_id, blueprint_info, partition_num))
    }

    fn get_actor_info(
        &mut self,
        actor_object_type: ActorStateRef,
    ) -> Result<
        (
            NodeId,
            Option<AttachedModuleId>,
            Rc<BlueprintDefinition>,
            BlueprintInfo,
        ),
        RuntimeError,
    > {
        let (node_id, module_id) = self.get_actor_object_id(actor_object_type)?;
        let blueprint_info = self.get_blueprint_info(&node_id, module_id)?;
        let blueprint_definition =
            self.get_blueprint_default_definition(blueprint_info.blueprint_id.clone())?;

        Ok((node_id, module_id, blueprint_definition, blueprint_info))
    }

    fn get_actor_field_info(
        &mut self,
        actor_object_type: ActorStateRef,
        field_index: u8,
    ) -> Result<(NodeId, BlueprintInfo, PartitionNumber, FieldTransience), RuntimeError> {
        let (node_id, module_id, blueprint_definition, info) =
            self.get_actor_info(actor_object_type)?;

        let (partition_description, field_schema) = blueprint_definition
            .interface
            .state
            .field(field_index)
            .ok_or_else(|| {
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

                if !self.is_feature_enabled(&parent_id, None, feature.as_str())? {
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
            PartitionDescription::Logical(offset) => {
                let base = match module_id {
                    None => MAIN_BASE_PARTITION,
                    Some(module_id) => {
                        let object_module: ModuleId = module_id.into();
                        object_module.base_partition_num()
                    }
                };
                base.at_offset(offset).expect("Module number overflow")
            }
        };

        Ok((node_id, info, partition_num, field_schema.transience))
    }

    /// ASSUMPTIONS:
    /// Assumes the caller has already checked that the entity type on the GlobalAddress is valid
    /// against the given self module.
    fn globalize_with_address_internal(
        &mut self,
        node_id: NodeId,
        modules: IndexMap<AttachedModuleId, NodeId>,
        global_address_reservation: GlobalAddressReservation,
    ) -> Result<GlobalAddress, RuntimeError> {
        // Check global address reservation
        let global_address = {
            let dropped_node = self.kernel_drop_node(global_address_reservation.0.as_node_id())?;

            let type_info: Option<TypeInfoSubstate> = dropped_node
                .substates
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
        if !modules.contains_key(&AttachedModuleId::RoleAssignment) {
            return Err(RuntimeError::SystemError(SystemError::MissingModule(
                ModuleId::RoleAssignment,
            )));
        }
        if !modules.contains_key(&AttachedModuleId::Metadata) {
            return Err(RuntimeError::SystemError(SystemError::MissingModule(
                ModuleId::Metadata,
            )));
        }

        self.api
            .kernel_get_system_state()
            .system
            .modules
            .add_replacement(
                (node_id, ModuleId::Main),
                (*global_address.as_node_id(), ModuleId::Main),
            );

        // Read the type info
        let mut object_info = self.get_object_info(&node_id)?;

        // Verify can globalize with address
        let num_main_partitions = {
            if object_info.is_global() {
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
            let blueprint_definition = self.get_blueprint_default_definition(
                object_info.blueprint_info.blueprint_id.clone(),
            )?;

            if blueprint_definition.interface.is_transient {
                return Err(RuntimeError::SystemError(
                    SystemError::GlobalizingTransientBlueprint,
                ));
            }

            blueprint_definition
                .interface
                .state
                .num_logical_partitions()
        };

        let mut partitions = btreemap!(
            SCHEMAS_PARTITION => (node_id, SCHEMAS_PARTITION),
        );

        // Move self modules to the newly created global node, and drop
        for offset in 0u8..num_main_partitions {
            let partition_number = MAIN_BASE_PARTITION
                .at_offset(PartitionOffset(offset))
                .unwrap();

            partitions.insert(partition_number, (node_id, partition_number));
        }

        // Move other modules, and drop
        for (module_id, node_id) in &modules {
            match module_id {
                AttachedModuleId::RoleAssignment
                | AttachedModuleId::Metadata
                | AttachedModuleId::Royalty => {
                    let blueprint_id = self.get_object_info(node_id)?.blueprint_info.blueprint_id;
                    let expected_blueprint = module_id.static_blueprint();
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
                            (*node_id, ModuleId::Main),
                            (*global_address.as_node_id(), module_id.clone().into()),
                        );

                    // Move and drop
                    let blueprint_definition =
                        self.get_blueprint_default_definition(blueprint_id.clone())?;
                    let num_logical_partitions = blueprint_definition
                        .interface
                        .state
                        .num_logical_partitions();

                    let module_id: ModuleId = module_id.clone().into();
                    let module_base_partition = module_id.base_partition_num();
                    for offset in 0u8..num_logical_partitions {
                        let src = MAIN_BASE_PARTITION
                            .at_offset(PartitionOffset(offset))
                            .unwrap();
                        let dest = module_base_partition
                            .at_offset(PartitionOffset(offset))
                            .unwrap();

                        partitions.insert(dest, (*node_id, src));
                    }
                }
            }
        }

        self.kernel_create_node_from(global_address.into(), partitions)?;

        // Update Object Info
        {
            let mut module_versions = index_map_new();
            for module_id in modules.keys() {
                module_versions.insert(module_id.clone(), BlueprintVersion::default());
            }
            object_info.object_type = ObjectType::Global {
                modules: module_versions,
            };

            self.kernel_set_substate(
                &global_address.into(),
                TYPE_INFO_FIELD_PARTITION,
                SubstateKey::Field(0u8),
                IndexedScryptoValue::from_typed(&TypeInfoSubstate::Object(object_info)),
            )?;
        }

        // Drop nodes
        {
            self.kernel_drop_node(&node_id)?;
            for (_module_id, node_id) in &modules {
                self.kernel_drop_node(&node_id)?;
            }
        }

        Ok(global_address)
    }

    #[cfg_attr(feature = "std", catch_unwind_ignore)]
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
        module_id: Option<AttachedModuleId>,
        feature: &str,
    ) -> Result<bool, RuntimeError> {
        match module_id {
            None => {
                let object_info = self.get_object_info(node_id)?;
                let enabled = object_info.blueprint_info.features.contains(feature);
                Ok(enabled)
            }
            _ => Ok(false),
        }
    }
}

#[cfg_attr(
    feature = "std",
    catch_unwind(crate::utils::catch_unwind_system_panic_transformer)
)]
impl<'a, Y: SystemBasedKernelApi> SystemFieldApi<RuntimeError> for SystemService<'a, Y> {
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
            scrypto_encode(&wrapper.into_payload()).unwrap()
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

        let substate = IndexedScryptoValue::from_typed(&FieldSubstate::new_unlocked_field(value));

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

#[cfg_attr(
    feature = "std",
    catch_unwind(crate::utils::catch_unwind_system_panic_transformer)
)]
impl<'a, Y: SystemBasedKernelApi> SystemObjectApi<RuntimeError> for SystemService<'a, Y> {
    // Costing through kernel
    #[trace_resources]
    fn new_object(
        &mut self,
        blueprint_ident: &str,
        features: Vec<&str>,
        generic_args: GenericArgs,
        fields: IndexMap<u8, FieldValue>,
        kv_entries: IndexMap<u8, IndexMap<Vec<u8>, KVEntry>>,
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
        node_id: NodeId,
        modules: IndexMap<AttachedModuleId, NodeId>,
        address_reservation: Option<GlobalAddressReservation>,
    ) -> Result<GlobalAddress, RuntimeError> {
        // TODO: optimize by skipping address allocation
        let (global_address_reservation, global_address) =
            if let Some(reservation) = address_reservation {
                let address = self.get_reservation_address(reservation.0.as_node_id())?;
                (reservation, address)
            } else {
                let blueprint_id = self.get_object_info(&node_id)?.blueprint_info.blueprint_id;
                self.allocate_global_address(blueprint_id)?
            };

        self.globalize_with_address_internal(node_id, modules, global_address_reservation)?;

        Ok(global_address)
    }

    // Costing through kernel
    #[trace_resources]
    fn globalize_with_address_and_create_inner_object_and_emit_event(
        &mut self,
        node_id: NodeId,
        modules: IndexMap<AttachedModuleId, NodeId>,
        address_reservation: GlobalAddressReservation,
        inner_object_blueprint: &str,
        inner_object_fields: IndexMap<u8, FieldValue>,
        event_name: &str,
        event_data: Vec<u8>,
    ) -> Result<(GlobalAddress, NodeId), RuntimeError> {
        let actor_blueprint = self.get_object_info(&node_id)?.blueprint_info.blueprint_id;

        let global_address =
            self.globalize_with_address_internal(node_id, modules, address_reservation)?;

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
            indexmap!(),
        )?;

        self.emit_event_internal(
            EmitterActor::AsObject(global_address.as_node_id().clone(), None),
            event_name.to_string(),
            event_data,
            EventFlags::empty(),
        )?;

        Ok((global_address, inner_object))
    }

    #[trace_resources]
    fn call_method(
        &mut self,
        receiver: &NodeId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let object_info = self.get_object_info(&receiver)?;

        let args = IndexedScryptoValue::from_vec(args).map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        let auth_actor_info = SystemModuleMixer::on_call_method(
            self,
            receiver,
            ModuleId::Main,
            false,
            method_name,
            &args,
        )?;

        let rtn = self
            .api
            .kernel_invoke(Box::new(KernelInvocation {
                call_frame_data: Actor::Method(MethodActor {
                    method_type: MethodType::Main,
                    node_id: receiver.clone(),
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

    #[trace_resources]
    fn call_direct_access_method(
        &mut self,
        receiver: &NodeId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let object_info = self.get_object_info(&receiver)?;

        let args = IndexedScryptoValue::from_vec(args).map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        let auth_actor_info = SystemModuleMixer::on_call_method(
            self,
            receiver,
            ModuleId::Main,
            true,
            method_name,
            &args,
        )?;

        let rtn = self
            .api
            .kernel_invoke(Box::new(KernelInvocation {
                call_frame_data: Actor::Method(MethodActor {
                    method_type: MethodType::Direct,
                    node_id: receiver.clone(),
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
    fn call_module_method(
        &mut self,
        receiver: &NodeId,
        module_id: AttachedModuleId,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        // Key Value Stores do not have methods so we remove that possibility here
        let object_info = self.get_object_info(&receiver)?;
        match &object_info.object_type {
            ObjectType::Owned => {
                return Err(RuntimeError::SystemError(
                    SystemError::ObjectModuleDoesNotExist(module_id),
                ));
            }
            ObjectType::Global { modules } => {
                if !modules.contains_key(&module_id) {
                    return Err(RuntimeError::SystemError(
                        SystemError::ObjectModuleDoesNotExist(module_id),
                    ));
                }
            }
        }

        let args = IndexedScryptoValue::from_vec(args).map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        let auth_actor_info = SystemModuleMixer::on_call_method(
            self,
            receiver,
            module_id.into(),
            false,
            method_name,
            &args,
        )?;

        let rtn = self
            .api
            .kernel_invoke(Box::new(KernelInvocation {
                call_frame_data: Actor::Method(MethodActor {
                    method_type: MethodType::Module(module_id),
                    node_id: receiver.clone(),
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

        let instance_context_check = {
            // Allow proofs to be dropped on their own
            if info.blueprint_info.blueprint_id.eq(&BlueprintId::new(
                &RESOURCE_PACKAGE,
                FUNGIBLE_PROOF_BLUEPRINT,
            )) || info.blueprint_info.blueprint_id.eq(&BlueprintId::new(
                &RESOURCE_PACKAGE,
                NON_FUNGIBLE_PROOF_BLUEPRINT,
            )) {
                None
            } else {
                match info.blueprint_info.outer_obj_info {
                    OuterObjectInfo::Some { outer_object } => Some(outer_object),
                    OuterObjectInfo::None => None,
                }
            }
        };

        // If outer object exists, only outer object may drop object
        if let Some(outer_object) = instance_context_check {
            match actor.instance_context() {
                Some(instance_context) if instance_context.outer_object.eq(&outer_object) => {}
                _ => {
                    return Err(RuntimeError::SystemError(SystemError::InvalidDropAccess(
                        Box::new(InvalidDropAccess {
                            node_id: (*node_id).into(),
                            package_address: info.blueprint_info.blueprint_id.package_address,
                            blueprint_name: info.blueprint_info.blueprint_id.blueprint_name,
                            actor_package: actor.package_address(),
                        }),
                    )));
                }
            }
        } else {
            // Otherwise, only blueprint may drop object
            if Some(info.blueprint_info.blueprint_id.clone()) != actor.blueprint_id() {
                return Err(RuntimeError::SystemError(SystemError::InvalidDropAccess(
                    Box::new(InvalidDropAccess {
                        node_id: (*node_id).into(),
                        package_address: info.blueprint_info.blueprint_id.package_address,
                        blueprint_name: info.blueprint_info.blueprint_id.blueprint_name,
                        actor_package: actor.package_address(),
                    }),
                )));
            }
        }

        let mut dropped_node = self.api.kernel_drop_node(&node_id)?;
        let fields =
            if let Some(user_substates) = dropped_node.substates.remove(&MAIN_BASE_PARTITION) {
                user_substates
                    .into_iter()
                    .map(|(_key, v)| {
                        let substate: FieldSubstate<ScryptoValue> = v.as_typed().unwrap();
                        scrypto_encode(&substate.into_payload()).unwrap()
                    })
                    .collect()
            } else {
                vec![]
            };

        Ok(fields)
    }
}

#[cfg_attr(
    feature = "std",
    catch_unwind(crate::utils::catch_unwind_system_panic_transformer)
)]
impl<'a, Y: SystemBasedKernelApi> SystemKeyValueEntryApi<RuntimeError> for SystemService<'a, Y> {
    // Costing through kernel
    #[trace_resources]
    fn key_value_entry_get(
        &mut self,
        handle: KeyValueEntryHandle,
    ) -> Result<Vec<u8>, RuntimeError> {
        let data = self.api.kernel_get_lock_data(handle)?;
        if !data.is_kv_entry() {
            return Err(RuntimeError::SystemError(
                SystemError::NotAKeyValueEntryHandle,
            ));
        }

        self.api.kernel_read_substate(handle).map(|v| {
            let wrapper: KeyValueEntrySubstate<ScryptoValue> = v.as_typed().unwrap();
            scrypto_encode(&wrapper.into_value()).unwrap()
        })
    }

    // Costing through kernel
    fn key_value_entry_lock(&mut self, handle: KeyValueEntryHandle) -> Result<(), RuntimeError> {
        let data = self.api.kernel_get_lock_data(handle)?;
        match data {
            SystemLockData::KeyValueEntry(
                KeyValueEntryLockData::KVStoreWrite { .. }
                | KeyValueEntryLockData::KVCollectionWrite { .. },
            ) => {}
            _ => {
                return Err(RuntimeError::SystemError(
                    SystemError::NotAKeyValueEntryWriteHandle,
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
        let data = self.api.kernel_get_lock_data(handle)?;
        if !data.is_kv_entry_with_write() {
            return Err(RuntimeError::SystemError(
                SystemError::NotAKeyValueEntryWriteHandle,
            ));
        }

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
            SystemLockData::KeyValueEntry(KeyValueEntryLockData::KVCollectionWrite {
                collection_index,
                target,
            }) => {
                self.validate_blueprint_payload(
                    &target,
                    BlueprintPayloadIdentifier::KeyValueEntry(collection_index, KeyOrValue::Value),
                    &buffer,
                )?;
            }
            SystemLockData::KeyValueEntry(KeyValueEntryLockData::KVStoreWrite {
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
                    SystemError::NotAKeyValueEntryWriteHandle,
                ));
            }
        }

        let substate =
            IndexedScryptoValue::from_slice(&buffer).expect("Should be valid due to payload check");

        let value = substate.as_scrypto_value().clone();
        let kv_entry = KeyValueEntrySubstate::unlocked_entry(value);
        let indexed = IndexedScryptoValue::from_typed(&kv_entry);

        self.api.kernel_write_substate(handle, indexed)?;

        Ok(())
    }

    // Costing through kernel
    fn key_value_entry_close(&mut self, handle: KeyValueEntryHandle) -> Result<(), RuntimeError> {
        let data = self.api.kernel_get_lock_data(handle)?;
        if !data.is_kv_entry() {
            return Err(RuntimeError::SystemError(
                SystemError::NotAKeyValueEntryHandle,
            ));
        }

        self.api.kernel_close_substate(handle)
    }
}

#[cfg_attr(
    feature = "std",
    catch_unwind(crate::utils::catch_unwind_system_panic_transformer)
)]
impl<'a, Y: SystemBasedKernelApi> SystemKeyValueStoreApi<RuntimeError> for SystemService<'a, Y> {
    // Costing through kernel
    #[trace_resources]
    fn key_value_store_new(
        &mut self,
        data_schema: KeyValueStoreDataSchema,
    ) -> Result<NodeId, RuntimeError> {
        let mut additional_schemas = index_map_new();
        let (key_type, value_type, allow_ownership) = match data_schema {
            KeyValueStoreDataSchema::Local {
                additional_schema,
                key_type,
                value_type,
                allow_ownership,
            } => {
                validate_schema(additional_schema.v1())
                    .map_err(|_| RuntimeError::SystemError(SystemError::InvalidGenericArgs))?;
                let schema_hash = additional_schema.generate_schema_hash();
                additional_schemas.insert(schema_hash, additional_schema);
                (
                    GenericSubstitution::Local(ScopedTypeId(schema_hash, key_type)),
                    GenericSubstitution::Local(ScopedTypeId(schema_hash, value_type)),
                    allow_ownership,
                )
            }
            KeyValueStoreDataSchema::Remote {
                key_type,
                value_type,
                allow_ownership,
            } => (
                GenericSubstitution::Remote(key_type),
                GenericSubstitution::Remote(value_type),
                allow_ownership,
            ),
        };

        self.validate_kv_store_generic_args(&additional_schemas, &key_type, &value_type)
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
            key_generic_substitution: key_type,
            value_generic_substitution: value_type,
            allow_ownership: allow_ownership,
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

        let target = KVStoreTypeTarget {
            kv_store_type: info.generic_substitutions,
            meta: *node_id,
        };

        self.validate_kv_store_payload(&target, KeyOrValue::Key, &key)?;

        let lock_data = if flags.contains(LockFlags::MUTABLE) {
            SystemLockData::KeyValueEntry(KeyValueEntryLockData::KVStoreWrite {
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
            let lock_status = self.api.kernel_read_substate(handle).map(|v| {
                let kv_entry: KeyValueEntrySubstate<ScryptoValue> = v.as_typed().unwrap();
                kv_entry.lock_status()
            })?;

            if let LockStatus::Locked = lock_status {
                return Err(RuntimeError::SystemError(SystemError::KeyValueEntryLocked));
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

#[cfg_attr(
    feature = "std",
    catch_unwind(crate::utils::catch_unwind_system_panic_transformer)
)]
impl<'a, Y: SystemBasedKernelApi> SystemActorIndexApi<RuntimeError> for SystemService<'a, Y> {
    // Costing through kernel
    fn actor_index_insert(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        key: Vec<u8>,
        buffer: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        let actor_object_type: ActorStateRef = object_handle.try_into()?;

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

        let value: ScryptoValue = scrypto_decode(&buffer).unwrap();
        let index_entry = IndexEntrySubstate::entry(value);
        let value = IndexedScryptoValue::from_typed(&index_entry);

        self.api
            .kernel_set_substate(&node_id, partition_num, SubstateKey::Map(key), value)
    }

    // Costing through kernel
    fn actor_index_remove(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        key: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, RuntimeError> {
        let actor_object_type: ActorStateRef = object_handle.try_into()?;

        let (node_id, _info, partition_num) = self.get_actor_collection_partition_info(
            actor_object_type,
            collection_index,
            &BlueprintPartitionType::IndexCollection,
        )?;

        let rtn = self
            .api
            .kernel_remove_substate(&node_id, partition_num, &SubstateKey::Map(key))?
            .map(|v| {
                let value: IndexEntrySubstate<ScryptoValue> = v.as_typed().unwrap();
                scrypto_encode(value.value()).unwrap()
            });

        Ok(rtn)
    }

    // Costing through kernel
    fn actor_index_scan_keys(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        limit: u32,
    ) -> Result<Vec<Vec<u8>>, RuntimeError> {
        let actor_object_type: ActorStateRef = object_handle.try_into()?;

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
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        limit: u32,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, RuntimeError> {
        let actor_object_type: ActorStateRef = object_handle.try_into()?;

        let (node_id, _info, partition_num) = self.get_actor_collection_partition_info(
            actor_object_type,
            collection_index,
            &BlueprintPartitionType::IndexCollection,
        )?;

        let substates = self
            .api
            .kernel_drain_substates::<MapKey>(&node_id, partition_num, limit)?
            .into_iter()
            .map(|(key, value)| {
                let value: IndexEntrySubstate<ScryptoValue> = value.as_typed().unwrap();
                let value = scrypto_encode(value.value()).unwrap();

                (key.into_map(), value)
            })
            .collect();

        Ok(substates)
    }
}

#[cfg_attr(
    feature = "std",
    catch_unwind(crate::utils::catch_unwind_system_panic_transformer)
)]
impl<'a, Y: SystemBasedKernelApi> SystemActorSortedIndexApi<RuntimeError> for SystemService<'a, Y> {
    // Costing through kernel
    #[trace_resources]
    fn actor_sorted_index_insert(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        sorted_key: SortedKey,
        buffer: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        let actor_object_type: ActorStateRef = object_handle.try_into()?;

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

        let value: ScryptoValue = scrypto_decode(&buffer).unwrap();
        let sorted_entry = SortedIndexEntrySubstate::entry(value);
        let value = IndexedScryptoValue::from_typed(&sorted_entry);

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
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        sorted_key: &SortedKey,
    ) -> Result<Option<Vec<u8>>, RuntimeError> {
        let actor_object_type: ActorStateRef = object_handle.try_into()?;

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
            .map(|v| {
                let value: SortedIndexEntrySubstate<ScryptoValue> = v.as_typed().unwrap();
                scrypto_encode(value.value()).unwrap()
            });

        Ok(rtn)
    }

    // Costing through kernel
    #[trace_resources]
    fn actor_sorted_index_scan(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        limit: u32,
    ) -> Result<Vec<(SortedKey, Vec<u8>)>, RuntimeError> {
        let actor_object_type: ActorStateRef = object_handle.try_into()?;

        let (node_id, _info, partition_num) = self.get_actor_collection_partition_info(
            actor_object_type,
            collection_index,
            &BlueprintPartitionType::SortedIndexCollection,
        )?;

        let substates = self
            .api
            .kernel_scan_sorted_substates(&node_id, partition_num, limit)?
            .into_iter()
            .map(|(key, value)| {
                let value: SortedIndexEntrySubstate<ScryptoValue> = value.as_typed().unwrap();
                let value = scrypto_encode(value.value()).unwrap();

                (key, value)
            })
            .collect();

        Ok(substates)
    }
}

#[cfg_attr(
    feature = "std",
    catch_unwind(crate::utils::catch_unwind_system_panic_transformer)
)]
impl<'a, Y: SystemBasedKernelApi> SystemBlueprintApi<RuntimeError> for SystemService<'a, Y> {
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

    fn resolve_blueprint_type(
        &mut self,
        blueprint_type_id: &BlueprintTypeIdentifier,
    ) -> Result<(Rc<VersionedScryptoSchema>, ScopedTypeId), RuntimeError> {
        self.get_blueprint_type_schema(blueprint_type_id)
    }
}

#[cfg_attr(
    feature = "std",
    catch_unwind(crate::utils::catch_unwind_system_panic_transformer)
)]
impl<'a, Y: SystemBasedKernelApi> SystemCostingApi<RuntimeError> for SystemService<'a, Y> {
    fn consume_cost_units(
        &mut self,
        costing_entry: ClientCostingEntry,
    ) -> Result<(), RuntimeError> {
        let system_logic = self
            .api
            .kernel_get_system_state()
            .system
            .versioned_system_logic;
        if !system_logic.should_consume_cost_units(self.api) {
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
                ClientCostingEntry::Bls12381V1Verify { size } => {
                    ExecutionCostingEntry::Bls12381V1Verify { size }
                }
                ClientCostingEntry::Bls12381V1AggregateVerify { sizes } => {
                    ExecutionCostingEntry::Bls12381V1AggregateVerify { sizes }
                }
                ClientCostingEntry::Bls12381V1FastAggregateVerify { size, keys_cnt } => {
                    ExecutionCostingEntry::Bls12381V1FastAggregateVerify { size, keys_cnt }
                }
                ClientCostingEntry::Bls12381G2SignatureAggregate { signatures_cnt } => {
                    ExecutionCostingEntry::Bls12381G2SignatureAggregate { signatures_cnt }
                }
                ClientCostingEntry::Keccak256Hash { size } => {
                    ExecutionCostingEntry::Keccak256Hash { size }
                }
                ClientCostingEntry::Blake2b256Hash { size } => {
                    ExecutionCostingEntry::Blake2b256Hash { size }
                }
                ClientCostingEntry::Ed25519Verify { size } => {
                    ExecutionCostingEntry::Ed25519Verify { size }
                }
                ClientCostingEntry::Secp256k1EcdsaVerify => {
                    ExecutionCostingEntry::Secp256k1EcdsaVerify
                }
                ClientCostingEntry::Secp256k1EcdsaKeyRecover => {
                    ExecutionCostingEntry::Secp256k1EcdsaVerifyAndKeyRecover
                }
            })
    }

    #[trace_resources]
    fn start_lock_fee(&mut self, amount: Decimal, contingent: bool) -> Result<bool, RuntimeError> {
        // Child subintents are only allowed to use contingent fees
        if !contingent {
            let stack_id = self.api.kernel_get_current_stack_id_uncosted();
            if stack_id != 0 {
                return Err(RuntimeError::SystemError(
                    SystemError::CannotLockFeeInChildSubintent(stack_id),
                ));
            }
        }

        let costing_enabled = self
            .api
            .kernel_get_system()
            .modules
            .enabled_modules
            .contains(EnabledModules::COSTING);

        // We do costing up front
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(ExecutionCostingEntry::LockFee)?;

        let event_data = {
            let lock_fee_event = LockFeeEvent { amount };
            scrypto_encode(&lock_fee_event).unwrap()
        };

        // If costing is enabled, reserve event and pay for the event up front for the call to lock_fee()
        // Otherwise, we just simulate the call
        if costing_enabled {
            self.api
                .kernel_get_system()
                .modules
                .assert_can_add_event()?;
            self.api.kernel_get_system().modules.apply_execution_cost(
                ExecutionCostingEntry::EmitEvent {
                    size: event_data.len(),
                },
            )?;
        } else {
            self.emit_event_internal(
                EmitterActor::CurrentActor,
                LockFeeEvent::EVENT_NAME.to_string(),
                event_data,
                EventFlags::FORCE_WRITE,
            )?;
        }

        Ok(costing_enabled)
    }

    #[trace_resources]
    #[cfg_attr(feature = "std", catch_unwind_ignore)]
    fn lock_fee(&mut self, locked_fee: LiquidFungibleResource, contingent: bool) {
        // Credit cost units
        let vault_id = self
            .current_actor()
            .node_id()
            .expect("Caller should only be fungible vault method");
        self.api
            .kernel_get_system()
            .modules
            .lock_fee(vault_id, locked_fee.clone(), contingent);

        // Emit Locked Fee event
        {
            let type_identifier = EventTypeIdentifier(
                Emitter::Method(vault_id, ObjectModuleId::Main),
                LockFeeEvent::EVENT_NAME.to_string(),
            );

            let lock_fee_event = LockFeeEvent {
                amount: locked_fee.amount(),
            };
            let payload = scrypto_encode(&lock_fee_event).unwrap();

            let event = Event {
                type_identifier,
                payload,
                flags: EventFlags::FORCE_WRITE,
            };

            self.api
                .kernel_get_system()
                .modules
                .add_event_unchecked(event)
                .expect("Event should never exceed size.");
        }
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
        if let Some(costing) = self.api.kernel_get_system().modules.costing_mut() {
            costing
                .apply_execution_cost_2(ExecutionCostingEntry::QueryFeeReserve)
                .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;
        }

        if let Some(fee_reserve) = self.api.kernel_get_system().modules.fee_reserve() {
            Ok(fee_reserve.usd_price())
        } else {
            Err(RuntimeError::SystemError(
                SystemError::CostingModuleNotEnabled,
            ))
        }
    }

    fn max_per_function_royalty_in_xrd(&mut self) -> Result<Decimal, RuntimeError> {
        if let Some(costing) = self.api.kernel_get_system().modules.costing_mut() {
            costing
                .apply_execution_cost_2(ExecutionCostingEntry::QueryCostingModule)
                .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;
            Ok(costing.config.max_per_function_royalty_in_xrd)
        } else {
            Err(RuntimeError::SystemError(
                SystemError::CostingModuleNotEnabled,
            ))
        }
    }

    fn tip_percentage_truncated(&mut self) -> Result<u32, RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(ExecutionCostingEntry::QueryFeeReserve)?;

        if let Some(fee_reserve) = self.api.kernel_get_system().modules.fee_reserve() {
            Ok(fee_reserve.tip().truncate_to_percentage_u32())
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

#[cfg_attr(
    feature = "std",
    catch_unwind(crate::utils::catch_unwind_system_panic_transformer)
)]
impl<'a, Y: SystemBasedKernelApi> SystemActorApi<RuntimeError> for SystemService<'a, Y> {
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

    #[trace_resources]
    fn actor_get_node_id(&mut self, ref_handle: ActorRefHandle) -> Result<NodeId, RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(ExecutionCostingEntry::QueryActor)?;

        let actor_ref: ActorObjectRef = ref_handle.try_into()?;

        let node_id = match actor_ref {
            ActorObjectRef::SELF => {
                self.current_actor()
                    .node_id()
                    .ok_or(RuntimeError::SystemError(
                        SystemError::ActorNodeIdDoesNotExist,
                    ))?
            }
            ActorObjectRef::Outer => {
                let (node_id, module_id) = self.get_actor_object_id(ActorStateRef::SELF)?;
                match module_id {
                    None => {
                        let info = self.get_object_info(&node_id)?;
                        match info.blueprint_info.outer_obj_info {
                            OuterObjectInfo::Some { outer_object } => {
                                Ok(outer_object.into_node_id())
                            }
                            OuterObjectInfo::None => Err(RuntimeError::SystemError(
                                SystemError::OuterObjectDoesNotExist,
                            )),
                        }
                    }
                    _ => Err(RuntimeError::SystemError(
                        SystemError::ModulesDontHaveOuterObjects,
                    )),
                }?
            }
            ActorObjectRef::Global => {
                let actor = self.current_actor();
                if actor.is_direct_access() {
                    return Err(RuntimeError::SystemError(
                        SystemError::GlobalAddressDoesNotExist,
                    ));
                }

                if let Some(node_id) = actor.node_id() {
                    let visibility = self.kernel_get_node_visibility_uncosted(&node_id);
                    if let ReferenceOrigin::Global(address) =
                        visibility.reference_origin(node_id).unwrap()
                    {
                        address.into_node_id()
                    } else {
                        return Err(RuntimeError::SystemError(
                            SystemError::GlobalAddressDoesNotExist,
                        ));
                    }
                } else {
                    return Err(RuntimeError::SystemError(
                        SystemError::GlobalAddressDoesNotExist,
                    ));
                }
            }
            ActorObjectRef::AuthZone => self
                .current_actor()
                .self_auth_zone()
                .ok_or_else(|| RuntimeError::SystemError(SystemError::AuthModuleNotEnabled))?,
        };

        Ok(node_id)
    }

    #[trace_resources]
    fn actor_is_feature_enabled(
        &mut self,
        object_handle: ActorStateHandle,
        feature: &str,
    ) -> Result<bool, RuntimeError> {
        self.api
            .kernel_get_system()
            .modules
            .apply_execution_cost(ExecutionCostingEntry::QueryActor)?;

        let actor_object_type: ActorStateRef = object_handle.try_into()?;
        let (node_id, module_id) = self.get_actor_object_id(actor_object_type)?;
        self.is_feature_enabled(&node_id, module_id, feature)
    }

    // Costing through kernel
    #[trace_resources]
    fn actor_open_field(
        &mut self,
        object_handle: ActorStateHandle,
        field_index: u8,
        flags: LockFlags,
    ) -> Result<SubstateHandle, RuntimeError> {
        let actor_object_type: ActorStateRef = object_handle.try_into()?;

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
                        IndexedScryptoValue::from_typed(&FieldSubstate::new_unlocked_field(
                            default_value,
                        ))
                    }),
                    SystemLockData::Field(lock_data),
                )?
            }
        };

        if flags.contains(LockFlags::MUTABLE) {
            let lock_status = self.api.kernel_read_substate(handle).map(|v| {
                let field: FieldSubstate<ScryptoValue> = v.as_typed().unwrap();
                field.into_lock_status()
            })?;

            if let LockStatus::Locked = lock_status {
                return Err(RuntimeError::SystemError(SystemError::FieldLocked(
                    object_handle,
                    field_index,
                )));
            }
        }

        Ok(handle)
    }

    #[trace_resources]
    fn actor_emit_event(
        &mut self,
        event_name: String,
        event_data: Vec<u8>,
        event_flags: EventFlags,
    ) -> Result<(), RuntimeError> {
        if event_flags.contains(EventFlags::FORCE_WRITE) {
            let blueprint_id = self.actor_get_blueprint_id()?;

            if !blueprint_id.package_address.eq(&RESOURCE_PACKAGE)
                || !blueprint_id.blueprint_name.eq(FUNGIBLE_VAULT_BLUEPRINT)
            {
                return Err(RuntimeError::SystemError(
                    SystemError::ForceWriteEventFlagsNotAllowed,
                ));
            }
        }

        self.emit_event_internal(
            EmitterActor::CurrentActor,
            event_name,
            event_data,
            event_flags,
        )
    }
}

#[cfg_attr(
    feature = "std",
    catch_unwind(crate::utils::catch_unwind_system_panic_transformer)
)]
impl<'a, Y: SystemBasedKernelApi> SystemActorKeyValueEntryApi<RuntimeError>
    for SystemService<'a, Y>
{
    // Costing through kernel
    #[trace_resources]
    fn actor_open_key_value_entry(
        &mut self,
        object_handle: ActorStateHandle,
        collection_index: CollectionIndex,
        key: &Vec<u8>,
        flags: LockFlags,
    ) -> Result<KeyValueEntryHandle, RuntimeError> {
        if flags.contains(LockFlags::UNMODIFIED_BASE) || flags.contains(LockFlags::FORCE_WRITE) {
            return Err(RuntimeError::SystemError(SystemError::InvalidLockFlags));
        }

        let actor_object_type: ActorStateRef = object_handle.try_into()?;

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
            KeyValueEntryLockData::KVCollectionWrite {
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

            if substate.is_locked() {
                return Err(RuntimeError::SystemError(SystemError::KeyValueEntryLocked));
            }
        }

        Ok(handle)
    }

    // Costing through kernel
    fn actor_remove_key_value_entry(
        &mut self,
        object_handle: ActorStateHandle,
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

#[cfg_attr(
    feature = "std",
    catch_unwind(crate::utils::catch_unwind_system_panic_transformer)
)]
impl<'a, Y: SystemBasedKernelApi> SystemExecutionTraceApi<RuntimeError> for SystemService<'a, Y> {
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

#[cfg_attr(
    feature = "std",
    catch_unwind(crate::utils::catch_unwind_system_panic_transformer)
)]
impl<'a, Y: SystemBasedKernelApi> SystemTransactionRuntimeApi<RuntimeError>
    for SystemService<'a, Y>
{
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

    #[trace_resources]
    fn bech32_encode_address(&mut self, address: GlobalAddress) -> Result<String, RuntimeError> {
        if let Some(costing) = self.api.kernel_get_system().modules.costing_mut() {
            costing
                .apply_execution_cost_2(ExecutionCostingEntry::EncodeBech32Address)
                .map_err(|e| RuntimeError::SystemModuleError(SystemModuleError::CostingError(e)))?;
        }

        let network_definition = &self
            .api
            .kernel_get_system()
            .modules
            .transaction_runtime
            .network_definition;

        AddressBech32Encoder::new(&network_definition)
            .encode(&address.into_node_id().0)
            .map_err(|_| RuntimeError::SystemError(SystemError::AddressBech32EncodeError))
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

        Err(RuntimeError::ApplicationError(
            ApplicationError::PanicMessage(message),
        ))
    }
}

#[cfg_attr(
    feature = "std",
    catch_unwind(crate::utils::catch_unwind_system_panic_transformer)
)]
impl<'a, Y: SystemBasedKernelApi> SystemApi<RuntimeError> for SystemService<'a, Y> {}

#[cfg_attr(
    feature = "std",
    catch_unwind(crate::utils::catch_unwind_system_panic_transformer)
)]
impl<'a, Y: SystemBasedKernelApi> KernelNodeApi for SystemService<'a, Y> {
    fn kernel_pin_node(&mut self, node_id: NodeId) -> Result<(), RuntimeError> {
        self.api.kernel_pin_node(node_id)
    }

    fn kernel_drop_node(&mut self, node_id: &NodeId) -> Result<DroppedNode, RuntimeError> {
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

    fn kernel_create_node_from(
        &mut self,
        node_id: NodeId,
        partitions: BTreeMap<PartitionNumber, (NodeId, PartitionNumber)>,
    ) -> Result<(), RuntimeError> {
        self.api.kernel_create_node_from(node_id, partitions)
    }
}

#[cfg_attr(
    feature = "std",
    catch_unwind(crate::utils::catch_unwind_system_panic_transformer)
)]
impl<'a, Y: SystemBasedKernelApi> KernelSubstateApi<SystemLockData> for SystemService<'a, Y> {
    fn kernel_mark_substate_as_transient(
        &mut self,
        node_id: NodeId,
        partition_num: PartitionNumber,
        key: SubstateKey,
    ) -> Result<(), RuntimeError> {
        self.api
            .kernel_mark_substate_as_transient(node_id, partition_num, key)
    }

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

    fn kernel_scan_keys<K: SubstateKeyContent>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        limit: u32,
    ) -> Result<Vec<SubstateKey>, RuntimeError> {
        self.api
            .kernel_scan_keys::<K>(node_id, partition_num, limit)
    }

    fn kernel_drain_substates<K: SubstateKeyContent>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        limit: u32,
    ) -> Result<Vec<(SubstateKey, IndexedScryptoValue)>, RuntimeError> {
        self.api
            .kernel_drain_substates::<K>(node_id, partition_num, limit)
    }
}

impl<'a, Y: SystemBasedKernelApi> KernelInternalApi for SystemService<'a, Y> {
    type System = Y::CallbackObject;

    fn kernel_get_system_state(&mut self) -> SystemState<'_, Y::CallbackObject> {
        self.api.kernel_get_system_state()
    }

    fn kernel_get_current_stack_depth_uncosted(&self) -> usize {
        self.api.kernel_get_current_stack_depth_uncosted()
    }

    fn kernel_get_current_stack_id_uncosted(&self) -> usize {
        self.api.kernel_get_current_stack_id_uncosted()
    }

    fn kernel_get_node_visibility_uncosted(&self, node_id: &NodeId) -> NodeVisibility {
        self.api.kernel_get_node_visibility_uncosted(node_id)
    }

    fn kernel_read_substate_uncosted(
        &self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Option<&IndexedScryptoValue> {
        self.api
            .kernel_read_substate_uncosted(node_id, partition_num, substate_key)
    }
}
