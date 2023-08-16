use crate::errors::RuntimeError;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::payload_validation::{SchemaOrigin, TypeInfoForValidation, ValidationContext};
use crate::system::system::{FieldSubstate, KeyValueEntrySubstate};
use radix_engine_common::prelude::{scrypto_decode, scrypto_encode, ScryptoCustomExtension, ScryptoSchema, ScryptoValue};
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
    SystemPartitionDescriptor,
};
use crate::types::Condition;

#[derive(Debug)]
pub enum NodeCheckState {
    Object {
        object_info: ObjectInfo,
        bp_definition: BlueprintDefinition,
        expected_fields: BTreeSet<(ObjectModuleId, FieldIndex)>,
        excluded_fields: BTreeSet<FieldIndex>,
    },
    KeyValueStore {
        kv_info: KeyValueStoreInfo,
    },
}

impl NodeCheckState {
    pub fn finish(&self) {
        match self {
            NodeCheckState::Object {
                expected_fields, ..
            } => {
                if !expected_fields.is_empty() {
                    panic!("Missing expected fields: {:?}", expected_fields);
                }
            }
            NodeCheckState::KeyValueStore { .. } => {}
        }
    }
}

#[derive(Debug)]
pub struct SystemDatabaseCheckerResults {
    pub global_node_count: usize,
    pub interior_node_count: usize,
    pub package_count: usize,
    pub blueprint_count: usize,

    pub node_count: usize,
    pub partition_count: usize,
    pub substate_count: usize,
}

pub struct SystemDatabaseChecker;

impl SystemDatabaseChecker {
    pub fn new() -> Self {
        SystemDatabaseChecker {}
    }

    pub fn check_db<S: SubstateDatabase + ListableSubstateDatabase>(
        &self,
        substate_db: &S,
    ) -> SystemDatabaseCheckerResults {
        let mut global_node_count = 0usize;
        let mut interior_node_count = 0usize;
        let mut package_count = 0usize;
        let mut blueprint_count = 0usize;
        let mut node_count = 0usize;
        let mut partition_count = 0usize;
        let mut substate_count = 0usize;
        let mut last_node: Option<(NodeId, NodeCheckState)> = None;

        let reader = SystemDatabaseReader::new(substate_db);
        for (node_id, partition_number) in reader.partitions_iter() {
            let new_node = match &mut last_node {
                Some(last_info) => {
                    if node_id.ne(&last_info.0) {
                        None
                    } else {
                        Some(last_info)
                    }
                }
                None => None,
            };

            let node_check_state = match new_node {
                None => {
                    if let Some((node_id, finished_node)) = &last_node {
                        finished_node.finish();
                    }

                    if node_id.is_global_package() {
                        package_count += 1;
                        let definition =
                            reader.get_package_definition(PackageAddress::new_or_panic(node_id.0));
                        blueprint_count += definition.len();
                    }

                    let new_node_check_state = self.check_node(&reader, &node_id);
                    if let NodeCheckState::Object { object_info, .. } = &new_node_check_state {
                        if object_info.global {
                            global_node_count += 1;
                        } else {
                            interior_node_count += 1;
                        }
                    } else {
                        interior_node_count += 1;
                    }

                    node_count += 1;
                    last_node = Some((node_id, new_node_check_state));

                    &mut last_node.as_mut().unwrap().1
                }
                Some((_, stored_type_info)) => stored_type_info,
            };

            let partition_substate_count =
                self.check_partition(&reader, node_check_state, &node_id, partition_number);

            substate_count += partition_substate_count;
            partition_count += 1;
        }

        if let Some((_, finished_node)) = &last_node {
            finished_node.finish();
        }

        SystemDatabaseCheckerResults {
            global_node_count,
            interior_node_count,
            package_count,
            blueprint_count,

            node_count,
            partition_count,
            substate_count,
        }
    }

