use crate::api::CollectionIndex;
use crate::blueprints::package::BlueprintType;
use crate::schema::*;
use crate::types::*;
use crate::*;
use radix_engine_common::crypto::Hash;
use radix_engine_interface::blueprints::resource::Vault;
use sbor::rust::fmt;
use sbor::rust::fmt::{Debug, Formatter};
use sbor::rust::prelude::*;

pub const PACKAGE_CODE_ID: u64 = 0u64;
pub const RESOURCE_CODE_ID: u64 = 1u64;
pub const IDENTITY_CODE_ID: u64 = 2u64;
pub const CONSENSUS_MANAGER_CODE_ID: u64 = 3u64;
pub const ACCOUNT_CODE_ID: u64 = 5u64;
pub const ACCESS_CONTROLLER_CODE_ID: u64 = 6u64;
pub const TRANSACTION_PROCESSOR_CODE_ID: u64 = 7u64;
pub const METADATA_CODE_ID: u64 = 10u64;
pub const ROYALTY_CODE_ID: u64 = 11u64;
pub const ROLE_ASSIGNMENT_CODE_ID: u64 = 12u64;
pub const POOL_CODE_ID: u64 = 13u64;
pub const TRANSACTION_TRACKER_CODE_ID: u64 = 14u64;

pub const PACKAGE_FIELDS_PARTITION_OFFSET: PartitionOffset = PartitionOffset(0u8);

pub const PACKAGE_BLUEPRINTS_PARTITION_OFFSET: PartitionOffset = PartitionOffset(1u8);
pub const PACKAGE_BLUEPRINTS_COLLECTION_INDEX: CollectionIndex = 0u8;

pub const PACKAGE_BLUEPRINT_DEPENDENCIES_PARTITION_OFFSET: PartitionOffset = PartitionOffset(2u8);
pub const PACKAGE_BLUEPRINT_DEPENDENCIES_COLLECTION_INDEX: CollectionIndex = 1u8;

// There is no offset for the package schema collection as it is directly mapped to SCHEMAS_PARTITION
pub const PACKAGE_SCHEMAS_COLLECTION_INDEX: CollectionIndex = 2u8;

pub const PACKAGE_ROYALTY_PARTITION_OFFSET: PartitionOffset = PartitionOffset(3u8);
pub const PACKAGE_ROYALTY_COLLECTION_INDEX: CollectionIndex = 3u8;

pub const PACKAGE_AUTH_TEMPLATE_PARTITION_OFFSET: PartitionOffset = PartitionOffset(4u8);
pub const PACKAGE_AUTH_TEMPLATE_COLLECTION_INDEX: CollectionIndex = 4u8;

pub const PACKAGE_VM_TYPE_PARTITION_OFFSET: PartitionOffset = PartitionOffset(5u8);
pub const PACKAGE_VM_TYPE_COLLECTION_INDEX: CollectionIndex = 5u8;

pub const PACKAGE_ORIGINAL_CODE_PARTITION_OFFSET: PartitionOffset = PartitionOffset(6u8);
pub const PACKAGE_ORIGINAL_CODE_COLLECTION_INDEX: CollectionIndex = 6u8;

pub const PACKAGE_INSTRUMENTED_CODE_PARTITION_OFFSET: PartitionOffset = PartitionOffset(7u8);
pub const PACKAGE_INSTRUMENTED_CODE_COLLECTION_INDEX: CollectionIndex = 7u8;

#[derive(Copy, Debug, Clone, PartialEq, Eq, Sbor)]
pub enum VmType {
    Native,
    ScryptoV1,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq)]
pub struct PackageVmTypeSubstate {
    pub vm_type: VmType,
}

#[derive(Clone, Sbor, PartialEq, Eq)]
pub struct PackageOriginalCodeSubstate {
    pub code: Vec<u8>,
}

#[derive(Clone, Sbor, PartialEq, Eq)]
pub struct PackageInstrumentedCodeSubstate {
    pub code: Vec<u8>,
}

impl Debug for PackageOriginalCodeSubstate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PackageOriginalCodeSubstate")
            .field("len", &self.code.len())
            .finish()
    }
}

impl Debug for PackageInstrumentedCodeSubstate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("PackageInstrumentedCodeSubstate")
            .field("len", &self.code.len())
            .finish()
    }
}

#[derive(Debug, PartialEq, Eq, ScryptoSbor)]
pub struct PackageRoyaltyAccumulatorSubstate {
    /// The vault for collecting package royalties.
    pub royalty_vault: Vault,
}

