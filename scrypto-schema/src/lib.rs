#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

use bitflags::bitflags;
use radix_engine_common::data::scrypto::{ScryptoCustomTypeKind, ScryptoDescribe, ScryptoSchema};
use radix_engine_common::prelude::replace_self_package_address;
use radix_engine_common::types::PackageAddress;
use radix_engine_common::{ManifestSbor, ScryptoSbor};
use sbor::rust::prelude::*;
use sbor::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct KeyValueStoreSchema {
    pub schema: ScryptoSchema,
    pub key: LocalTypeIndex,
    pub value: LocalTypeIndex,
    pub can_own: bool, // TODO: Can this be integrated with ScryptoSchema?
}

impl KeyValueStoreSchema {
    pub fn new<K: ScryptoDescribe, V: ScryptoDescribe>(can_own: bool) -> Self {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let key_type_index = aggregator.add_child_type_and_descendents::<K>();
        let value_type_index = aggregator.add_child_type_and_descendents::<V>();
        let schema = generate_full_schema(aggregator);
        Self {
            schema,
            key: key_type_index,
            value: value_type_index,
            can_own,
        }
    }

    pub fn replace_self_package_address(&mut self, package_address: PackageAddress) {
        replace_self_package_address(&mut self.schema, package_address);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub enum Generic {
    Any,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintSchemaInit {
    pub generics: Vec<Generic>,
    pub schema: ScryptoSchema,
    pub state: BlueprintStateSchemaInit,
    pub events: BlueprintEventSchemaInit,
    pub functions: BlueprintFunctionsSchemaInit,
}

impl Default for BlueprintSchemaInit {
    fn default() -> Self {
        Self {
            generics: Vec::new(),
            schema: ScryptoSchema {
                type_kinds: Vec::new(),
                type_metadata: Vec::new(),
                type_validations: Vec::new(),
            },
            state: BlueprintStateSchemaInit::default(),
            events: BlueprintEventSchemaInit::default(),
            functions: BlueprintFunctionsSchemaInit::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, ScryptoSbor, ManifestSbor)]
pub struct BlueprintStateSchemaInit {
    pub fields: Vec<FieldSchema<TypeRef<LocalTypeIndex>>>,
    pub collections: Vec<BlueprintCollectionSchema<TypeRef<LocalTypeIndex>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, ScryptoSbor, ManifestSbor)]
#[sbor(transparent)]
pub struct BlueprintEventSchemaInit {
    pub event_schema: BTreeMap<String, TypeRef<LocalTypeIndex>>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct FunctionSchemaInit {
    pub receiver: Option<ReceiverInfo>,
    pub input: TypeRef<LocalTypeIndex>,
    pub output: TypeRef<LocalTypeIndex>,
    pub export: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, ScryptoSbor, ManifestSbor)]
pub struct BlueprintFunctionsSchemaInit {
    pub functions: BTreeMap<String, FunctionSchemaInit>,
    pub virtual_lazy_load_functions: BTreeMap<u8, String>,
}

impl BlueprintFunctionsSchemaInit {
    pub fn exports(&self) -> Vec<String> {
        let mut exports: Vec<String> = self.functions.values().map(|t| t.export.clone()).collect();
        for export in self.virtual_lazy_load_functions.values() {
            exports.push(export.clone());
        }
        exports
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub enum TypeRef<T> {
    Static(T),   // Type is defined by blueprint
    Generic(u8), // Type bounds is defined by blueprint, the type itself is defined by the instance
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintKeyValueStoreSchema<T> {
    pub key: T,
    pub value: T,
    pub can_own: bool, // TODO: Can this be integrated with ScryptoSchema?
}

impl<T> BlueprintKeyValueStoreSchema<T> {
    pub fn map<U, F: Fn(T) -> U + Copy>(self, f: F) -> BlueprintKeyValueStoreSchema<U> {
        BlueprintKeyValueStoreSchema {
            key: f(self.key),
            value: f(self.value),
            can_own: self.can_own,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintIndexSchema {}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintSortedIndexSchema {}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub enum BlueprintCollectionSchema<T> {
    KeyValueStore(BlueprintKeyValueStoreSchema<T>),
    Index(BlueprintIndexSchema),
    SortedIndex(BlueprintSortedIndexSchema),
}

impl<T> BlueprintCollectionSchema<T> {
    pub fn map<U, F: Fn(T) -> U + Copy>(self, f: F) -> BlueprintCollectionSchema<U> {
        match self {
            BlueprintCollectionSchema::Index(schema) => BlueprintCollectionSchema::Index(schema),
            BlueprintCollectionSchema::SortedIndex(schema) => {
                BlueprintCollectionSchema::SortedIndex(schema)
            }
            BlueprintCollectionSchema::KeyValueStore(schema) => {
                BlueprintCollectionSchema::KeyValueStore(schema.map(f))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum Condition {
    Always,
    IfFeature(String),
    IfOuterFeature(String),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct FieldSchema<V> {
    pub field: V,
    pub condition: Condition,
}

impl FieldSchema<TypeRef<LocalTypeIndex>> {
    pub fn if_feature<I: Into<LocalTypeIndex>, S: ToString>(value: I, feature: S) -> Self {
        FieldSchema {
            field: TypeRef::Static(value.into()),
            condition: Condition::IfFeature(feature.to_string()),
        }
    }

    pub fn if_outer_feature<I: Into<LocalTypeIndex>, S: ToString>(value: I, feature: S) -> Self {
        FieldSchema {
            field: TypeRef::Static(value.into()),
            condition: Condition::IfOuterFeature(feature.to_string()),
        }
    }

    pub fn static_field<I: Into<LocalTypeIndex>>(value: I) -> Self {
        FieldSchema {
            field: TypeRef::Static(value.into()),
            condition: Condition::Always,
        }
    }
}

bitflags! {
    #[derive(Sbor)]
    pub struct RefTypes: u32 {
        const NORMAL = 0b00000001;
        const DIRECT_ACCESS = 0b00000010;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct ReceiverInfo {
    pub receiver: Receiver,
    pub ref_types: RefTypes,
}

impl ReceiverInfo {
    pub fn normal_ref() -> Self {
        Self {
            receiver: Receiver::SelfRef,
            ref_types: RefTypes::NORMAL,
        }
    }

    pub fn normal_ref_mut() -> Self {
        Self {
            receiver: Receiver::SelfRefMut,
            ref_types: RefTypes::NORMAL,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum Receiver {
    SelfRef,
    SelfRefMut,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct InstanceSchema {
    pub schema: ScryptoSchema,
    pub type_index: Vec<LocalTypeIndex>,
}
