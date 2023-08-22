use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::payload_validation::{SchemaOrigin, TypeInfoForValidation, ValidationContext};
use crate::system::system::{FieldSubstate, KeyValueEntrySubstate};
use radix_engine_common::prelude::{
    scrypto_decode, scrypto_encode, Hash, ScryptoCustomExtension, ScryptoSchema, ScryptoValue,
};
use radix_engine_interface::api::{FieldIndex, ObjectModuleId};
use radix_engine_interface::blueprints::package::{
    BlueprintDefinition, BlueprintPayloadIdentifier, BlueprintType, KeyOrValue,
};
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use radix_engine_store_interface::interface::ListableSubstateDatabase;
use radix_engine_store_interface::interface::SubstateDatabase;
use sbor::rust::prelude::*;
use sbor::{validate_payload_against_schema, LocatedValidationError};

use crate::system::system_db_reader::{
    ObjectPartitionDescriptor, ResolvedPayloadSchema, SystemDatabaseReader,
    SystemPartitionDescriptor, SystemReaderError,
};
use crate::types::Condition;

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
        expected_fields: BTreeSet<(ObjectModuleId, FieldIndex)>,
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
    InvalidFieldKey,
    ContainsFieldWhichShouldNotExist,
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
    MissingSortedIndexCollectionKeySchema(SystemReaderError),
    MissingSortedIndexCollectionValueSchema(SystemReaderError),
    InvalidSortedIndexCollectionKey,
    InvalidSortedIndexCollectionValue,
    MissingObjectTypeTarget(SystemReaderError),
    InvalidTypeInfoKey,
    InvalidTypeInfoValue,
    InvalidSchemaKey,
    InvalidSchemaValue,
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
    TransientObjectFound,
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

pub struct SystemDatabaseChecker;

impl SystemDatabaseChecker {
    pub fn new() -> Self {
        SystemDatabaseChecker {}
    }

    pub fn check_db<S: SubstateDatabase + ListableSubstateDatabase>(
        &self,
        substate_db: &S,
    ) -> Result<SystemDatabaseCheckerResults, SystemDatabaseCheckError> {
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

        Ok(SystemDatabaseCheckerResults {
            node_counts,
            partition_count,
            substate_count,
        })
    }

    fn check_node<S: SubstateDatabase + ListableSubstateDatabase>(
        &self,
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
                    return Err(SystemNodeCheckError::TransientObjectFound);
                }

                let mut expected_fields = BTreeSet::new();
                let mut excluded_fields = BTreeSet::new();

