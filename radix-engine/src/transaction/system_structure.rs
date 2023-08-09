use crate::types::*;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use radix_engine_store_interface::interface::SubstateDatabase;
use sbor::rust::prelude::*;

use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::track::{ReadOnly, TrackedNode, TrackedSubstateValue};
use crate::transaction::{SystemPartitionDescription, SystemReader};

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub enum SubstateSystemStructure {
    SystemField(SystemFieldStructure),
    SystemType,
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
    field_kind: SystemFieldKind,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub enum SystemFieldKind {
    TypeInfo,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct KeyValueStoreEntryStructure {
    key_value_store_address: InternalAddress,
    key_schema_hash: Hash,
    key_local_type_index: LocalTypeIndex,
    value_schema_hash: Hash,
    value_local_type_index: LocalTypeIndex,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct FieldStructure {
    value_schema: ObjectSubstateTypeReference,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct KeyValuePartitionEntryStructure {
    key_schema: ObjectSubstateTypeReference,
    value_schema: ObjectSubstateTypeReference,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct IndexPartitionEntryStructure {
    key_schema: ObjectSubstateTypeReference,
    value_schema: ObjectSubstateTypeReference,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct SortedIndexPartitionEntryStructure {
    key_schema: ObjectSubstateTypeReference,
    value_schema: ObjectSubstateTypeReference,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub enum ObjectSubstateTypeReference {
    Package(PackageTypeReference),
    ObjectInstance(ObjectInstanceTypeReference),
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct PackageTypeReference {
    package_address: PackageAddress,
    schema_hash: Hash,
    local_type_index: LocalTypeIndex,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct ObjectInstanceTypeReference {
    entity_address: NodeId,
    schema_hash: Hash,
    instance_type_index: u8,
    local_type_index: LocalTypeIndex,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct KeyValueTypeReference {
    key_value_store_address: InternalAddress,
    schema_hash: Hash,
    key_local_type_index: LocalTypeIndex,
    value_local_type_index: LocalTypeIndex,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct EventSystemStructure {
    package_type_reference: PackageTypeReference,
}

#[derive(Default, Debug, Clone, ScryptoSbor)]
pub struct SystemStructure {
    pub substate_system_structures:
        IndexMap<NodeId, IndexMap<PartitionNumber, IndexMap<SubstateKey, SubstateSystemStructure>>>,
    pub event_system_structures: IndexMap<EventTypeIdentifier, EventSystemStructure>,
}

impl SystemStructure {
    pub fn resolve<S: SubstateDatabase>(
        substate_db: &S,
        updates: &IndexMap<NodeId, TrackedNode>,
        application_events: &Vec<(EventTypeIdentifier, Vec<u8>)>,
    ) -> Self {
        let substate_system_structures = SubstateSchemaMapper::new(substate_db, &updates).run();
        let event_system_structures =
            EventSchemaMapper::new(substate_db, &updates, application_events).run();

        SystemStructure {
            substate_system_structures,
            event_system_structures,
        }
    }
}

/// Note that the implementation below assumes that substate owned objects can not be
/// detached. If this changes, we will have to account for objects that are removed
/// from a substate.
pub struct SubstateSchemaMapper<'a, S: SubstateDatabase> {
    system_reader: SystemReader<'a, S>,
    tracked: &'a IndexMap<NodeId, TrackedNode>,
}

impl<'a, S: SubstateDatabase> SubstateSchemaMapper<'a, S> {
    pub fn new(substate_db: &'a S, tracked: &'a IndexMap<NodeId, TrackedNode>) -> Self {
        Self {
            system_reader: SystemReader::new_with_overlay(substate_db, tracked),
            tracked,
        }
    }

    pub fn run(
        &self,
    ) -> IndexMap<NodeId, IndexMap<PartitionNumber, IndexMap<SubstateKey, SubstateSystemStructure>>>
    {
        let mut substate_structures = index_map_new();
        for (node_id, tracked_node) in self.tracked {
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

                    let partition_description =
                        self.system_reader.partition_description(partition_num);

                    let system_substate_structure = self.resolve_substate_structure(
                        node_id,
                        partition_description,
                        &tracked_substate.substate_key,
                    );

                    substate_structures
                        .entry(node_id.clone())
                        .or_insert(index_map_new())
                        .entry(partition_num.clone())
                        .or_insert(index_map_new())
                        .insert(
                            tracked_substate.substate_key.clone(),
                            system_substate_structure,
                        );
                }
            }
        }

        substate_structures
    }

    pub fn resolve_substate_structure(
        &self,
        node_id: &NodeId,
        partition_description: SystemPartitionDescription,
        key: &SubstateKey,
    ) -> SubstateSystemStructure {
        match partition_description {
            SystemPartitionDescription::TypeInfo => {
                SubstateSystemStructure::SystemField(SystemFieldStructure {
                    field_kind: SystemFieldKind::TypeInfo,
                })
            }
            SystemPartitionDescription::System(_partition_num) => {
                SubstateSystemStructure::SystemType
            }
            SystemPartitionDescription::Module(module_id, partition_offset) => {
                let (blueprint_id, type_instances) = if let ObjectModuleId::Main = module_id {
                    let main_type_info =
                        self.system_reader
                            .get_type_info(node_id)
                            .unwrap_or_else(|| {
                                panic!("Could not read type info substate for node {node_id:?}")
                            });
                    match main_type_info {
                        TypeInfoSubstate::Object(info) => (
                            info.blueprint_info.blueprint_id,
                            info.blueprint_info.type_substitutions,
                        ),
                        TypeInfoSubstate::KeyValueStore(info) => {
                            return SubstateSystemStructure::KeyValueStoreEntry(
                                KeyValueStoreEntryStructure {
                                    key_value_store_address: (*node_id).try_into().unwrap(),
                                    key_schema_hash: info.schema.key_type_substitution.0,
                                    key_local_type_index: info.schema.key_type_substitution.1,
                                    value_schema_hash: info.schema.value_type_substitution.0,
                                    value_local_type_index: info.schema.value_type_substitution.1,
                                },
                            )
                        }
                        TypeInfoSubstate::GlobalAddressPhantom(_)
                        | TypeInfoSubstate::GlobalAddressReservation(_) => {
                            panic!("Unexpected Type Info {:?}", main_type_info)
                        }
                    }
                } else {
                    (module_id.static_blueprint().unwrap(), vec![])
                };

                let blueprint_definition = self
                    .system_reader
                    .get_blueprint_definition(&blueprint_id)
                    .unwrap();
                let resolver = ObjectSubstateTypeReferenceResolver::new(
                    &node_id,
                    &blueprint_id,
                    &type_instances,
                );
                self.resolve_object_substate_structure(
                    &resolver,
                    &blueprint_definition.interface.state,
                    partition_offset,
                    key,
                )
            }
        }
    }

    pub fn resolve_object_substate_structure(
        &self,
        resolver: &ObjectSubstateTypeReferenceResolver,
        state_schema: &IndexedStateSchema,
        partition_offset: PartitionOffset,
        key: &SubstateKey,
    ) -> SubstateSystemStructure {
        if partition_offset.0 >= state_schema.num_partitions {
            panic!("Partition offset larger than partition count");
        }

        if let Some((PartitionDescription::Logical(offset), fields)) = &state_schema.fields {
            if offset.eq(&partition_offset) {
                if let SubstateKey::Field(field_index) = key {
                    let field = fields
                        .get(*field_index as usize)
                        .expect("Field index was not valid");
                    return SubstateSystemStructure::ObjectField(FieldStructure {
                        value_schema: resolver.resolve(field.field),
                    });
                } else {
                    panic!("Expected a field substate key");
                }
            }
        }

        for (partition_description, collection_schema) in &state_schema.collections {
            match partition_description {
                PartitionDescription::Logical(offset) => {
                    if offset.eq(&partition_offset) {
                        match collection_schema {
                            BlueprintCollectionSchema::KeyValueStore(kv_schema) => {
                                return SubstateSystemStructure::ObjectKeyValuePartitionEntry(
                                    KeyValuePartitionEntryStructure {
                                        key_schema: resolver.resolve(kv_schema.key),
                                        value_schema: resolver.resolve(kv_schema.value),
                                    },
                                )
                            }
                            BlueprintCollectionSchema::Index(kv_schema) => {
                                return SubstateSystemStructure::ObjectIndexPartitionEntry(
                                    IndexPartitionEntryStructure {
                                        key_schema: resolver.resolve(kv_schema.key),
                                        value_schema: resolver.resolve(kv_schema.value),
                                    },
                                )
                            }
                            BlueprintCollectionSchema::SortedIndex(kv_schema) => {
                                return SubstateSystemStructure::ObjectSortedIndexPartitionEntry(
                                    SortedIndexPartitionEntryStructure {
                                        key_schema: resolver.resolve(kv_schema.key),
                                        value_schema: resolver.resolve(kv_schema.value),
                                    },
                                )
                            }
                        }
                    }
                }
                PartitionDescription::Physical(..) => {}
            }
        }

        panic!("Partition offset did not match any partitions on the blueprint definition")
    }
}

pub struct ObjectSubstateTypeReferenceResolver<'a> {
    node_id: &'a NodeId,
    blueprint_id: &'a BlueprintId,
    type_instances: &'a Vec<TypeIdentifier>,
}

impl<'a> ObjectSubstateTypeReferenceResolver<'a> {
    pub fn new(
        node_id: &'a NodeId,
        blueprint_id: &'a BlueprintId,
        type_instances: &'a Vec<TypeIdentifier>,
    ) -> Self {
        Self {
            node_id,
            blueprint_id,
            type_instances,
        }
    }

    pub fn resolve(&self, type_pointer: TypePointer) -> ObjectSubstateTypeReference {
        match type_pointer {
            TypePointer::Package(type_identifier) => {
                ObjectSubstateTypeReference::Package(PackageTypeReference {
                    package_address: self.blueprint_id.package_address,
                    schema_hash: type_identifier.0,
                    local_type_index: type_identifier.1,
                })
            }
            TypePointer::Instance(instance_type_index) => {
                let type_identifier = *self
                    .type_instances
                    .get(instance_type_index as usize)
                    .expect("Instance type index not valid");
                ObjectSubstateTypeReference::ObjectInstance(ObjectInstanceTypeReference {
                    entity_address: (*self.node_id).try_into().unwrap(),
                    instance_type_index,
                    schema_hash: type_identifier.0,
                    local_type_index: type_identifier.1,
                })
            }
        }
    }
}

/// Note that the implementation below assumes that substate owned objects can not be
/// detached. If this changes, we will have to account for objects that are removed
/// from a substate.
pub struct EventSchemaMapper<'a, S: SubstateDatabase> {
    system_reader: SystemReader<'a, S>,
    application_events: &'a Vec<(EventTypeIdentifier, Vec<u8>)>,
}

impl<'a, S: SubstateDatabase> EventSchemaMapper<'a, S> {
    pub fn new(
        substate_db: &'a S,
        tracked: &'a IndexMap<NodeId, TrackedNode>,
        application_events: &'a Vec<(EventTypeIdentifier, Vec<u8>)>,
    ) -> Self {
        Self {
            system_reader: SystemReader::new_with_overlay(substate_db, tracked),
            application_events,
        }
    }

    pub fn run(&self) -> IndexMap<EventTypeIdentifier, EventSystemStructure> {
        let mut event_system_structures = index_map_new();
        for (event_type_identifier, _) in self.application_events {
            if !event_system_structures.contains_key(event_type_identifier) {
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

            let type_pointer = blueprint_definition.interface.get_event_type_pointer(event_type_identifier.1.as_str()).unwrap();

            let TypePointer::Package(type_identifier) = type_pointer else {
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
