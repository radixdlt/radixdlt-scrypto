use crate::blueprints::package::BlueprintType;
use crate::schema::*;
use crate::types::*;
use crate::*;
use radix_engine_common::crypto::Hash;
use radix_engine_interface::blueprints::resource::Vault;
use sbor::rust::fmt;
use sbor::rust::fmt::{Debug, Formatter};
use sbor::rust::prelude::*;
use sbor::LocalTypeIndex;

pub const PACKAGE_CODE_ID: u64 = 0u64;
pub const RESOURCE_CODE_ID: u64 = 1u64;
pub const IDENTITY_CODE_ID: u64 = 2u64;
pub const CONSENSUS_MANAGER_CODE_ID: u64 = 3u64;
pub const ACCOUNT_CODE_ID: u64 = 5u64;
pub const ACCESS_CONTROLLER_CODE_ID: u64 = 6u64;
pub const TRANSACTION_PROCESSOR_CODE_ID: u64 = 7u64;
pub const METADATA_CODE_ID: u64 = 10u64;
pub const ROYALTY_CODE_ID: u64 = 11u64;
pub const ACCESS_RULES_CODE_ID: u64 = 12u64;
pub const POOL_CODE_ID: u64 = 13u64;
pub const TRANSACTION_TRACKER_CODE_ID: u64 = 14u64;

pub const PACKAGE_FIELDS_PARTITION_OFFSET: PartitionOffset = PartitionOffset(0u8);
pub const PACKAGE_BLUEPRINTS_PARTITION_OFFSET: PartitionOffset = PartitionOffset(1u8);
pub const PACKAGE_BLUEPRINT_DEPENDENCIES_PARTITION_OFFSET: PartitionOffset = PartitionOffset(2u8);
pub const PACKAGE_SCHEMAS_PARTITION_OFFSET: PartitionOffset = PartitionOffset(3u8);
pub const PACKAGE_ROYALTY_PARTITION_OFFSET: PartitionOffset = PartitionOffset(4u8);
pub const PACKAGE_AUTH_TEMPLATE_PARTITION_OFFSET: PartitionOffset = PartitionOffset(5u8);