                for (module_id, _version) in &object_info.module_versions {
                    match module_id {
                        ObjectModuleId::Main => {
                            if let Some((_, fields)) = &bp_definition.interface.state.fields {
                                for (field_index, field_schema) in fields.iter().enumerate() {
                                    match &field_schema.condition {
                                        Condition::Always => {
                                            expected_fields.insert((*module_id, field_index as u8));
                                        }
                                        Condition::IfFeature(feature) => {
                                            if object_info
                                                .blueprint_info
                                                .features
                                                .contains(feature.as_str())
                                            {
                                                expected_fields
                                                    .insert((*module_id, field_index as u8));
                                            } else {
                                                excluded_fields.insert(field_index as u8);
                                            }
                                        }
                                        Condition::IfOuterFeature(feature) => {
                                            if outer_object
                                                .as_ref()
                                                .ok_or_else(|| {
                                                    SystemNodeCheckError::InvalidCondition
                                                })?
                                                .blueprint_info
                                                .features
                                                .contains(feature.as_str())
                                            {
                                                expected_fields
                                                    .insert((*module_id, field_index as u8));
                                            } else {
                                                excluded_fields.insert(field_index as u8);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            let blueprint_id = module_id.static_blueprint().unwrap();
                            let module_def = reader
                                .get_blueprint_definition(&blueprint_id)
                                .map_err(SystemNodeCheckError::MissingBlueprint)?;
                            if let Some((_, fields)) = &module_def.interface.state.fields {
                                for (field_index, field_schema) in fields.iter().enumerate() {
                                    match &field_schema.condition {
                                        Condition::Always => {
                                            expected_fields.insert((*module_id, field_index as u8));
                                        }
                                        _ => {
                                            return Err(SystemNodeCheckError::FoundModuleWithConditionalFields);
                                        }
                                    }
                                }
                            }
                        }
                    };
                }

                SystemNodeCheckerState {
                    node_id: *node_id,
                    node_type: SystemNodeType::Object {
                        object_info,
                        bp_definition,
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
            if object_info.global {
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
        &self,
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
                SystemPartitionDescriptor::TypeInfo => {
                    for (key, value) in reader
                        .substates_iter::<FieldKey>(&node_checker_state.node_id, partition_number)
                    {
                        match key {
                            SubstateKey::Field(0u8) => {}
                            _ => return Err(SystemPartitionCheckError::InvalidTypeInfoKey),
                        };

                        let _type_info: TypeInfoSubstate = scrypto_decode(&value)
                            .map_err(|_| SystemPartitionCheckError::InvalidTypeInfoValue)?;

                        substate_count += 1;
                    }
                }
                SystemPartitionDescriptor::Schema => {
                    for (key, value) in reader
                        .substates_iter::<MapKey>(&node_checker_state.node_id, partition_number)
                    {
                        let map_key = match key {
                            SubstateKey::Map(map_key) => map_key,
                            _ => return Err(SystemPartitionCheckError::InvalidSchemaKey),
                        };
                        let _schema_hash: Hash = scrypto_decode(&map_key)
                            .map_err(|_| SystemPartitionCheckError::InvalidSchemaKey)?;

                        let _schema: KeyValueEntrySubstate<ScryptoSchema> = scrypto_decode(&value)
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

                    for (key, value) in reader
                        .substates_iter::<MapKey>(&node_checker_state.node_id, partition_number)
                    {
                        // Key Check
                        {
                            let map_key = match key {
                                SubstateKey::Map(map_key) => map_key,
                                _ => {
                                    return Err(SystemPartitionCheckError::InvalidKeyValueStoreKey)
                                }
                            };
                            self.validate_payload(reader, &map_key, &key_schema)
                                .map_err(|_| SystemPartitionCheckError::InvalidKeyValueStoreKey)?;
                        }

                        // Value Check
                        {
                            let entry: KeyValueEntrySubstate<ScryptoValue> = scrypto_decode(&value)
                                .map_err(|_| {
                                    SystemPartitionCheckError::InvalidKeyValueStoreValue
                                })?;
                            if let Some(value) = entry.value {
                                let entry_payload = scrypto_encode(&value).map_err(|_| {
                                    SystemPartitionCheckError::InvalidKeyValueStoreValue
                                })?;
                                self.validate_payload(reader, &entry_payload, &value_schema)
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

                    match object_partition_descriptor {
                        ObjectPartitionDescriptor::Field => {
                            for (key, value) in reader.substates_iter::<FieldKey>(
                                &node_checker_state.node_id,
                                partition_number,
                            ) {
                                let field_index = match key {
                                    SubstateKey::Field(field_index) => field_index,
                                    _ => return Err(SystemPartitionCheckError::InvalidFieldKey),
                                };
                                match &mut node_checker_state.node_type {
                                    SystemNodeType::Object {
                                        expected_fields,
                                        excluded_fields,
                                        ..
                                    } => {
                                        expected_fields.remove(&(module_id, field_index));

                                        if module_id.eq(&ObjectModuleId::Main)
                                            && excluded_fields.contains(&field_index)
                                        {
                                            return Err(SystemPartitionCheckError::ContainsFieldWhichShouldNotExist);
                                        }
                                    }
                                    _ => return Err(SystemPartitionCheckError::InvalidFieldKey),
                                }

                                let field: FieldSubstate<ScryptoValue> = scrypto_decode(&value)
                                    .map_err(|_| SystemPartitionCheckError::InvalidFieldValue)?;
                                let field_payload = scrypto_encode(&field.value.0)
                                    .map_err(|_| SystemPartitionCheckError::InvalidFieldValue)?;

                                let field_identifier =
                                    BlueprintPayloadIdentifier::Field(field_index);
                                let field_schema = reader
                                    .get_blueprint_payload_schema(&type_target, &field_identifier)
                                    .map_err(SystemPartitionCheckError::MissingFieldSchema)?;

                                self.validate_payload(reader, &field_payload, &field_schema)
                                    .map_err(|_| SystemPartitionCheckError::InvalidFieldValue)?;

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

                            for (key, value) in reader.substates_iter::<MapKey>(
                                &node_checker_state.node_id,
                                partition_number,
                            ) {
                                let map_key =
                                    match key {
                                        SubstateKey::Map(map_key) => map_key,
                                        _ => return Err(
                                            SystemPartitionCheckError::InvalidIndexCollectionKey,
                                        ),
                                    };

                                self.validate_payload(reader, &map_key, &key_schema)
                                    .map_err(|_| {
                                        SystemPartitionCheckError::InvalidIndexCollectionKey
                                    })?;

                                self.validate_payload(reader, &value, &value_schema)
                                    .map_err(|_| {
                                        SystemPartitionCheckError::InvalidIndexCollectionValue
                                    })?;

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

                            for (key, value) in reader.substates_iter::<MapKey>(
                                &node_checker_state.node_id,
                                partition_number,
                            ) {
                                // Key check
                                {
                                    let map_key = match key {
                                        SubstateKey::Map(map_key) => map_key,
                                        _ => return Err(
                                            SystemPartitionCheckError::InvalidKeyValueCollectionKey,
                                        ),
                                    };
                                    self.validate_payload(reader, &map_key, &key_schema)
                                        .map_err(|_| {
                                            SystemPartitionCheckError::FailedBlueprintSchemaCheck(
                                                key_identifier.clone(),
                                            )
                                        })?;
                                }

                                // Value check
                                {
                                    let entry: KeyValueEntrySubstate<ScryptoValue> = scrypto_decode(&value)
                                        .map_err(|_| SystemPartitionCheckError::InvalidKeyValueCollectionValue)?;
                                    if let Some(value) = entry.value {
                                        let entry_payload = scrypto_encode(&value)
                                            .map_err(|_| SystemPartitionCheckError::InvalidKeyValueCollectionValue)?;
                                        self.validate_payload(reader, &entry_payload, &value_schema)
                                            .map_err(|_| SystemPartitionCheckError::InvalidKeyValueCollectionValue)?;
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

                            for (key, value) in reader.substates_iter::<SortedKey>(
                                &node_checker_state.node_id,
                                partition_number,
                            ) {
                                let sorted_key = match key {
                                    SubstateKey::Sorted(sorted_key) => sorted_key,
                                    _ => return Err(
                                        SystemPartitionCheckError::InvalidSortedIndexCollectionKey,
                                    ),
                                };

                                self.validate_payload(reader, &sorted_key.1, &key_schema)
                                    .map_err(|_| {
                                        SystemPartitionCheckError::InvalidSortedIndexCollectionKey
                                    })?;

                                self.validate_payload(reader, &value, &value_schema)
                                    .map_err(|_| {
                                        SystemPartitionCheckError::InvalidSortedIndexCollectionValue
                                    })?;

                                substate_count += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(SystemPartitionCheckResults { substate_count })
    }

    fn validate_payload<'a, S: SubstateDatabase + ListableSubstateDatabase>(
        &'a self,
        reader: &SystemDatabaseReader<S>,
        payload: &[u8],
        payload_schema: &'a ResolvedPayloadSchema,
    ) -> Result<(), LocatedValidationError<ScryptoCustomExtension>> {
        let validation_context: Box<dyn ValidationContext<Error = String>> =
            Box::new(ValidationPayloadCheckerContext {
                reader,
                schema_origin: payload_schema.schema_origin.clone(),
                allow_ownership: payload_schema.allow_ownership,
                allow_non_global_ref: payload_schema.allow_non_global_refs,
            });

        validate_payload_against_schema::<ScryptoCustomExtension, _>(
            payload,
            &payload_schema.schema,
            payload_schema.type_index,
            &validation_context,
        )
    }
}

struct ValidationPayloadCheckerContext<'a, S: SubstateDatabase> {
    reader: &'a SystemDatabaseReader<'a, S>,
    schema_origin: SchemaOrigin,
    allow_non_global_ref: bool,
    allow_ownership: bool,
}

impl<'a, S: SubstateDatabase> ValidationContext for ValidationPayloadCheckerContext<'a, S> {
    type Error = String;

    fn get_node_type_info(&self, node_id: &NodeId) -> Result<TypeInfoForValidation, String> {
        let type_info = self
            .reader
            .get_type_info(node_id)
            .map_err(|_| "Type Info missing".to_string())?;
        let type_info_for_validation = match type_info {
            TypeInfoSubstate::Object(object_info) => TypeInfoForValidation::Object {
                package: object_info.blueprint_info.blueprint_id.package_address,
                blueprint: object_info.blueprint_info.blueprint_id.blueprint_name,
            },
            TypeInfoSubstate::KeyValueStore(..) => TypeInfoForValidation::KeyValueStore,
            TypeInfoSubstate::GlobalAddressReservation(..) => {
                TypeInfoForValidation::GlobalAddressReservation
            }
            TypeInfoSubstate::GlobalAddressPhantom(..) => {
                return Err("Found invalid stored address phantom".to_string())
            }
        };

        Ok(type_info_for_validation)
    }

    fn schema_origin(&self) -> &SchemaOrigin {
        &self.schema_origin
    }

    fn allow_ownership(&self) -> bool {
        self.allow_ownership
    }

    fn allow_non_global_ref(&self) -> bool {
        self.allow_non_global_ref
    }
}