impl Clone for PackageRoyaltyAccumulatorSubstate {
    fn clone(&self) -> Self {
        Self {
            royalty_vault: Vault(self.royalty_vault.0.clone()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub enum BlueprintPayloadDef {
    Static(TypeIdentifier), // Fully Resolved type is defined in package
    Generic(u8), // Fully Resolved type is mapped directly to a generic defined by instance
                 // TODO: How to represent a structure containing a generic?
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct FunctionSchema {
    pub receiver: Option<ReceiverInfo>,
    pub input: BlueprintPayloadDef,
    pub output: BlueprintPayloadDef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ScryptoSbor, Ord, PartialOrd, Hash)]
pub struct BlueprintVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Default for BlueprintVersion {
    fn default() -> Self {
        Self {
            major: 1,
            minor: 0,
            patch: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, Ord, PartialOrd, Hash)]
pub struct CanonicalBlueprintId {
    pub address: PackageAddress,
    pub blueprint: String,
    pub version: BlueprintVersion,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, Ord, PartialOrd, Hash)]
pub struct BlueprintVersionKey {
    pub blueprint: String,
    pub version: BlueprintVersion,
}

impl BlueprintVersionKey {
    pub fn new_default<S: ToString>(blueprint: S) -> Self {
        Self {
            blueprint: blueprint.to_string(),
            version: BlueprintVersion::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
#[sbor(transparent)]
pub struct BlueprintDependencies {
    pub dependencies: BTreeSet<GlobalAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct PackageExport {
    pub code_hash: Hash,
    pub export_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct BlueprintDefinition {
    // Frontend interface, this must be backward compatible with minor version updates
    pub interface: BlueprintInterface,

    // Backend implementation pointers

    // There is an implicit variant that must be maintained in that the key set in `function_exports`
    // matches that of the `functions` under `interface`. This is currently maintained since the
    // `publish` interface uses `BlueprintDefinitionInit` rather than `BlueprintDefinition`.
    pub function_exports: BTreeMap<String, PackageExport>,
    pub hook_exports: BTreeMap<BlueprintHook, PackageExport>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum KeyOrValue {
    Key,
    Value,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum InputOrOutput {
    Input,
    Output,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum BlueprintPayloadIdentifier {
    Function(String, InputOrOutput),
    Event(String),
    Field(u8),
    KeyValueEntry(u8, KeyOrValue),
    IndexEntry(u8, KeyOrValue),
    SortedIndexEntry(u8, KeyOrValue),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum BlueprintPartitionType {
    KeyValueCollection,
    IndexCollection,
    SortedIndexCollection,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintInterface {
    pub blueprint_type: BlueprintType,
    pub is_transient: bool,
    pub generics: Vec<GenericBound>,
    pub feature_set: BTreeSet<String>,
    pub state: IndexedStateSchema,
    pub functions: BTreeMap<String, FunctionSchema>,
    pub events: BTreeMap<String, BlueprintPayloadDef>,
}

impl BlueprintInterface {
    pub fn get_field_payload_def(&self, field_index: u8) -> Option<BlueprintPayloadDef> {
        self.state.get_field_payload_def(field_index)
    }

    pub fn get_kv_key_payload_def(&self, collection_index: u8) -> Option<BlueprintPayloadDef> {
        self.state.get_kv_key_payload_def(collection_index)
    }

    pub fn find_function(&self, ident: &str) -> Option<FunctionSchema> {
        if let Some(x) = self.functions.get(ident) {
            if x.receiver.is_none() {
                return Some(x.clone());
            }
        }
        None
    }

    pub fn find_method(&self, ident: &str) -> Option<FunctionSchema> {
        if let Some(x) = self.functions.get(ident) {
            if x.receiver.is_some() {
                return Some(x.clone());
            }
        }
        None
    }

    pub fn get_function_input_payload_def(&self, ident: &str) -> Option<BlueprintPayloadDef> {
        let schema = self.functions.get(ident)?;
        Some(schema.input.clone())
    }

    pub fn get_function_output_payload_def(&self, ident: &str) -> Option<BlueprintPayloadDef> {
        let schema = self.functions.get(ident)?;
        Some(schema.output.clone())
    }

    pub fn get_event_payload_def(&self, event_name: &str) -> Option<BlueprintPayloadDef> {
        self.events.get(event_name).cloned()
    }

    pub fn get_payload_def(
        &self,
        payload_identifier: &BlueprintPayloadIdentifier,
    ) -> Option<(BlueprintPayloadDef, bool, bool)> {
        match payload_identifier {
            BlueprintPayloadIdentifier::Function(function_ident, InputOrOutput::Input) => {
                let payload_def = self.get_function_input_payload_def(function_ident.as_str())?;
                Some((payload_def, true, true))
            }
            BlueprintPayloadIdentifier::Function(function_ident, InputOrOutput::Output) => {
                let payload_def = self.get_function_output_payload_def(function_ident.as_str())?;
                Some((payload_def, true, true))
            }
            BlueprintPayloadIdentifier::Field(field_index) => {
                let payload_def = self.get_field_payload_def(*field_index)?;
                Some((payload_def, true, self.is_transient))
            }
            BlueprintPayloadIdentifier::KeyValueEntry(collection_index, KeyOrValue::Key) => {
                let payload_def = self.get_kv_key_payload_def(*collection_index)?;
                Some((payload_def, false, self.is_transient))
            }
            BlueprintPayloadIdentifier::KeyValueEntry(collection_index, KeyOrValue::Value) => {
                let (payload_def, allow_ownership) =
                    self.state.get_kv_value_payload_def(*collection_index)?;
                Some((payload_def, allow_ownership, self.is_transient))
            }
            BlueprintPayloadIdentifier::IndexEntry(collection_index, KeyOrValue::Key) => {
                let type_pointer = self.state.get_index_payload_def_key(*collection_index)?;
                Some((type_pointer, false, self.is_transient))
            }
            BlueprintPayloadIdentifier::IndexEntry(collection_index, KeyOrValue::Value) => {
                let type_pointer = self.state.get_index_payload_def_value(*collection_index)?;
                Some((type_pointer, false, self.is_transient))
            }
            BlueprintPayloadIdentifier::SortedIndexEntry(collection_index, KeyOrValue::Key) => {
                let type_pointer = self
                    .state
                    .get_sorted_index_payload_def_key(*collection_index)?;
                Some((type_pointer, false, self.is_transient))
            }
            BlueprintPayloadIdentifier::SortedIndexEntry(collection_index, KeyOrValue::Value) => {
                let type_pointer = self
                    .state
                    .get_sorted_index_payload_def_value(*collection_index)?;
                Some((type_pointer, false, self.is_transient))
            }
            BlueprintPayloadIdentifier::Event(event_name) => {
                let type_pointer = self.get_event_payload_def(event_name.as_str())?;
                Some((type_pointer, false, false))
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SystemInstruction {
    MapCollectionToPhysicalPartition {
        collection_index: u8,
        partition_num: PartitionNumber,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor, PartialOrd, Ord, Hash)]
pub enum PartitionDescription {
    Logical(PartitionOffset),
    Physical(PartitionNumber),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct IndexedStateSchema {
    pub fields: Option<(PartitionDescription, Vec<FieldSchema<BlueprintPayloadDef>>)>,
    pub collections: Vec<(
        PartitionDescription,
        BlueprintCollectionSchema<BlueprintPayloadDef>,
    )>,
    pub num_logical_partitions: u8,
}

impl IndexedStateSchema {
    pub fn from_schema(
        schema_hash: Hash,
        schema: BlueprintStateSchemaInit,
        system_mappings: BTreeMap<usize, PartitionNumber>,
    ) -> Self {
        let mut partition_offset = 0u8;

        let mut fields = None;
        if !schema.fields.is_empty() {
            let schema_fields = schema
                .fields
                .into_iter()
                .map(|field_schema| {
                    let pointer = match field_schema.field {
                        TypeRef::Static(type_index) => {
                            BlueprintPayloadDef::Static(TypeIdentifier(schema_hash, type_index))
                        }
                        TypeRef::Generic(instance_index) => {
                            BlueprintPayloadDef::Generic(instance_index)
                        }
                    };
                    FieldSchema {
                        field: pointer,
                        condition: field_schema.condition,
                        transience: field_schema.transience,
                    }
                })
                .collect();
            fields = Some((
                PartitionDescription::Logical(PartitionOffset(partition_offset)),
                schema_fields,
            ));
            partition_offset += 1;
        };

        let mut collections = Vec::new();
        for (collection_index, collection_schema) in schema.collections.into_iter().enumerate() {
            let schema = collection_schema.map(|type_ref| match type_ref {
                TypeRef::Static(type_index) => {
                    BlueprintPayloadDef::Static(TypeIdentifier(schema_hash, type_index))
                }
                TypeRef::Generic(generic_index) => BlueprintPayloadDef::Generic(generic_index),
            });

            if let Some(partition_num) = system_mappings.get(&collection_index) {
                collections.push((PartitionDescription::Physical(*partition_num), schema));
            } else {
                collections.push((
                    PartitionDescription::Logical(PartitionOffset(partition_offset)),
                    schema,
                ));
                partition_offset += 1;
            }
        }

        Self {
            fields,
            collections,
            num_logical_partitions: partition_offset,
        }
    }

    pub fn num_logical_partitions(&self) -> u8 {
        self.num_logical_partitions
    }

    pub fn num_fields(&self) -> usize {
        match &self.fields {
            Some((_, indices)) => indices.len(),
            _ => 0usize,
        }
    }

    pub fn get_partition(
        &self,
        collection_index: u8,
    ) -> Option<(PartitionDescription, BlueprintPartitionType)> {
        self.collections
            .get(collection_index as usize)
            .map(|(partition, schema)| {
                let partition_type = match schema {
                    BlueprintCollectionSchema::KeyValueStore(..) => {
                        BlueprintPartitionType::KeyValueCollection
                    }
                    BlueprintCollectionSchema::Index(..) => BlueprintPartitionType::IndexCollection,
                    BlueprintCollectionSchema::SortedIndex(..) => {
                        BlueprintPartitionType::SortedIndexCollection
                    }
                };
                (*partition, partition_type)
            })
    }

    pub fn get_field_payload_def(&self, field_index: u8) -> Option<BlueprintPayloadDef> {
        let (_partition, fields) = self.fields.clone()?;
        let field_schema = fields.get(field_index.clone() as usize)?;
        Some(field_schema.field.clone())
    }

    pub fn get_kv_key_payload_def(&self, collection_index: u8) -> Option<BlueprintPayloadDef> {
        let (_partition, schema) = self.collections.get(collection_index.clone() as usize)?;
        match schema {
            BlueprintCollectionSchema::KeyValueStore(key_value_store) => {
                Some(key_value_store.key.clone())
            }
            _ => None,
        }
    }

    pub fn get_kv_value_payload_def(
        &self,
        collection_index: u8,
    ) -> Option<(BlueprintPayloadDef, bool)> {
        let (_partition, schema) = self.collections.get(collection_index.clone() as usize)?;
        match schema {
            BlueprintCollectionSchema::KeyValueStore(key_value_store) => Some((
                key_value_store.value.clone(),
                key_value_store.allow_ownership,
            )),
            _ => None,
        }
    }

    pub fn get_index_payload_def_key(&self, collection_index: u8) -> Option<BlueprintPayloadDef> {
        let (_partition, schema) = self.collections.get(collection_index.clone() as usize)?;
        match schema {
            BlueprintCollectionSchema::Index(index) => Some(index.key.clone()),
            _ => None,
        }
    }

    pub fn get_index_payload_def_value(&self, collection_index: u8) -> Option<BlueprintPayloadDef> {
        let (_partition, schema) = self.collections.get(collection_index.clone() as usize)?;
        match schema {
            BlueprintCollectionSchema::Index(index) => Some(index.value.clone()),
            _ => None,
        }
    }

    pub fn get_sorted_index_payload_def_key(
        &self,
        collection_index: u8,
    ) -> Option<BlueprintPayloadDef> {
        let (_partition, schema) = self.collections.get(collection_index.clone() as usize)?;
        match schema {
            BlueprintCollectionSchema::SortedIndex(index) => Some(index.key.clone()),
            _ => None,
        }
    }

    pub fn get_sorted_index_payload_def_value(
        &self,
        collection_index: u8,
    ) -> Option<BlueprintPayloadDef> {
        let (_partition, schema) = self.collections.get(collection_index.clone() as usize)?;
        match schema {
            BlueprintCollectionSchema::SortedIndex(index) => Some(index.value.clone()),
            _ => None,
        }
    }

    pub fn fields_partition(&self) -> Option<PartitionDescription> {
        match &self.fields {
            Some((partition, ..)) => Some(partition.clone()),
            _ => None,
        }
    }

    pub fn field(
        &self,
        field_index: u8,
    ) -> Option<(PartitionDescription, FieldSchema<BlueprintPayloadDef>)> {
        match &self.fields {
            Some((partition, fields)) => {
                let field_index: usize = field_index.into();
                fields
                    .get(field_index)
                    .cloned()
                    .map(|f| (partition.clone(), f))
            }
            _ => None,
        }
    }
}
