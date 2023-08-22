use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::types::*;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use radix_engine_store_interface::interface::SubstateDatabase;
use sbor::rust::prelude::*;

use crate::system::system_db_reader::{
    ObjectPartitionDescriptor, SystemDatabaseReader, SystemPartitionDescriptor,
};
use crate::system::system_type_checker::BlueprintTypeTarget;
use crate::track::{ReadOnly, SystemUpdates, TrackedNode, TrackedSubstateValue};

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub enum SubstateSystemStructure {
    SystemField(SystemFieldStructure),
    SystemSchema,
    // KeyValueStore substates
    KeyValueStoreEntry(KeyValueStoreEntryStructure),
    // Object substates
    ObjectField(FieldStructure),
    ObjectKeyValuePartitionEntry(KeyValuePartitionEntryStructure),
    ObjectIndexPartitionEntry(IndexPartitionEntryStructure),
    ObjectSortedIndexPartitionEntry(SortedIndexPartitionEntryStructure),
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct SystemFieldStructure {
    pub field_kind: SystemFieldKind,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub enum SystemFieldKind {
    TypeInfo,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct KeyValueStoreEntryStructure {
    pub key_value_store_address: InternalAddress,
    pub key_schema_hash: SchemaHash,
    pub key_local_type_index: LocalTypeIndex,
    pub value_schema_hash: SchemaHash,
    pub value_local_type_index: LocalTypeIndex,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct FieldStructure {
    pub value_schema: ObjectSubstateTypeReference,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct KeyValuePartitionEntryStructure {
    pub key_schema: ObjectSubstateTypeReference,
    pub value_schema: ObjectSubstateTypeReference,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct IndexPartitionEntryStructure {
    pub key_schema: ObjectSubstateTypeReference,
    pub value_schema: ObjectSubstateTypeReference,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct SortedIndexPartitionEntryStructure {
    pub key_schema: ObjectSubstateTypeReference,
    pub value_schema: ObjectSubstateTypeReference,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub enum ObjectSubstateTypeReference {
    Package(PackageTypeReference),
    ObjectInstance(ObjectInstanceTypeReference),
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct PackageTypeReference {
    pub package_address: PackageAddress,
    pub schema_hash: SchemaHash,
    pub local_type_index: LocalTypeIndex,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct ObjectInstanceTypeReference {
    pub entity_address: NodeId,
    pub schema_hash: SchemaHash,
    pub instance_type_index: u8,
    pub local_type_index: LocalTypeIndex,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct EventSystemStructure {
    pub package_type_reference: PackageTypeReference,
}

pub type SubstateSystemStructures =
    IndexMap<NodeId, IndexMap<PartitionNumber, IndexMap<SubstateKey, SubstateSystemStructure>>>;

#[derive(Default, Debug, Clone, ScryptoSbor)]
pub struct SystemStructure {
    pub substate_system_structures: SubstateSystemStructures,
    pub event_system_structures: IndexMap<EventTypeIdentifier, EventSystemStructure>,
}

impl SystemStructure {
    pub fn resolve<S: SubstateDatabase>(
        substate_db: &S,
        updates: &IndexMap<NodeId, TrackedNode>,
        application_events: &Vec<(EventTypeIdentifier, Vec<u8>)>,
    ) -> Self {
        let mut substate_schema_mapper =
            SubstateSchemaMapper::new(SystemDatabaseReader::new_with_overlay(substate_db, updates));
        substate_schema_mapper.add_all_written_substate_structures(updates);
        let substate_system_structures = substate_schema_mapper.done();

        let event_system_structures =
            EventSchemaMapper::new(substate_db, &updates, application_events).run();

        SystemStructure {
            substate_system_structures,
            event_system_structures,
        }
    }
}

/// A builder of [`SubstateSystemStructures`].
/// Note that the implementation below assumes that substate owned objects can not be
/// detached. If this changes, we will have to account for objects that are removed
/// from a substate.
pub struct SubstateSchemaMapper<'a, S: SubstateDatabase> {
    /// The source of type information.
    system_reader: SystemDatabaseReader<'a, S>,
    /// The result of the build.
    substate_structures: SubstateSystemStructures,
}

impl<'a, S: SubstateDatabase> SubstateSchemaMapper<'a, S> {
    /// Creates an empty builder.
    pub fn new(system_reader: SystemDatabaseReader<'a, S>) -> Self {
        Self {
            system_reader,
            substate_structures: index_map_new(),
        }
    }

    /// Resolves a [`SubstateSystemStructure`] of the given substate and adds it to the build.
    pub fn add_substate_structure(
        &mut self,
        node_id: &NodeId,
        partition_num: &PartitionNumber,
        key: &SubstateKey,
    ) {
        let partition_descriptors = self
            .system_reader
            .get_partition_descriptors(node_id, partition_num)
            .unwrap();
        let substate_structure =
            self.resolve_substate_structure(node_id, partition_descriptors, key);
        self.substate_structures
            .entry(node_id.clone())
            .or_insert_with(|| index_map_new())
            .entry(partition_num.clone())
            .or_insert_with(|| index_map_new())
            .insert(key.clone(), substate_structure);
    }

    /// A batch `add_substate_structure()` counterpart, tailored for processing all substates
    /// *written* to the track (i.e. skipping reads).
    pub fn add_all_written_substate_structures(&mut self, tracked: &IndexMap<NodeId, TrackedNode>) {
        for (node_id, tracked_node) in tracked {
            for (partition_num, tracked_partition) in &tracked_node.tracked_partitions {
                for (_, tracked_substate) in &tracked_partition.substates {
                    match &tracked_substate.substate_value {
                        TrackedSubstateValue::New(_)
                        | TrackedSubstateValue::ReadExistAndWrite(_, _)
                        | TrackedSubstateValue::ReadNonExistAndWrite(_)
                        | TrackedSubstateValue::WriteOnly(_) => {
                            // The substate has been written - so process this substate structure
                        }
                        TrackedSubstateValue::ReadOnly(ReadOnly::Existent(_))
                        | TrackedSubstateValue::ReadOnly(ReadOnly::NonExistent)
                        | TrackedSubstateValue::Garbage => {
                            // We don't process substates which were only read
                            // NOTE:
                            //   If in future we want to enable this for reads too, it should be possible to
                            //     enable this for TrackedSubstateValue::ReadOnly(ReadOnly::Existent(_))
                            //     but it is not possible for NonExistent reads.
                            //   If a transaction fails, it's possible to get reads of non-existent substates
                            //     where the type info can't be resolved below. For example, if boostrap fails,
                            //     consensus manager substates are read but the type info is not written.
                            continue;
                        }
                    }

                    self.add_substate_structure(
                        node_id,
                        partition_num,
                        &tracked_substate.substate_key,
                    );
                }
            }
        }
    }

    /// A batch `add_substate_structure()` counterpart, tailored for processing all substates
    /// captured in the given [`SystemUpdates`].
    pub fn add_all_system_updates(&mut self, updates: &SystemUpdates) {
        for ((node_id, partition_num), substate_updates) in updates {
            for substate_key in substate_updates.keys() {
                self.add_substate_structure(node_id, partition_num, substate_key);
            }
        }
    }

    /// Finalizes the build.
    pub fn done(self) -> SubstateSystemStructures {
        self.substate_structures
    }

    fn resolve_substate_structure(
        &self,
        node_id: &NodeId,
        partition_descriptors: Vec<SystemPartitionDescriptor>,
        key: &SubstateKey,
    ) -> SubstateSystemStructure {
        match &partition_descriptors[0] {
            SystemPartitionDescriptor::TypeInfo => {
                SubstateSystemStructure::SystemField(SystemFieldStructure {
                    field_kind: SystemFieldKind::TypeInfo,
                })
            }
            SystemPartitionDescriptor::Schema => SubstateSystemStructure::SystemSchema,
            SystemPartitionDescriptor::KeyValueStore => {
                let info = self
                    .system_reader
                    .get_kv_store_type_target(node_id)
                    .expect(&format!("Could not get type info for node {node_id:?}"));

                let key_type_id = match info.kv_store_type.key_generic_substitutions {
                    GenericSubstitution::Local(type_id) => type_id,
                };
                let value_type_id = match info.kv_store_type.value_generic_substitutions {
                    GenericSubstitution::Local(type_id) => type_id,
                };
                SubstateSystemStructure::KeyValueStoreEntry(KeyValueStoreEntryStructure {
                    key_value_store_address: (*node_id).try_into().unwrap(),
                    key_schema_hash: key_type_id.0,
                    key_local_type_index: key_type_id.1,
                    value_schema_hash: value_type_id.0,
                    value_local_type_index: value_type_id.1,
                })
            }
            SystemPartitionDescriptor::Object(module_id, object_partition_descriptor) => {
                let bp_type_target = self
                    .system_reader
                    .get_blueprint_type_target(node_id, *module_id)
                    .expect(&format!("Could not get type info for node {node_id:?}"));

                self.resolve_object_substate_structure(
                    &bp_type_target,
                    object_partition_descriptor,
                    key,
                )
            }
        }
    }

    fn resolve_object_substate_structure(
        &self,
        bp_type_target: &BlueprintTypeTarget,
        object_partition_desciptor: &ObjectPartitionDescriptor,
        key: &SubstateKey,
    ) -> SubstateSystemStructure {
        match object_partition_desciptor {
            ObjectPartitionDescriptor::Field => {
                let field_index = match key {
                    SubstateKey::Field(field_index) => field_index,
                    _ => panic!("Invalid field key"),
                };

                let payload_identifier = BlueprintPayloadIdentifier::Field(*field_index);
                let type_reference = self
                    .system_reader
                    .get_blueprint_payload_schema_pointer(&bp_type_target, &payload_identifier)
                    .expect("Could not resolve to type reference");
                return SubstateSystemStructure::ObjectField(FieldStructure {
                    value_schema: type_reference,
                });
            }

            ObjectPartitionDescriptor::KeyValueCollection(collection_index) => {
                let key_identifier =
                    BlueprintPayloadIdentifier::KeyValueEntry(*collection_index, KeyOrValue::Key);
                let value_identifier =
                    BlueprintPayloadIdentifier::KeyValueEntry(*collection_index, KeyOrValue::Value);
                let key_type_reference = self
                    .system_reader
                    .get_blueprint_payload_schema_pointer(&bp_type_target, &key_identifier)
                    .expect("Could not resolve to type reference");
                let value_type_reference = self
                    .system_reader
                    .get_blueprint_payload_schema_pointer(&bp_type_target, &value_identifier)
                    .expect("Could not resolve to type reference");
                SubstateSystemStructure::ObjectKeyValuePartitionEntry(
                    KeyValuePartitionEntryStructure {
                        key_schema: key_type_reference,
                        value_schema: value_type_reference,
                    },
                )
            }

            ObjectPartitionDescriptor::IndexCollection(collection_index) => {
                let key_identifier =
                    BlueprintPayloadIdentifier::IndexEntry(*collection_index, KeyOrValue::Key);
                let value_identifier =
                    BlueprintPayloadIdentifier::IndexEntry(*collection_index, KeyOrValue::Value);
                let key_type_reference = self
                    .system_reader
                    .get_blueprint_payload_schema_pointer(&bp_type_target, &key_identifier)
                    .expect("Could not resolve to type reference");
                let value_type_reference = self
                    .system_reader
                    .get_blueprint_payload_schema_pointer(&bp_type_target, &value_identifier)
                    .expect("Could not resolve to type reference");
                SubstateSystemStructure::ObjectIndexPartitionEntry(IndexPartitionEntryStructure {
                    key_schema: key_type_reference,
                    value_schema: value_type_reference,
                })
            }

            ObjectPartitionDescriptor::SortedIndexCollection(collection_index) => {
                let key_identifier = BlueprintPayloadIdentifier::SortedIndexEntry(
                    *collection_index,
                    KeyOrValue::Key,
                );
                let value_identifier = BlueprintPayloadIdentifier::SortedIndexEntry(
                    *collection_index,
                    KeyOrValue::Value,
                );
                let key_type_reference = self
                    .system_reader
                    .get_blueprint_payload_schema_pointer(&bp_type_target, &key_identifier)
                    .expect("Could not resolve to type reference");
                let value_type_reference = self
                    .system_reader
                    .get_blueprint_payload_schema_pointer(&bp_type_target, &value_identifier)
                    .expect("Could not resolve to type reference");
                SubstateSystemStructure::ObjectSortedIndexPartitionEntry(
                    SortedIndexPartitionEntryStructure {
                        key_schema: key_type_reference,
                        value_schema: value_type_reference,
                    },
                )
            }
        }
    }
}

/// Note that the implementation below assumes that substate owned objects can not be
/// detached. If this changes, we will have to account for objects that are removed
/// from a substate.
pub struct EventSchemaMapper<'a, S: SubstateDatabase> {
    system_reader: SystemDatabaseReader<'a, S>,
    application_events: &'a Vec<(EventTypeIdentifier, Vec<u8>)>,
}

impl<'a, S: SubstateDatabase> EventSchemaMapper<'a, S> {
    pub fn new(
        substate_db: &'a S,
        tracked: &'a IndexMap<NodeId, TrackedNode>,
        application_events: &'a Vec<(EventTypeIdentifier, Vec<u8>)>,
    ) -> Self {
        Self {
            system_reader: SystemDatabaseReader::new_with_overlay(substate_db, tracked),
            application_events,
        }
    }

    pub fn run(&self) -> IndexMap<EventTypeIdentifier, EventSystemStructure> {
        let mut event_system_structures = index_map_new();
        for (event_type_identifier, _) in self.application_events {
            if event_system_structures.contains_key(event_type_identifier) {
                continue;
            }
            let blueprint_id = match &event_type_identifier.0 {
                Emitter::Function(blueprint_id) => blueprint_id.clone(),
                Emitter::Method(node_id, module_id) => {
                    if let ObjectModuleId::Main = module_id {
                        let main_type_info = self.system_reader.get_type_info(node_id).unwrap();
                        match main_type_info {
                            TypeInfoSubstate::Object(info) => info.blueprint_info.blueprint_id,
                            _ => panic!("Unexpected Type Info {:?}", main_type_info),
                        }
                    } else {
                        module_id.static_blueprint().unwrap()
                    }
                }
            };

            let blueprint_definition = self
                .system_reader
                .get_blueprint_definition(&blueprint_id)
                .unwrap();

            let type_pointer = blueprint_definition
                .interface
                .get_event_payload_def(event_type_identifier.1.as_str())
                .unwrap();

            let BlueprintPayloadDef::Static(type_identifier) = type_pointer else {
                panic!("Event identifier type pointer cannot be an instance type pointer");
            };

            let event_system_structure = EventSystemStructure {
                package_type_reference: PackageTypeReference {
                    package_address: blueprint_id.package_address,
                    schema_hash: type_identifier.0,
                    local_type_index: type_identifier.1,
                },
            };

            event_system_structures.insert(event_type_identifier.clone(), event_system_structure);
        }

        event_system_structures
    }
}
