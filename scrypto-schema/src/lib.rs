#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

use radix_engine_common::data::scrypto::{ScryptoCustomTypeKind, ScryptoDescribe, ScryptoSchema};
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
pub struct BlueprintIndexSchema {
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintSortedIndexSchema {
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

    pub tuple_module: Option<(u8, Vec<LocalTypeIndex>)>,
    pub kv_store_modules: Vec<(u8, BlueprintKeyValueStoreSchema)>,
    pub index_modules: Vec<(u8, BlueprintIndexSchema)>,
    pub sorted_index_modules: Vec<(u8, BlueprintSortedIndexSchema)>,

    /// For each function, there is a [`FunctionSchema`]
    pub functions: BTreeMap<String, FunctionSchema>,
    /// For each virtual lazy load function, there is a [`VirtualLazyLoadSchema`]
    pub virtual_lazy_load_functions: BTreeMap<u8, VirtualLazyLoadSchema>,
    /// For each event, there is a name [`String`] that maps to a [`LocalTypeIndex`]
    pub event_schema: BTreeMap<String, LocalTypeIndex>,
}

impl From<BlueprintSchema> for IndexedBlueprintSchema {
    fn from(schema: BlueprintSchema) -> Self {
        let mut module_offset = 0u8;

        let tuple_module = if schema.fields.is_empty() {
            None
        } else {
            let tuple_module = Some((module_offset, schema.fields));
            module_offset += 1;
            tuple_module
        };

        let mut kv_store_modules = Vec::new();
        for kv_schema in schema.kv_stores {
            kv_store_modules.push((module_offset, kv_schema));
            module_offset += 1;
        }

        let mut index_modules = Vec::new();
        for index_schema in schema.indices {
            index_modules.push((module_offset, index_schema));
            module_offset += 1;
        }

        let mut sorted_index_modules = Vec::new();
        for sorted_index_schema in schema.sorted_indices {
            sorted_index_modules.push((module_offset, sorted_index_schema));
            module_offset += 1;
        }

        Self {
            outer_blueprint: schema.outer_blueprint,
            schema: schema.schema,
            tuple_module,
            kv_store_modules,
            index_modules,
            sorted_index_modules,
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
    pub fn num_fields(&self) -> usize {
        self.tuple_module
            .as_ref()
            .map(|(_, fields)| fields.len())
            .unwrap_or(0)
    }

    pub fn field(&self, field_index: u8) -> Option<(u8, LocalTypeIndex)> {
        self.tuple_module.as_ref().and_then(|(offset, fields)| {
            let field_index: usize = field_index.into();
            fields.get(field_index).cloned().map(|f| (*offset, f))
        })
    }

    pub fn key_value_store_module_offset(
        &self,
        kv_handle: u8,
    ) -> Option<&(u8, BlueprintKeyValueStoreSchema)> {
        self.kv_store_modules.get(kv_handle as usize)
    }

    pub fn index_module_offset(
        &self,
        index_handle: u8,
    ) -> Option<&(u8, BlueprintIndexSchema)> {
        self.index_modules.get(index_handle as usize)
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
        for (_offset, kv_schema) in &self.kv_store_modules {
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

        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct InstanceSchema {
    pub schema: ScryptoSchema,
    pub type_index: Vec<LocalTypeIndex>,
}