pub const PACKAGE_VM_TYPE_PARTITION_OFFSET: PartitionOffset = PartitionOffset(6u8);
pub const PACKAGE_ORIGINAL_CODE_PARTITION_OFFSET: PartitionOffset = PartitionOffset(7u8);
pub const PACKAGE_INSTRUMENTED_CODE_PARTITION_OFFSET: PartitionOffset = PartitionOffset(8u8);

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sbor)]
pub enum TypePointer {
    Package(Hash, LocalTypeIndex), // For static types
    Instance(u8),                  // For generics
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct FunctionSchema {
    pub receiver: Option<ReceiverInfo>,
    pub input: TypePointer,
    pub output: TypePointer,
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
    pub virtual_lazy_load_functions: BTreeMap<u8, PackageExport>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintInterface {
    pub blueprint_type: BlueprintType,
    pub generics: Vec<Generic>,
    pub feature_set: BTreeSet<String>,
    pub state: IndexedStateSchema,
    pub functions: BTreeMap<String, FunctionSchema>,
    pub events: BTreeMap<String, TypePointer>,
}

impl BlueprintInterface {
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

    pub fn get_field_type_pointer(&self, field_index: u8) -> Option<TypePointer> {
        let (_partition, fields) = self.state.fields.clone()?;
        let field_schema = fields.get(field_index.clone() as usize)?;
        Some(field_schema.field.clone())
    }

    pub fn get_kv_key_type_pointer(&self, collection_index: u8) -> Option<TypePointer> {
        let (_partition, schema) = self
            .state
            .collections
            .get(collection_index.clone() as usize)?;
        match schema {
            BlueprintCollectionSchema::KeyValueStore(key_value_store) => {
                Some(key_value_store.key.clone())
            }
            _ => None,
        }
    }

    pub fn get_kv_value_type_pointer(&self, collection_index: u8) -> Option<(TypePointer, bool)> {
        let (_partition, schema) = self
            .state
            .collections
            .get(collection_index.clone() as usize)?;
        match schema {
            BlueprintCollectionSchema::KeyValueStore(key_value_store) => {
                Some((key_value_store.value.clone(), key_value_store.can_own))
            }
            _ => None,
        }
    }

    pub fn get_function_input_type_pointer(&self, ident: &str) -> Option<TypePointer> {
        let schema = self.functions.get(ident)?;
        Some(schema.input.clone())
    }

    pub fn get_function_output_type_pointer(&self, ident: &str) -> Option<TypePointer> {
        let schema = self.functions.get(ident)?;
        Some(schema.output.clone())
    }

    pub fn get_event_type_pointer(&self, event_name: &str) -> Option<TypePointer> {
        self.events.get(event_name).cloned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct IndexedStateSchema {
    pub fields: Option<(PartitionOffset, Vec<FieldSchema<TypePointer>>)>,
    pub collections: Vec<(PartitionOffset, BlueprintCollectionSchema<TypePointer>)>,
    pub num_partitions: u8,
}

impl IndexedStateSchema {
    pub fn from_schema(schema_hash: Hash, schema: BlueprintStateSchemaInit) -> Self {
        let mut partition_offset = 0u8;

        let mut fields = None;
        if !schema.fields.is_empty() {
            let schema_fields = schema
                .fields
                .into_iter()
                .map(|field_schema| {
                    // FIXME: Verify that these are checked to be consistent
                    let pointer = match field_schema.field {
                        TypeRef::Static(type_index) => {
                            TypePointer::Package(schema_hash, type_index)
                        }
                        TypeRef::Generic(instance_index) => TypePointer::Instance(instance_index),
                    };
                    FieldSchema {
                        field: pointer,
                        condition: field_schema.condition,
                    }
                })
                .collect();
            fields = Some((PartitionOffset(partition_offset), schema_fields));
            partition_offset += 1;
        };

        let mut collections = Vec::new();
        for collection_schema in schema.collections {
            let schema = collection_schema.map(|type_ref| match type_ref {
                TypeRef::Static(type_index) => TypePointer::Package(schema_hash, type_index),
                TypeRef::Generic(instance_index) => TypePointer::Instance(instance_index),
            });
            collections.push((PartitionOffset(partition_offset), schema));
            partition_offset += 1;
        }

        Self {
            fields,
            collections,
            num_partitions: partition_offset,
        }
    }

    pub fn num_partitions(&self) -> u8 {
        self.num_partitions
    }

    pub fn num_fields(&self) -> usize {
        match &self.fields {
            Some((_, indices)) => indices.len(),
            _ => 0usize,
        }
    }

    pub fn field(&self, field_index: u8) -> Option<(PartitionOffset, FieldSchema<TypePointer>)> {
        match &self.fields {
            Some((offset, fields)) => {
                let field_index: usize = field_index.into();
                fields
                    .get(field_index)
                    .cloned()
                    .map(|f| (offset.clone(), f))
            }
            _ => None,
        }
    }

    pub fn key_value_store_partition(
        mut self,
        collection_index: u8,
    ) -> Option<(PartitionOffset, BlueprintKeyValueStoreSchema<TypePointer>)> {
        let index = collection_index as usize;
        if index >= self.collections.len() {
            return None;
        }

        match self.collections.swap_remove(index) {
            (offset, BlueprintCollectionSchema::KeyValueStore(schema)) => Some((offset, schema)),
            _ => None,
        }
    }

    pub fn index_partition(
        &self,
        collection_index: u8,
    ) -> Option<(PartitionOffset, &BlueprintIndexSchema)> {
        match self.collections.get(collection_index as usize) {
            Some((offset, BlueprintCollectionSchema::Index(schema))) => {
                Some((offset.clone(), schema))
            }
            _ => None,
        }
    }

    pub fn sorted_index_partition(
        &self,
        collection_index: u8,
    ) -> Option<(PartitionOffset, &BlueprintSortedIndexSchema)> {
        match self.collections.get(collection_index as usize) {
            Some((offset, BlueprintCollectionSchema::SortedIndex(schema))) => {
                Some((offset.clone(), schema))
            }
            _ => None,
        }
    }

    pub fn validate_instance_schema(&self, instance_schema: &Option<InstanceSchema>) -> bool {
        for (_, partition) in &self.collections {
            match partition {
                BlueprintCollectionSchema::KeyValueStore(kv_schema) => {
                    match &kv_schema.key {
                        TypePointer::Package(..) => {}
                        TypePointer::Instance(type_index) => {
                            if let Some(instance_schema) = instance_schema {
                                if instance_schema.type_index.len() < (*type_index as usize) {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        }
                    }

                    match &kv_schema.value {
                        TypePointer::Package(..) => {}
                        TypePointer::Instance(type_index) => {
                            if let Some(instance_schema) = instance_schema {
                                if instance_schema.type_index.len() < (*type_index as usize) {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        true
    }
}
