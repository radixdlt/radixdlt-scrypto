use crate::internal_prelude::*;
use crate::system::system_db_reader::{
    ObjectPartitionDescriptor, SystemDatabaseReader, SystemPartitionDescriptor, SystemReaderError,
};
use crate::system::system_substates::FieldSubstate;
use crate::system::type_info::TypeInfoSubstate;
use radix_blueprint_schema_init::Condition;
use radix_common::prelude::{
    scrypto_decode, scrypto_encode, Hash, ScryptoValue, VersionedScryptoSchema,
};
use radix_engine_interface::api::{FieldIndex, ModuleId};
use radix_engine_interface::blueprints::package::{
    BlueprintDefinition, BlueprintPayloadIdentifier, BlueprintType, KeyOrValue,
};
use radix_engine_interface::types::*;
use radix_substate_store_interface::interface::ListableSubstateDatabase;
use radix_substate_store_interface::interface::SubstateDatabase;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SystemNodeCheckerState {
    node_id: NodeId,
    node_type: SystemNodeType,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SystemNodeType {
    Object {
        object_info: ObjectInfo,
        bp_definition: BlueprintDefinition,
        expected_fields: BTreeSet<(ModuleId, FieldIndex)>,
        excluded_fields: BTreeSet<FieldIndex>,
    },
    KeyValueStore,
}

impl SystemNodeCheckerState {
    pub fn finish(&self) -> Result<(), SystemNodeCheckError> {
        match &self.node_type {
            SystemNodeType::Object {
                expected_fields, ..
            } => {
                if !expected_fields.is_empty() {
                    return Err(SystemNodeCheckError::MissingExpectedFields);
                }
            }
            SystemNodeType::KeyValueStore => {}
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct NodeCounts {
    pub node_count: usize,
    pub global_node_count: usize,
    pub interior_node_count: usize,
    pub package_count: usize,
    pub blueprint_count: usize,
    pub scrypto_global_component_count: usize,
    pub native_global_component_count: usize,
    pub object_count: BTreeMap<PackageAddress, BTreeMap<String, usize>>,
}

#[derive(Debug)]
pub struct SystemDatabaseCheckerResults {
    pub node_counts: NodeCounts,
    pub partition_count: usize,
    pub substate_count: usize,
}

#[derive(Debug)]
pub struct SystemPartitionCheckResults {
    pub substate_count: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SystemPartitionCheckError {
    NoPartitionDescription(SystemReaderError),
    MissingKeyValueStoreTarget(SystemReaderError),
    MissingKeyValueStoreKeySchema(SystemReaderError),
    MissingKeyValueStoreValueSchema(SystemReaderError),
    InvalidKeyValueStoreKey,
    InvalidKeyValueStoreValue,
    ContainsFieldWhichShouldNotExist(BlueprintId, NodeId, u8),
    InvalidFieldValue,
    MissingFieldSchema(SystemReaderError),
    MissingKeyValueCollectionKeySchema(SystemReaderError),
    MissingKeyValueCollectionValueSchema(SystemReaderError),
    InvalidKeyValueCollectionKey,
    FailedBlueprintSchemaCheck(BlueprintPayloadIdentifier),
    InvalidKeyValueCollectionValue,
    MissingIndexCollectionKeySchema(SystemReaderError),
    MissingIndexCollectionValueSchema(SystemReaderError),
    InvalidIndexCollectionKey,
    InvalidIndexCollectionValue,
    InvalidPartition,
    MissingSortedIndexCollectionKeySchema(SystemReaderError),
    MissingSortedIndexCollectionValueSchema(SystemReaderError),
    InvalidSortedIndexCollectionKey,
    InvalidSortedIndexCollectionValue,
    MissingObjectTypeTarget(SystemReaderError),
    InvalidTypeInfoKey,
    InvalidTypeInfoValue,
    InvalidSchemaKey,
    InvalidSchemaValue,
    InvalidBootLoaderPartition,
    InvalidProtocolUpdateStatusPartition,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SystemNodeCheckError {
    NoTypeInfo(SystemReaderError),
    NoMappedEntityType,
    MissingOuterObject(SystemReaderError),
    MissingExpectedFields,
    InvalidCondition,
    MissingBlueprint(SystemReaderError),
    InvalidOuterObject,
    TransientObjectFound(BlueprintId),
    FoundModuleWithConditionalFields,
    FoundGlobalAddressPhantom,
    FoundGlobalAddressReservation,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeInfo {
    Object(BlueprintId),
    KeyValueStore,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SystemDatabaseCheckError {
    NodeError(SystemNodeCheckError),
    PartitionError(NodeInfo, SystemPartitionCheckError),
}

pub trait ApplicationChecker {
    type ApplicationCheckerResults: Debug + Default;
    fn on_field(
        &mut self,
        _info: BlueprintInfo,
        _node_id: NodeId,
        _module_id: ModuleId,
        _field_index: FieldIndex,
        _value: &Vec<u8>,
    ) {
    }

    fn on_collection_entry(
        &mut self,
        _info: BlueprintInfo,
        _node_id: NodeId,
        _module_id: ModuleId,
        _collection_index: CollectionIndex,
        _key: &Vec<u8>,
        _value: &Vec<u8>,
    ) {
    }

    fn on_finish(&self) -> Self::ApplicationCheckerResults {
        Self::ApplicationCheckerResults::default()
    }
}

impl ApplicationChecker for () {
    type ApplicationCheckerResults = ();
}

pub struct SystemDatabaseChecker<A: ApplicationChecker> {
    application_checker: A,
    scrypto_global_component_count: usize,
    native_global_component_count: usize,
    object_count: BTreeMap<PackageAddress, BTreeMap<String, usize>>,
}

impl<A: ApplicationChecker> SystemDatabaseChecker<A> {
    pub fn new(checker: A) -> SystemDatabaseChecker<A> {
        SystemDatabaseChecker {
            application_checker: checker,
            scrypto_global_component_count: 0usize,
            native_global_component_count: 0usize,
            object_count: btreemap!(),
        }
    }
}

impl<A: ApplicationChecker> Default for SystemDatabaseChecker<A>
where
    A: Default,
{
    fn default() -> Self {
        Self {
            application_checker: A::default(),
            scrypto_global_component_count: 0usize,
            native_global_component_count: 0usize,
            object_count: btreemap!(),
        }
    }
}

impl<A: ApplicationChecker> SystemDatabaseChecker<A> {
    pub fn check_db<S: SubstateDatabase + ListableSubstateDatabase>(
        &mut self,
        substate_db: &S,
    ) -> Result<
        (SystemDatabaseCheckerResults, A::ApplicationCheckerResults),
        SystemDatabaseCheckError,
    > {
        let mut node_counts = NodeCounts::default();
        let mut partition_count = 0usize;
        let mut substate_count = 0usize;

        let mut current_checker_node: Option<SystemNodeCheckerState> = None;

        let reader = SystemDatabaseReader::new(substate_db);
        for (node_id, partition_number) in reader.partitions_iter() {
            let new_node = match &mut current_checker_node {
                Some(checker_state) => {
                    if node_id.ne(&checker_state.node_id) {
                        None
                    } else {
                        Some(checker_state)
                    }
                }
                None => None,
            };

            let node_checker_state = match new_node {
                None => {
                    if let Some(last_node_checker_state) = &current_checker_node {
                        last_node_checker_state
                            .finish()
                            .map_err(SystemDatabaseCheckError::NodeError)?;
                    }

                    let new_node_check_state = self
                        .check_node(&reader, &node_id, &mut node_counts)
                        .map_err(SystemDatabaseCheckError::NodeError)?;
                    current_checker_node = Some(new_node_check_state);
                    current_checker_node.as_mut().unwrap()
                }
                Some(stored_type_info) => stored_type_info,
            };

            let partition_results = self
                .check_partition(&reader, node_checker_state, partition_number)
                .map_err(|e| {
                    let node_info = match &node_checker_state.node_type {
                        SystemNodeType::Object { object_info, .. } => {
                            NodeInfo::Object(object_info.blueprint_info.blueprint_id.clone())
                        }
                        SystemNodeType::KeyValueStore {} => NodeInfo::KeyValueStore,
                    };
                    SystemDatabaseCheckError::PartitionError(node_info, e)
                })?;

            substate_count += partition_results.substate_count;
            partition_count += 1;
        }

        if let Some(finished_node) = &current_checker_node {
            finished_node
                .finish()
                .map_err(SystemDatabaseCheckError::NodeError)?;
        }

        node_counts.scrypto_global_component_count = self.scrypto_global_component_count;
        node_counts.native_global_component_count = self.native_global_component_count;
        node_counts.object_count.extend(self.object_count.clone());

        let system_checker_results = SystemDatabaseCheckerResults {
            node_counts,
            partition_count,
            substate_count,
        };

        let application_checker_results = self.application_checker.on_finish();

        Ok((system_checker_results, application_checker_results))
    }

    fn check_node<S: SubstateDatabase + ListableSubstateDatabase>(
        &mut self,
        reader: &SystemDatabaseReader<S>,
        node_id: &NodeId,
        node_counts: &mut NodeCounts,
    ) -> Result<SystemNodeCheckerState, SystemNodeCheckError> {
        let type_info = reader
            .get_type_info(node_id)
            .map_err(SystemNodeCheckError::NoTypeInfo)?;
        let _entity_type = node_id
            .entity_type()
            .ok_or_else(|| SystemNodeCheckError::NoMappedEntityType)?;
        let node_checker_state = match type_info {
            TypeInfoSubstate::Object(object_info) => {
                let bp_definition = reader
                    .get_blueprint_definition(&object_info.blueprint_info.blueprint_id)
                    .map_err(SystemNodeCheckError::MissingBlueprint)?;

                let outer_object = match (
                    &object_info.blueprint_info.outer_obj_info,
                    &bp_definition.interface.blueprint_type,
                ) {
                    (OuterObjectInfo::None, BlueprintType::Outer) => None,
                    (
                        OuterObjectInfo::Some { outer_object },
                        BlueprintType::Inner { outer_blueprint },
                    ) => {
                        let expected_outer_blueprint = BlueprintId::new(
                            &object_info.blueprint_info.blueprint_id.package_address,
                            outer_blueprint.as_str(),
                        );
                        let outer_object_info = reader
                            .get_object_info(*outer_object)
                            .map_err(SystemNodeCheckError::MissingOuterObject)?;

                        if !outer_object_info
                            .blueprint_info
                            .blueprint_id
                            .eq(&expected_outer_blueprint)
                        {
                            return Err(SystemNodeCheckError::InvalidOuterObject);
                        }

                        Some(outer_object_info)
                    }
                    _ => return Err(SystemNodeCheckError::InvalidOuterObject),
                };

                if bp_definition.interface.is_transient {
                    return Err(SystemNodeCheckError::TransientObjectFound(
                        object_info.blueprint_info.blueprint_id,
                    ));
                }

                let mut expected_fields = BTreeSet::new();
                let mut excluded_fields = BTreeSet::new();

                if let Some((_, fields)) = &bp_definition.interface.state.fields {
                    for (field_index, field_schema) in fields.iter().enumerate() {
                        match &field_schema.transience {
                            FieldTransience::TransientStatic { .. } => {
                                excluded_fields.insert(field_index as u8);
                                continue;
                            }
                            FieldTransience::NotTransient => {}
                        }

                        match &field_schema.condition {
                            Condition::Always => {
                                expected_fields.insert((ModuleId::Main, field_index as u8));
                            }
                            Condition::IfFeature(feature) => {
                                if object_info
                                    .blueprint_info
                                    .features
                                    .contains(feature.as_str())
                                {
                                    expected_fields.insert((ModuleId::Main, field_index as u8));
                                } else {
                                    excluded_fields.insert(field_index as u8);
                                }
                            }
                            Condition::IfOuterFeature(feature) => {
                                if outer_object
                                    .as_ref()
                                    .ok_or_else(|| SystemNodeCheckError::InvalidCondition)?
                                    .blueprint_info
                                    .features
                                    .contains(feature.as_str())
                                {
                                    expected_fields.insert((ModuleId::Main, field_index as u8));
                                } else {
                                    excluded_fields.insert(field_index as u8);
                                }
                            }
                        }
                    }
                }

                match &object_info.object_type {
                    ObjectType::Global { modules } => {
                        for (module_id, _version) in modules {
                            let blueprint_id = module_id.static_blueprint();
                            let module_def = reader
                                .get_blueprint_definition(&blueprint_id)
                                .map_err(SystemNodeCheckError::MissingBlueprint)?;
                            if let Some((_, fields)) = &module_def.interface.state.fields {
                                for (field_index, field_schema) in fields.iter().enumerate() {
                                    match &field_schema.condition {
                                        Condition::Always => {
                                            let module_id: ModuleId = (*module_id).into();
                                            expected_fields.insert((module_id, field_index as u8));
                                        }
                                        _ => {
                                            return Err(SystemNodeCheckError::FoundModuleWithConditionalFields);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    ObjectType::Owned => {}
                }

                if node_id.entity_type().unwrap() == EntityType::GlobalGenericComponent {
                    self.scrypto_global_component_count += 1;
                } else if node_id.is_global_component() {
                    self.native_global_component_count += 1;
                }

                self.object_count
                    .entry(object_info.blueprint_info.blueprint_id.package_address)
                    .or_default()
                    .entry(
                        object_info
                            .blueprint_info
                            .blueprint_id
                            .blueprint_name
                            .clone(),
                    )
                    .or_default()
                    .add_assign(&1);

                SystemNodeCheckerState {
                    node_id: *node_id,
                    node_type: SystemNodeType::Object {
                        object_info,
                        bp_definition: bp_definition.as_ref().clone(),
                        expected_fields,
                        excluded_fields,
                    },
                }
            }
            TypeInfoSubstate::KeyValueStore(..) => SystemNodeCheckerState {
                node_id: *node_id,
                node_type: SystemNodeType::KeyValueStore,
            },
            TypeInfoSubstate::GlobalAddressPhantom(..) => {
                return Err(SystemNodeCheckError::FoundGlobalAddressPhantom);
            }
            TypeInfoSubstate::GlobalAddressReservation(..) => {
                return Err(SystemNodeCheckError::FoundGlobalAddressReservation);
            }
        };

        if node_id.is_global_package() {
            node_counts.package_count += 1;
            let definition = reader.get_package_definition(PackageAddress::new_or_panic(node_id.0));
            node_counts.blueprint_count += definition.len();
        }

        if let SystemNodeType::Object { object_info, .. } = &node_checker_state.node_type {
            if object_info.is_global() {
                node_counts.global_node_count += 1;
            } else {
                node_counts.interior_node_count += 1;
            }
        } else {
            node_counts.interior_node_count += 1;
        }

        node_counts.node_count += 1;

        Ok(node_checker_state)
    }

    fn check_partition<S: SubstateDatabase + ListableSubstateDatabase>(
        &mut self,
        reader: &SystemDatabaseReader<S>,
        node_checker_state: &mut SystemNodeCheckerState,
        partition_number: PartitionNumber,
    ) -> Result<SystemPartitionCheckResults, SystemPartitionCheckError> {
        let partition_descriptors = reader
            .get_partition_descriptors(&node_checker_state.node_id, &partition_number)
            .map_err(SystemPartitionCheckError::NoPartitionDescription)?;

        let mut substate_count = 0;

        for partition_descriptor in partition_descriptors {
            match partition_descriptor {
                SystemPartitionDescriptor::BootLoader => {
                    if node_checker_state
                        .node_id
                        .ne(TRANSACTION_TRACKER.as_node_id())
                    {
                        return Err(SystemPartitionCheckError::InvalidBootLoaderPartition);
                    }

                    for _ in reader.field_iter(&node_checker_state.node_id, partition_number) {
                        substate_count += 1;
                    }
                }
                SystemPartitionDescriptor::ProtocolUpdateStatus => {
                    if node_checker_state
                        .node_id
                        .ne(TRANSACTION_TRACKER.as_node_id())
                    {
                        return Err(
                            SystemPartitionCheckError::InvalidProtocolUpdateStatusPartition,
                        );
                    }

                    for _ in reader.field_iter(&node_checker_state.node_id, partition_number) {
                        substate_count += 1;
                    }
                }
                SystemPartitionDescriptor::TypeInfo => {
                    for (key, value) in
                        reader.field_iter(&node_checker_state.node_id, partition_number)
                    {
                        if key != 0 {
                            return Err(SystemPartitionCheckError::InvalidTypeInfoKey);
                        };

                        let _type_info: TypeInfoSubstate = scrypto_decode(&value)
                            .map_err(|_| SystemPartitionCheckError::InvalidTypeInfoValue)?;

                        substate_count += 1;
                    }
                }
                SystemPartitionDescriptor::Schema => {
                    for (map_key, value) in
                        reader.map_iter(&node_checker_state.node_id, partition_number)
                    {
                        let _schema_hash: Hash = scrypto_decode(&map_key)
                            .map_err(|_| SystemPartitionCheckError::InvalidSchemaKey)?;

                        let _schema: KeyValueEntrySubstate<VersionedScryptoSchema> =
                            scrypto_decode(&value)
                                .map_err(|_| SystemPartitionCheckError::InvalidSchemaValue)?;

                        substate_count += 1;
                    }
                }
                SystemPartitionDescriptor::KeyValueStore => {
                    let type_target = reader
                        .get_kv_store_type_target(&node_checker_state.node_id)
                        .map_err(SystemPartitionCheckError::MissingKeyValueStoreTarget)?;
                    let key_schema = reader
                        .get_kv_store_payload_schema(&type_target, KeyOrValue::Key)
                        .map_err(SystemPartitionCheckError::MissingKeyValueStoreKeySchema)?;
                    let value_schema = reader
                        .get_kv_store_payload_schema(&type_target, KeyOrValue::Value)
                        .map_err(SystemPartitionCheckError::MissingKeyValueStoreValueSchema)?;

                    for (map_key, value) in
                        reader.map_iter(&node_checker_state.node_id, partition_number)
                    {
                        // Key Check
                        {
                            reader
                                .validate_payload(
                                    &map_key,
                                    &key_schema,
                                    KEY_VALUE_STORE_PAYLOAD_MAX_DEPTH,
                                )
                                .map_err(|_| SystemPartitionCheckError::InvalidKeyValueStoreKey)?;
                        }

                        // Value Check
                        {
                            let entry: KeyValueEntrySubstate<ScryptoValue> = scrypto_decode(&value)
                                .map_err(|_| {
                                    SystemPartitionCheckError::InvalidKeyValueStoreValue
                                })?;
                            if let Some(value) = entry.into_value() {
                                let entry_payload = scrypto_encode(&value).map_err(|_| {
                                    SystemPartitionCheckError::InvalidKeyValueStoreValue
                                })?;
                                reader
                                    .validate_payload(
                                        &entry_payload,
                                        &value_schema,
                                        KEY_VALUE_STORE_PAYLOAD_MAX_DEPTH,
                                    )
                                    .map_err(|_| {
                                        SystemPartitionCheckError::InvalidKeyValueStoreValue
                                    })?;
                            }
                        }

                        substate_count += 1;
                    }
                }
                SystemPartitionDescriptor::Object(module_id, object_partition_descriptor) => {
                    let type_target = reader
                        .get_blueprint_type_target(&node_checker_state.node_id, module_id)
                        .map_err(SystemPartitionCheckError::MissingObjectTypeTarget)?;

                    let (expected_fields, excluded_fields, object_info) =
                        match &mut node_checker_state.node_type {
                            SystemNodeType::Object {
                                expected_fields,
                                excluded_fields,
                                object_info,
                                ..
                            } => (expected_fields, excluded_fields, object_info),
                            _ => return Err(SystemPartitionCheckError::InvalidPartition),
                        };

                    match object_partition_descriptor {
                        ObjectPartitionDescriptor::Fields => {
                            for (field_index, value) in
                                reader.field_iter(&node_checker_state.node_id, partition_number)
                            {
                                expected_fields.remove(&(module_id, field_index));
                                if module_id.eq(&ModuleId::Main)
                                    && excluded_fields.contains(&field_index)
                                {
                                    return Err(
                                        SystemPartitionCheckError::ContainsFieldWhichShouldNotExist(
                                            object_info.blueprint_info.blueprint_id.clone(),
                                            node_checker_state.node_id,
                                            field_index,
                                        ),
                                    );
                                }

                                let field: FieldSubstate<ScryptoValue> = scrypto_decode(&value)
                                    .map_err(|_| SystemPartitionCheckError::InvalidFieldValue)?;
                                let field_payload = scrypto_encode(field.payload())
                                    .map_err(|_| SystemPartitionCheckError::InvalidFieldValue)?;

                                let field_identifier =
                                    BlueprintPayloadIdentifier::Field(field_index);
                                let field_schema = reader
                                    .get_blueprint_payload_schema(&type_target, &field_identifier)
                                    .map_err(SystemPartitionCheckError::MissingFieldSchema)?;

                                reader
                                    .validate_payload(
                                        &field_payload,
                                        &field_schema,
                                        BLUEPRINT_PAYLOAD_MAX_DEPTH,
                                    )
                                    .map_err(|_| SystemPartitionCheckError::InvalidFieldValue)?;

                                self.application_checker.on_field(
                                    object_info.blueprint_info.clone(),
                                    node_checker_state.node_id,
                                    module_id,
                                    field_index,
                                    &field_payload,
                                );

                                substate_count += 1;
                            }
                        }
                        ObjectPartitionDescriptor::IndexCollection(collection_index) => {
                            let key_schema = {
                                let key_identifier = BlueprintPayloadIdentifier::IndexEntry(
                                    collection_index,
                                    KeyOrValue::Key,
                                );
                                reader
                                    .get_blueprint_payload_schema(&type_target, &key_identifier)
                                    .map_err(
                                        SystemPartitionCheckError::MissingIndexCollectionKeySchema,
                                    )?
                            };

                            let value_schema = {
                                let value_identifier = BlueprintPayloadIdentifier::IndexEntry(
                                    collection_index,
                                    KeyOrValue::Value,
                                );
                                reader
                                    .get_blueprint_payload_schema(&type_target, &value_identifier)
                                    .map_err(SystemPartitionCheckError::MissingIndexCollectionValueSchema)?
                            };

                            for (map_key, value) in
                                reader.map_iter(&node_checker_state.node_id, partition_number)
                            {
                                // Key Check
                                let key = {
                                    reader
                                        .validate_payload(
                                            &map_key,
                                            &key_schema,
                                            BLUEPRINT_PAYLOAD_MAX_DEPTH,
                                        )
                                        .map_err(|_| {
                                            SystemPartitionCheckError::InvalidIndexCollectionKey
                                        })?;

                                    map_key
                                };

                                // Value Check
                                let value = {
                                    let entry: IndexEntrySubstate<ScryptoValue> =
                                        scrypto_decode(&value).map_err(|_| {
                                            SystemPartitionCheckError::InvalidIndexCollectionValue
                                        })?;
                                    let value = scrypto_encode(entry.value()).map_err(|_| {
                                        SystemPartitionCheckError::InvalidIndexCollectionValue
                                    })?;

                                    reader
                                        .validate_payload(
                                            &value,
                                            &value_schema,
                                            BLUEPRINT_PAYLOAD_MAX_DEPTH,
                                        )
                                        .map_err(|_| {
                                            SystemPartitionCheckError::InvalidIndexCollectionValue
                                        })?;

                                    value
                                };

                                self.application_checker.on_collection_entry(
                                    object_info.blueprint_info.clone(),
                                    node_checker_state.node_id,
                                    module_id,
                                    collection_index,
                                    &key,
                                    &value,
                                );

                                substate_count += 1;
                            }
                        }

                        ObjectPartitionDescriptor::KeyValueCollection(collection_index) => {
                            let key_identifier = BlueprintPayloadIdentifier::KeyValueEntry(
                                collection_index,
                                KeyOrValue::Key,
                            );

                            let key_schema = reader
                                .get_blueprint_payload_schema(&type_target, &key_identifier)
                                .map_err(
                                    SystemPartitionCheckError::MissingKeyValueCollectionKeySchema,
                                )?;

                            let value_schema = {
                                let value_identifier = BlueprintPayloadIdentifier::KeyValueEntry(
                                    collection_index,
                                    KeyOrValue::Value,
                                );
                                reader
                                    .get_blueprint_payload_schema(&type_target, &value_identifier)
                                    .map_err(SystemPartitionCheckError::MissingKeyValueCollectionValueSchema)?
                            };

                            for (map_key, value) in
                                reader.map_iter(&node_checker_state.node_id, partition_number)
                            {
                                // Key check
                                let key = {
                                    reader
                                        .validate_payload(
                                            &map_key,
                                            &key_schema,
                                            BLUEPRINT_PAYLOAD_MAX_DEPTH,
                                        )
                                        .map_err(|_| {
                                            SystemPartitionCheckError::FailedBlueprintSchemaCheck(
                                                key_identifier.clone(),
                                            )
                                        })?;

                                    map_key
                                };

                                // Value check
                                {
                                    let entry: KeyValueEntrySubstate<ScryptoValue> = scrypto_decode(&value)
                                        .map_err(|_| SystemPartitionCheckError::InvalidKeyValueCollectionValue)?;
                                    if let Some(value) = entry.into_value() {
                                        let entry_payload = scrypto_encode(&value)
                                            .map_err(|_| SystemPartitionCheckError::InvalidKeyValueCollectionValue)?;
                                        reader.validate_payload(&entry_payload, &value_schema, BLUEPRINT_PAYLOAD_MAX_DEPTH)
                                            .map_err(|_| SystemPartitionCheckError::InvalidKeyValueCollectionValue)?;

                                        self.application_checker.on_collection_entry(
                                            object_info.blueprint_info.clone(),
                                            node_checker_state.node_id,
                                            module_id,
                                            collection_index,
                                            &key,
                                            &entry_payload,
                                        )
                                    }
                                }

                                substate_count += 1;
                            }
                        }
                        ObjectPartitionDescriptor::SortedIndexCollection(collection_index) => {
                            let key_schema = {
                                let key_identifier = BlueprintPayloadIdentifier::SortedIndexEntry(
                                    collection_index,
                                    KeyOrValue::Key,
                                );
                                reader
                                    .get_blueprint_payload_schema(&type_target, &key_identifier)
                                    .map_err(SystemPartitionCheckError::MissingSortedIndexCollectionKeySchema)?
                            };

                            let value_schema = {
                                let value_identifier = BlueprintPayloadIdentifier::SortedIndexEntry(
                                    collection_index,
                                    KeyOrValue::Value,
                                );
                                reader
                                    .get_blueprint_payload_schema(&type_target, &value_identifier)
                                    .map_err(SystemPartitionCheckError::MissingSortedIndexCollectionValueSchema)?
                            };

                            for (sorted_key, value) in
                                reader.sorted_iter(&node_checker_state.node_id, partition_number)
                            {
                                // Key Check
                                let key = {
                                    reader.validate_payload(
                                        &sorted_key.1,
                                        &key_schema,
                                        BLUEPRINT_PAYLOAD_MAX_DEPTH,
                                    )
                                    .map_err(|_| {
                                        SystemPartitionCheckError::InvalidSortedIndexCollectionKey
                                    })?;

                                    sorted_key.1
                                };

                                // Value Check
                                let value = {
                                    let entry: SortedIndexEntrySubstate<ScryptoValue> = scrypto_decode(&value)
                                        .map_err(|_| SystemPartitionCheckError::InvalidSortedIndexCollectionValue)?;
                                    let value = scrypto_encode(entry.value()).map_err(|_| {
                                        SystemPartitionCheckError::InvalidSortedIndexCollectionValue
                                    })?;

                                    reader.validate_payload(
                                        &value,
                                        &value_schema,
                                        BLUEPRINT_PAYLOAD_MAX_DEPTH,
                                    )
                                    .map_err(|_| {
                                        SystemPartitionCheckError::InvalidSortedIndexCollectionValue
                                    })?;

                                    value
                                };

                                self.application_checker.on_collection_entry(
                                    object_info.blueprint_info.clone(),
                                    node_checker_state.node_id,
                                    module_id,
                                    collection_index,
                                    &key,
                                    &value,
                                );

                                substate_count += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(SystemPartitionCheckResults { substate_count })
    }
}

/// Defines a composite application database checker that includes multiple other application
/// database checkers.
///
/// This macro can be invoked as follows:
///
/// ```ignore
/// define_composite_checker! {
///     CheckerIdent,
///     [
///         ResourceDatabaseChecker,
///         RoleAssignmentDatabaseChecker
///     ]
/// }
/// ```
///
/// The above macro invocation will create a struct with the given ident which implements
/// [`ApplicationChecker`]. Whenever one of the [`ApplicationChecker`] methods are called the data
/// is passed to the [`ApplicationChecker`] implementation of the child checkers.
#[macro_export]
macro_rules! define_composite_checker {
    (
        $ident: ident,
        [
            $($ty: ident $(< $( $generic_ident: ident: $generic_type: ty ),* $(,)? >)?),* $(,)?
        ] $(,)?
    ) => {
        paste::paste! {
            #[derive(Debug)]
            pub struct $ident<
            $(
                $($($generic_ident: $generic_type),*)?
            )*
            > {
                $(
                    pub [< $ty: snake >]: $ty $(< $($generic_ident),* >)?,
                )*
            }

            const _: () = {
                impl<
                $(
                    $($($generic_ident: $generic_type),*)?
                )*
                > $ident<
                $(
                    $($($generic_ident),*)?
                )*
                > {
                    pub fn new(
                        $(
                            [< $ty: snake >]: $ty $(< $($generic_ident),* >)?,
                        )*
                    ) -> Self {
                        Self {
                            $(
                                [< $ty: snake >],
                            )*
                        }
                    }
                }

                impl<
                $(
                    $($($generic_ident: $generic_type),*)?
                )* > $crate::system::checkers::ApplicationChecker for $ident<
                $(
                    $($($generic_ident),*)?
                )*
                > {
                    type ApplicationCheckerResults = (
                        $(
                            < $ty $(::< $($generic_ident),* >)? as $crate::system::checkers::ApplicationChecker >::ApplicationCheckerResults,
                        )*
                    );

                    fn on_field(
                        &mut self,
                        info: BlueprintInfo,
                        node_id: NodeId,
                        module_id: ModuleId,
                        field_index: FieldIndex,
                        value: &Vec<u8>,
                    ) {
                        $(
                            $crate::system::checkers::ApplicationChecker::on_field(
                                &mut self. [< $ty: snake >],
                                info.clone(),
                                node_id,
                                module_id,
                                field_index,
                                value,
                            );
                        )*
                    }

                    fn on_collection_entry(
                        &mut self,
                        info: BlueprintInfo,
                        node_id: NodeId,
                        module_id: ModuleId,
                        collection_index: CollectionIndex,
                        key: &Vec<u8>,
                        value: &Vec<u8>,
                    ) {
                        $(
                            $crate::system::checkers::ApplicationChecker::on_collection_entry(
                                &mut self. [< $ty: snake >],
                                info.clone(),
                                node_id,
                                module_id,
                                collection_index,
                                key,
                                value,
                            );
                        )*
                    }

                    fn on_finish(&self) -> Self::ApplicationCheckerResults {
                        (
                            $(
                                $crate::system::checkers::ApplicationChecker::on_finish(
                                    &self. [< $ty: snake >],
                                ),
                            )*
                        )
                    }
                }
            };
        }
    };
}
