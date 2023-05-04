#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

use radix_engine_common::data::scrypto::{ScryptoCustomTypeKind, ScryptoDescribe, ScryptoSchema};
use radix_engine_common::types::PartitionOffset;
use radix_engine_common::{ManifestSbor, ScryptoSbor};
use sbor::rust::prelude::*;
use sbor::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct KeyValueStoreSchema {
    pub key: LocalTypeIndex,
    pub value: LocalTypeIndex,
    pub can_own: bool, // TODO: Can this be integrated with ScryptoSchema?
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct KeyValueStoreInfo {
    pub schema: ScryptoSchema,
    pub kv_store_schema: KeyValueStoreSchema,
}

impl KeyValueStoreInfo {
    pub fn new<K: ScryptoDescribe, V: ScryptoDescribe>(can_own: bool) -> Self {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let key_type_index = aggregator.add_child_type_and_descendents::<K>();
        let value_type_index = aggregator.add_child_type_and_descendents::<V>();
        let schema = generate_full_schema(aggregator);
        Self {
            schema,
            kv_store_schema: KeyValueStoreSchema {
                key: key_type_index,
                value: value_type_index,
                can_own,
            },
        }
    }
}

// We keep one self-contained schema per blueprint:
// - Easier macro to export schema, as they work at blueprint level
// - Can always combine multiple schemas into one for storage benefits

#[derive(Default, Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct PackageSchema {
    pub blueprints: BTreeMap<String, BlueprintSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintSchema {
    pub outer_blueprint: Option<String>,
    pub schema: ScryptoSchema,

    /// State Schema
    pub fields: Vec<LocalTypeIndex>,
    pub kv_stores: Vec<BlueprintKeyValueStoreSchema>,
    pub indices: Vec<BlueprintIndexSchema>,
    pub sorted_indices: Vec<BlueprintSortedIndexSchema>,

    /// For each function, there is a [`FunctionSchema`]
    pub functions: BTreeMap<String, FunctionSchema>,
    /// For each virtual lazy load function, there is a [`VirtualLazyLoadSchema`]
    pub virtual_lazy_load_functions: BTreeMap<u8, VirtualLazyLoadSchema>,
    /// For each event, there is a name [`String`] that maps to a [`LocalTypeIndex`]
    pub event_schema: BTreeMap<String, LocalTypeIndex>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub enum TypeSchema {
    Blueprint(LocalTypeIndex),
    Instance(u8),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintKeyValueStoreSchema {
    pub key: TypeSchema,
    pub value: TypeSchema,
    pub can_own: bool, // TODO: Can this be integrated with ScryptoSchema?
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintIndexSchema {}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintSortedIndexSchema {}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub enum BlueprintPartitionSchema {
    Fields(Vec<LocalTypeIndex>),
    KeyValueStore(BlueprintKeyValueStoreSchema),
    Index(BlueprintIndexSchema),
    SortedIndex(BlueprintSortedIndexSchema),
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct FunctionSchema {
    pub receiver: Option<Receiver>,
    pub input: LocalTypeIndex,
    pub output: LocalTypeIndex,
    pub export_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct VirtualLazyLoadSchema {
    pub export_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum Receiver {
    SelfRef,
    SelfRefMut,
}

impl Default for BlueprintSchema {
    fn default() -> Self {
        Self {
            outer_blueprint: None,
            schema: ScryptoSchema {
                type_kinds: vec![],
                type_metadata: vec![],
                type_validations: vec![],
            },
            fields: Vec::default(),
            kv_stores: Vec::default(),
            indices: Vec::default(),
            sorted_indices: Vec::default(),
            functions: BTreeMap::default(),
            virtual_lazy_load_functions: BTreeMap::default(),
            event_schema: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct IndexedBlueprintSchema {
    pub outer_blueprint: Option<String>,

    pub schema: ScryptoSchema,

    pub partitions: Vec<BlueprintPartitionSchema>,
    pub fields_partition_index: Option<PartitionOffset>,

    /// For each function, there is a [`FunctionSchema`]
    pub functions: BTreeMap<String, FunctionSchema>,
    /// For each virtual lazy load function, there is a [`VirtualLazyLoadSchema`]
    pub virtual_lazy_load_functions: BTreeMap<u8, VirtualLazyLoadSchema>,
    /// For each event, there is a name [`String`] that maps to a [`LocalTypeIndex`]
    pub event_schema: BTreeMap<String, LocalTypeIndex>,
}

impl From<BlueprintSchema> for IndexedBlueprintSchema {
    fn from(schema: BlueprintSchema) -> Self {
        let mut paritions = Vec::new();
        let mut fields_partition_index = None;
        if !schema.fields.is_empty() {
            fields_partition_index = Some(PartitionOffset(0u8));
            paritions.push(BlueprintPartitionSchema::Fields(schema.fields));
        };
        for kv_schema in schema.kv_stores {
            paritions.push(BlueprintPartitionSchema::KeyValueStore(kv_schema));
        }

        for index_schema in schema.indices {
            paritions.push(BlueprintPartitionSchema::Index(index_schema));
        }

        for sorted_index_schema in schema.sorted_indices {
            paritions.push(BlueprintPartitionSchema::SortedIndex(sorted_index_schema));
        }

        Self {
            outer_blueprint: schema.outer_blueprint,
            schema: schema.schema,
            fields_partition_index,
            partitions: paritions,
            functions: schema.functions,
            virtual_lazy_load_functions: schema.virtual_lazy_load_functions,
            event_schema: schema.event_schema,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct IndexedPackageSchema {
    pub blueprints: BTreeMap<String, IndexedBlueprintSchema>,
}

impl From<PackageSchema> for IndexedPackageSchema {
    fn from(value: PackageSchema) -> Self {
        IndexedPackageSchema {
            blueprints: value
                .blueprints
                .into_iter()
                .map(|(name, b)| (name, b.into()))
                .collect(),
        }
    }
}

impl IndexedBlueprintSchema {
    pub fn fields(&self) -> Option<&Vec<LocalTypeIndex>> {
        match self.fields_partition_index {
            Some(partition) => match self.partitions.get(partition.0 as usize).unwrap() {
                BlueprintPartitionSchema::Fields(indices) => Some(indices),
                _ => panic!("Index broken!"),
            },
            _ => None,
        }
    }

    pub fn num_fields(&self) -> usize {
        self.fields().map(|l| l.len()).unwrap_or(0usize)
    }

    pub fn field(&self, field_index: u8) -> Option<(PartitionOffset, LocalTypeIndex)> {
        match self.fields_partition_index {
            Some(offset) => match self.partitions.get(offset.0 as usize).unwrap() {
                BlueprintPartitionSchema::Fields(fields) => {
                    let field_index: usize = field_index.into();
                    fields.get(field_index).cloned().map(|f| (offset, f))
                }
                _ => panic!("Index broken!"),
            },
            _ => None,
        }
    }

    pub fn key_value_store_partition(
        mut self,
        handle: u8,
    ) -> Option<(PartitionOffset, ScryptoSchema, BlueprintKeyValueStoreSchema)> {
        let index = handle as usize;
        if index >= self.partitions.len() {
            return None;
        }

        match self.partitions.swap_remove(index) {
            BlueprintPartitionSchema::KeyValueStore(schema) => {
                Some((PartitionOffset(handle), self.schema, schema))
            }
            _ => None,
        }
    }

    pub fn index_partition_offset(
        &self,
        handle: u8,
    ) -> Option<(PartitionOffset, &BlueprintIndexSchema)> {
        match self.partitions.get(handle as usize) {
            Some(BlueprintPartitionSchema::Index(schema)) => {
                Some((PartitionOffset(handle), schema))
            }
            _ => None,
        }
    }

    pub fn sorted_index_partition_offset(
        &self,
        handle: u8,
    ) -> Option<(PartitionOffset, &BlueprintSortedIndexSchema)> {
        match self.partitions.get(handle as usize) {
            Some(BlueprintPartitionSchema::SortedIndex(schema)) => {
                Some((PartitionOffset(handle), schema))
            }
            _ => None,
        }
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

    pub fn validate_instance_schema(&self, instance_schema: &Option<InstanceSchema>) -> bool {
        for partition in &self.partitions {
            match partition {
                BlueprintPartitionSchema::KeyValueStore(kv_schema) => {
                    match &kv_schema.key {
                        TypeSchema::Blueprint(..) => {}
                        TypeSchema::Instance(type_index) => {
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
                        TypeSchema::Blueprint(..) => {}
                        TypeSchema::Instance(type_index) => {
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

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct InstanceSchema {
    pub schema: ScryptoSchema,
    pub type_index: Vec<LocalTypeIndex>,
}
