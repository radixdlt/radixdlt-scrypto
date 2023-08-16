use radix_engine_common::prelude::{scrypto_decode, ScryptoSchema};
use radix_engine_interface::api::{FieldIndex, ObjectModuleId};
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use radix_engine_interface::blueprints::package::{BlueprintDefinition, BlueprintPayloadIdentifier, BlueprintType, KeyOrValue};
use radix_engine_store_interface::{
    interface::SubstateDatabase,
};
use radix_engine_store_interface::interface::{DbPartitionKey, ListableSubstateDatabase};
use sbor::rust::prelude::*;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::system::KeyValueEntrySubstate;

use crate::system::system_db_reader::{ObjectPartitionDescriptor, SystemDatabaseReader, SystemPartitionDescriptor};
use crate::system::system_type_checker::BlueprintTypeTarget;
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

    pub fn check_db<S: SubstateDatabase + ListableSubstateDatabase>(&self, substate_db: &S) -> SystemDatabaseCheckerResults {
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
                },
                None => None,
            };

            let node_check_state = match new_node {
                None => {
                    if let Some((node_id, finished_node)) = &last_node {
                        finished_node.finish();
                    }

                    if node_id.is_global_package() {
                        package_count += 1;
                        let definition = reader.get_package_definition(PackageAddress::new_or_panic(node_id.0));
                        blueprint_count += definition.len();
                    }

                    let new_node_check_state = self.check_node(&reader, &node_id);
                    if let NodeCheckState::Object { object_info, ..} = &new_node_check_state {
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
                Some((_, stored_type_info)) => {
                    stored_type_info
                }
            };

            let partition_substate_count = self.check_partition(&reader, node_check_state, &node_id, partition_number);

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

    fn check_node<S: SubstateDatabase + ListableSubstateDatabase>(&self, reader: &SystemDatabaseReader<S>, node_id: &NodeId) -> NodeCheckState {
        let type_info = reader.get_type_info(node_id).expect("All existing nodes must have a type info");
        let _entity_type = node_id.entity_type().expect("All existing nodes should have a matching entity type");
        let stored_type_info = match type_info {
            TypeInfoSubstate::Object(object_info) => {
                let bp_definition = reader.get_blueprint_definition(&object_info.blueprint_info.blueprint_id).expect("Missing blueprint");

                let outer_object =
                match (&object_info.blueprint_info.outer_obj_info, &bp_definition.interface.blueprint_type) {
                    (OuterObjectInfo::None, BlueprintType::Outer) => None,
                    (OuterObjectInfo::Some {
                        outer_object
                    }, BlueprintType::Inner { outer_blueprint }) => {
                        let expected_outer_blueprint = BlueprintId::new(&object_info.blueprint_info.blueprint_id.package_address, outer_blueprint.as_str());
                        let outer_object_info = reader.get_object_info(*outer_object).expect("Missing outer object");
                        assert_eq!(outer_object_info.blueprint_info.blueprint_id, expected_outer_blueprint, "Invalid outer object type");
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
                                        },
                                        Condition::IfFeature(feature) => {
                                            if object_info.blueprint_info.features.contains(feature.as_str()) {
                                                expected_fields.insert((*module_id, field_index as u8));
                                            } else {
                                                excluded_fields.insert(field_index as u8);
                                            }
                                        }
                                        Condition::IfOuterFeature(feature)  => {
                                            if outer_object.as_ref().expect("Invalid condition").blueprint_info.features.contains(feature.as_str()) {
                                                expected_fields.insert((*module_id, field_index as u8));
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
                            let module_def = reader.get_blueprint_definition(&blueprint_id).expect("Missing blueprint");
                            if let Some((_, fields)) = &module_def.interface.state.fields {
                                for (field_index, field_schema) in fields.iter().enumerate() {
                                    match &field_schema.condition {
                                        Condition::Always => {
                                            expected_fields.insert((*module_id, field_index as u8));
                                        },
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
            TypeInfoSubstate::KeyValueStore(kv_store_info) => {
                NodeCheckState::KeyValueStore {
                    kv_info: kv_store_info,
                }
            }
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
        partition_number: PartitionNumber
    ) -> usize {
        let partition_descriptors = reader.get_partition_descriptors(node_id, &partition_number);
        assert!(!partition_descriptors.is_empty(), "Partition does not describe anything about object");

        let mut substate_count = 0;

        match partition_descriptors[0] {
            SystemPartitionDescriptor::KeyValueStore => {
                for (key, value) in reader.substates_iter::<MapKey>(node_id, partition_number) {
                    let _map_key = match key {
                        SubstateKey::Map(map_key) => map_key,
                        _ => panic!("Invalid map key"),
                    };

                    // TODO: Check against schema

                    substate_count += 1;
                }
            }
            SystemPartitionDescriptor::Object(module_id, ObjectPartitionDescriptor::IndexCollection(collection_index)) => {
                let type_target = reader.get_type_target(node_id, module_id).expect("Missing type target");
                let key_identifier = BlueprintPayloadIdentifier::IndexEntry(collection_index, KeyOrValue::Key);
                reader.get_payload_schema(&type_target, &key_identifier).expect("Missing key schema");
                let value_identifier = BlueprintPayloadIdentifier::IndexEntry(collection_index, KeyOrValue::Value);
                reader.get_payload_schema(&type_target, &value_identifier).expect("Missing value schema");

                for (key, value) in reader.substates_iter::<MapKey>(node_id, partition_number) {
                    let _map_key = match key {
                        SubstateKey::Map(map_key) => map_key,
                        _ => panic!("Invalid map key"),
                    };

                    // TODO: Check against schema

                    substate_count += 1;
                }
            }
            SystemPartitionDescriptor::Object(module_id, ObjectPartitionDescriptor::KeyValueCollection(collection_index)) => {
                let type_target = reader.get_type_target(node_id, module_id).expect("Missing type target");
                let key_identifier = BlueprintPayloadIdentifier::KeyValueEntry(collection_index, KeyOrValue::Key);
                reader.get_payload_schema(&type_target, &key_identifier).expect("Missing key schema");
                let value_identifier = BlueprintPayloadIdentifier::KeyValueEntry(collection_index, KeyOrValue::Value);
                reader.get_payload_schema(&type_target, &value_identifier).expect("Missing value schema");

                for (key, value) in reader.substates_iter::<MapKey>(node_id, partition_number) {
                    let _map_key = match key {
                        SubstateKey::Map(map_key) => map_key,
                        _ => panic!("Invalid map key"),
                    };

                    // TODO: Check against schema

                    substate_count += 1;
                }
            }
            SystemPartitionDescriptor::Object(module_id, ObjectPartitionDescriptor::SortedIndexCollection(collection_index)) => {
                let type_target = reader.get_type_target(node_id, module_id).expect("Missing type target");
                let key_identifier = BlueprintPayloadIdentifier::SortedIndexEntry(collection_index, KeyOrValue::Key);
                reader.get_payload_schema(&type_target, &key_identifier).expect("Missing key schema");
                let value_identifier = BlueprintPayloadIdentifier::SortedIndexEntry(collection_index, KeyOrValue::Value);
                reader.get_payload_schema(&type_target, &value_identifier).expect("Missing value schema");

                for (key, value) in reader.substates_iter::<SortedU16Key>(node_id, partition_number) {
                    let _sorted_key = match key {
                        SubstateKey::Sorted(sorted_key) => sorted_key,
                        _ => panic!("Invalid sorted key"),
                    };

                    // TODO: Check against schema

                    substate_count += 1;
                }
            }
            SystemPartitionDescriptor::Object(module_id, ObjectPartitionDescriptor::Field) => {
                let type_target = reader.get_type_target(node_id, module_id).expect("Missing type target");

                for (key, _value) in reader.substates_iter::<FieldKey>(node_id, partition_number) {
                    let field_index = match key {
                        SubstateKey::Field(field_index) => field_index,
                        _ => panic!("Invalid Field key"),
                    };
                    match &mut *node_check_state {
                        NodeCheckState::Object {
                            expected_fields, excluded_fields, ..
                        } => {
                            expected_fields.remove(&(module_id, field_index));

                            if module_id.eq(&ObjectModuleId::Main) && excluded_fields.contains(&field_index) {
                                panic!("Contains field which should not exist");
                            }
                        }
                        _ => panic!("Invalid Field key")
                    }


                    let field_identifier = BlueprintPayloadIdentifier::Field(field_index);
                    reader.get_payload_schema(&type_target, &field_identifier).expect("Missing field schema");

                    let blueprint_id = reader.get_blueprint_id(node_id, module_id).expect("Invalid module");
                    let bp_def = reader.get_blueprint_definition(&blueprint_id).expect("Missing definition");
                    let _payload_def = bp_def.interface.get_field_payload_def(field_index).expect("Invalid field");

                    // TODO: Check against schema

                    substate_count += 1;
                }
            }
            SystemPartitionDescriptor::TypeInfo => {
                for (key, value) in reader.substates_iter::<FieldKey>(node_id, partition_number) {
                    match key {
                        SubstateKey::Field(0u8) => {},
                        _ => panic!("Invalid TypeInfo key"),
                    };

                    let _type_info: TypeInfoSubstate = scrypto_decode(&value).expect("Invalid Type Info");

                    substate_count += 1;
                }
            }
            SystemPartitionDescriptor::Schema => {
                for (key, value) in reader.substates_iter::<MapKey>(node_id, partition_number) {
                    let _map_key = match key {
                        SubstateKey::Map(map_key) => map_key,
                        _ => panic!("Invalid Schema key"),
                    };

                    let schema: KeyValueEntrySubstate<ScryptoSchema> = scrypto_decode(&value).expect("Invalid Schema");

                    substate_count += 1;
                }
            }
        }

        substate_count
    }
}
