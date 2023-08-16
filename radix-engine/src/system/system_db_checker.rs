use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use radix_engine_interface::blueprints::package::{BlueprintDefinition, BlueprintType};
use radix_engine_store_interface::{
    db_key_mapper::{DatabaseKeyMapper, SpreadPrefixKeyMapper},
    interface::SubstateDatabase,
};
use radix_engine_store_interface::interface::{DbPartitionKey, ListableSubstateDatabase};
use sbor::rust::prelude::*;
use crate::system::node_modules::type_info::TypeInfoSubstate;

use crate::system::system_db_reader::{SystemDatabaseReader, SystemPartitionDescriptor};

#[derive(Debug)]
pub enum NodeCheckingObject {
    Object {
        object_info: ObjectInfo,
        bp_definition: BlueprintDefinition,
    },
    KeyValueStore {
        kv_info: KeyValueStoreInfo,
    },
}

/// A System Layer (Layer 2) abstraction over an underlying substate database
pub struct SystemDatabaseChecker;

impl SystemDatabaseChecker {
    pub fn new() -> Self {
        SystemDatabaseChecker {}
    }

    pub fn check_db<S: SubstateDatabase + ListableSubstateDatabase>(&self, substate_db: &S) {
        let mut node_count = 0;
        let mut partition_count = 0;
        let mut last_node: Option<(NodeId, NodeCheckingObject)> = None;

        for partition_key in substate_db.list_partition_keys() {
            let (node_id, partition_number) = SpreadPrefixKeyMapper::from_db_partition_key(&partition_key);

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

            let node_checking_object = match new_node {
                None => {
                    let stored_type_info = self.check_node(substate_db, &node_id);
                    node_count += 1;
                    last_node = Some((node_id, stored_type_info));

                    &mut last_node.as_mut().unwrap().1
                }
                Some((_, stored_type_info)) => {
                    stored_type_info
                }
            };

            self.check_partition(substate_db, node_checking_object, &node_id, partition_number);

            partition_count += 1;
        }

        println!("node_count: {}\npartition_count: {}\n", node_count, partition_count);
    }

    fn check_node<S: SubstateDatabase + ListableSubstateDatabase>(&self, substate_db: &S, node_id: &NodeId) -> NodeCheckingObject {
        let reader = SystemDatabaseReader::new(substate_db);
        let type_info = reader.get_type_info(node_id).expect("All existing nodes must have a type info");
        let _entity_type = node_id.entity_type().expect("All existing nodes should have a matching entity type");
        let stored_type_info = match type_info {
            TypeInfoSubstate::Object(object_info) => {
                let bp_definition = reader.get_blueprint_definition(&object_info.blueprint_info.blueprint_id).expect("Missing blueprint");

                match (&object_info.blueprint_info.outer_obj_info, &bp_definition.interface.blueprint_type) {
                    (OuterObjectInfo::None, BlueprintType::Outer) => {}
                    (OuterObjectInfo::Some {
                        outer_object
                    }, BlueprintType::Inner { outer_blueprint }) => {
                        let expected_outer_blueprint = BlueprintId::new(&object_info.blueprint_info.blueprint_id.package_address, outer_blueprint.as_str());
                        let outer_object_info = reader.get_object_info(*outer_object).expect("Missing outer object");
                        assert_eq!(outer_object_info.blueprint_info.blueprint_id, expected_outer_blueprint, "Invalid outer object type");
                    }
                    _ => {
                        panic!("Invalid outer object type");
                    }
                }

                NodeCheckingObject::Object {
                    object_info,
                    bp_definition,
                }
            }
            TypeInfoSubstate::KeyValueStore(kv_store_info) => {
                NodeCheckingObject::KeyValueStore {
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
        substate_db: &S,
        node_checking_object: &mut NodeCheckingObject,
        node_id: &NodeId,
        partition_number: PartitionNumber
    ) {
        let reader = SystemDatabaseReader::new(substate_db);
        let partition_descriptors = reader.get_partition_descriptors(node_id, &partition_number);
        assert!(!partition_descriptors.is_empty(), "Partition does not describe anything about object");
    }
}