    fn check_node<S: SubstateDatabase + ListableSubstateDatabase>(
        &self,
        reader: &SystemDatabaseReader<S>,
        node_id: &NodeId,
    ) -> NodeCheckState {
        let type_info = reader
            .get_type_info(node_id)
            .expect("All existing nodes must have a type info");
        let _entity_type = node_id
            .entity_type()
            .expect("All existing nodes should have a matching entity type");
        let stored_type_info = match type_info {
            TypeInfoSubstate::Object(object_info) => {
                let bp_definition = reader
                    .get_blueprint_definition(&object_info.blueprint_info.blueprint_id)
                    .expect("Missing blueprint");

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
                            .expect("Missing outer object");
                        assert_eq!(
                            outer_object_info.blueprint_info.blueprint_id, expected_outer_blueprint,
                            "Invalid outer object type"
                        );
                        Some(outer_object_info)
                    }
                    _ => {
                        panic!("Invalid outer object type");
                    }
                };

                if bp_definition.interface.is_transient {
                    panic!("Transient object found.");
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
                                                .expect("Invalid condition")
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
                                .expect("Missing blueprint");
                            if let Some((_, fields)) = &module_def.interface.state.fields {
                                for (field_index, field_schema) in fields.iter().enumerate() {
                                    match &field_schema.condition {
                                        Condition::Always => {
                                            expected_fields.insert((*module_id, field_index as u8));
                                        }
                                        _ => {
                                            panic!("Modules should not have conditional fields")
                                        }
                                    }
                                }
                            }
                        }
                    };
                }

                NodeCheckState::Object {
                    object_info,
                    bp_definition,
                    expected_fields,
                    excluded_fields,
                }
            }
            TypeInfoSubstate::KeyValueStore(kv_store_info) => NodeCheckState::KeyValueStore {
                kv_info: kv_store_info,
            },
            TypeInfoSubstate::GlobalAddressPhantom(..) => {
                panic!("Global Address Phantom should never be stored");
            }
            TypeInfoSubstate::GlobalAddressReservation(..) => {
                panic!("Global Address Reservation should never be stored");
            }
        };

        stored_type_info
    }

    fn check_partition<S: SubstateDatabase + ListableSubstateDatabase>(
        &self,
        reader: &SystemDatabaseReader<S>,
        node_check_state: &mut NodeCheckState,
        node_id: &NodeId,
        partition_number: PartitionNumber,
    ) -> usize {
        let partition_descriptors = reader.get_partition_descriptors(node_id, &partition_number);
        assert!(
            !partition_descriptors.is_empty(),
            "Partition does not describe anything about object"
        );

        let mut substate_count = 0;

        match partition_descriptors[0] {
            SystemPartitionDescriptor::KeyValueStore => {
                let type_target = reader.get_kv_store_type_target(node_id).expect("Missing kv store type target");
                let key_schema = reader.get_kv_store_payload_schema(&type_target, KeyOrValue::Key).expect("Missing key schema");
                let value_schema = reader.get_kv_store_payload_schema(&type_target, KeyOrValue::Value).expect("Missing value schema");

                for (key, value) in reader.substates_iter::<MapKey>(node_id, partition_number) {
                    let map_key = match key {
                        SubstateKey::Map(map_key) => map_key,
                        _ => panic!("Invalid map key"),
                    };

                    self.validate_payload(reader, &map_key, &key_schema)
                        .expect("Invalid Key.");

                    let entry: KeyValueEntrySubstate<ScryptoValue> = scrypto_decode(&value).expect("Invalid KV Value");

                    if let Some(value) = entry.value {
                        let entry_payload = scrypto_encode(&value).unwrap();
                        self.validate_payload(reader, &entry_payload, &value_schema)
                            .expect("Invalid Value.");
                    }

                    substate_count += 1;
                }
            }
            SystemPartitionDescriptor::Object(
                module_id,
                ObjectPartitionDescriptor::IndexCollection(collection_index),
            ) => {
                let type_target = reader
                    .get_blueprint_type_target(node_id, module_id)
                    .expect("Missing type target");

                let key_schema = {
                    let key_identifier =
                        BlueprintPayloadIdentifier::IndexEntry(collection_index, KeyOrValue::Key);
                    reader
                        .get_blueprint_payload_schema(&type_target, &key_identifier)
                        .expect("Missing key schema")
                };

                let value_schema = {
                    let value_identifier =
                        BlueprintPayloadIdentifier::IndexEntry(collection_index, KeyOrValue::Value);
                    reader
                        .get_blueprint_payload_schema(&type_target, &value_identifier)
                        .expect("Missing value schema")
                };

                for (key, value) in reader.substates_iter::<MapKey>(node_id, partition_number) {
                    let map_key = match key {
                        SubstateKey::Map(map_key) => map_key,
                        _ => panic!("Invalid map key"),
                    };

                    self.validate_payload(reader, &map_key, &key_schema)
                        .expect("Invalid Key.");

                    self.validate_payload(reader, &value, &value_schema)
                        .expect("Invalid Value.");

                    substate_count += 1;
                }
            }
            SystemPartitionDescriptor::Object(
                module_id,
                ObjectPartitionDescriptor::KeyValueCollection(collection_index),
            ) => {
                let type_target = reader
                    .get_blueprint_type_target(node_id, module_id)
                    .expect("Missing type target");

                let key_schema = {
                    let key_identifier =
                        BlueprintPayloadIdentifier::KeyValueEntry(collection_index, KeyOrValue::Key);
                    reader
                        .get_blueprint_payload_schema(&type_target, &key_identifier)
                        .expect("Missing key schema")
                };

                let value_schema = {
                    let value_identifier =
                        BlueprintPayloadIdentifier::KeyValueEntry(collection_index, KeyOrValue::Value);
                    reader
                        .get_blueprint_payload_schema(&type_target, &value_identifier)
                        .expect("Missing value schema")
                };

                for (key, value) in reader.substates_iter::<MapKey>(node_id, partition_number) {
                    let map_key = match key {
                        SubstateKey::Map(map_key) => map_key,
                        _ => panic!("Invalid map key"),
                    };

                    self.validate_payload(reader, &map_key, &key_schema)
                        .expect("Invalid Key.");

                    let entry: KeyValueEntrySubstate<ScryptoValue> = scrypto_decode(&value).expect("Invalid KV Value");

                    if let Some(value) = entry.value {
                        let entry_payload = scrypto_encode(&value).unwrap();
                        self.validate_payload(reader, &entry_payload, &value_schema)
                            .expect("Invalid Value.");
                    }

                    substate_count += 1;
                }
            }
            SystemPartitionDescriptor::Object(
                module_id,
                ObjectPartitionDescriptor::SortedIndexCollection(collection_index),
            ) => {
                let type_target = reader
                    .get_blueprint_type_target(node_id, module_id)
                    .expect("Missing type target");

                let key_schema = {
                    let key_identifier =
                        BlueprintPayloadIdentifier::SortedIndexEntry(collection_index, KeyOrValue::Key);
                    reader
                        .get_blueprint_payload_schema(&type_target, &key_identifier)
                        .expect("Missing key schema")
                };

                let value_schema = {
                    let value_identifier = BlueprintPayloadIdentifier::SortedIndexEntry(
                        collection_index,
                        KeyOrValue::Value,
                    );
                    reader
                        .get_blueprint_payload_schema(&type_target, &value_identifier)
                        .expect("Missing value schema")
                };

                for (key, value) in reader.substates_iter::<SortedU16Key>(node_id, partition_number)
                {
                    let sorted_key = match key {
                        SubstateKey::Sorted(sorted_key) => sorted_key,
                        _ => panic!("Invalid sorted key"),
                    };

                    self.validate_payload(reader, &sorted_key.1, &key_schema)
                        .expect("Invalid Key.");

                    self.validate_payload(reader, &value, &value_schema)
                        .expect("Invalid Value.");

                    substate_count += 1;
                }
            }
            SystemPartitionDescriptor::Object(module_id, ObjectPartitionDescriptor::Field) => {
                let type_target = reader
                    .get_blueprint_type_target(node_id, module_id)
                    .expect("Missing type target");

                for (key, value) in reader.substates_iter::<FieldKey>(node_id, partition_number) {
                    let field_index = match key {
                        SubstateKey::Field(field_index) => field_index,
                        _ => panic!("Invalid Field key"),
                    };
                    match &mut *node_check_state {
                        NodeCheckState::Object {
                            expected_fields,
                            excluded_fields,
                            ..
                        } => {
                            expected_fields.remove(&(module_id, field_index));

                            if module_id.eq(&ObjectModuleId::Main)
                                && excluded_fields.contains(&field_index)
                            {
                                panic!("Contains field which should not exist");
                            }
                        }
                        _ => panic!("Invalid Field key"),
                    }

                    let field: FieldSubstate<ScryptoValue> = scrypto_decode(&value).expect("Invalid Field Value");
                    let field_payload = scrypto_encode(&field.value.0).unwrap();

                    let field_identifier = BlueprintPayloadIdentifier::Field(field_index);
                    let field_schema = reader
                        .get_blueprint_payload_schema(&type_target, &field_identifier)
                        .expect("Missing field schema");

                    self.validate_payload(reader, &field_payload, &field_schema)
                        .expect("Invalid Field.");

                    substate_count += 1;
                }
            }
            SystemPartitionDescriptor::TypeInfo => {
                for (key, value) in reader.substates_iter::<FieldKey>(node_id, partition_number) {
                    match key {
                        SubstateKey::Field(0u8) => {}
                        _ => panic!("Invalid TypeInfo key"),
                    };

                    let _type_info: TypeInfoSubstate =
                        scrypto_decode(&value).expect("Invalid Type Info");

                    substate_count += 1;
                }
            }
            SystemPartitionDescriptor::Schema => {
                for (key, value) in reader.substates_iter::<MapKey>(node_id, partition_number) {
                    let _map_key = match key {
                        SubstateKey::Map(map_key) => map_key,
                        _ => panic!("Invalid Schema key"),
                    };

                    let _schema: KeyValueEntrySubstate<ScryptoSchema> =
                        scrypto_decode(&value).expect("Invalid Schema");

                    substate_count += 1;
                }
            }
        }

        substate_count
    }

    fn validate_payload<'a, S: SubstateDatabase + ListableSubstateDatabase>(
        &'a self,
        reader: &SystemDatabaseReader<S>,
        payload: &[u8],
        payload_schema: &'a ResolvedPayloadSchema,
    ) -> Result<(), LocatedValidationError<ScryptoCustomExtension>> {
        let validation_context: Box<dyn ValidationContext> =
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
    fn get_node_type_info(&self, node_id: &NodeId) -> Result<TypeInfoForValidation, RuntimeError> {
        let type_info = self
            .reader
            .get_type_info(node_id)
            .expect("Type Info missing");
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
                panic!("Found invalid stored address phantom")
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
