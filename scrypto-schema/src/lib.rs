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
pub struct BlueprintSchemaInit {
    pub schema: ScryptoSchema,
    pub state: BlueprintStateSchemaInit,
    pub events: BlueprintEventSchemaInit,
    pub functions: BlueprintFunctionsTemplateInit,
}

impl Default for BlueprintSchemaInit {
    fn default() -> Self {
        Self {
            schema: ScryptoSchema {
                type_kinds: Vec::new(),
                type_metadata: Vec::new(),
                type_validations: Vec::new(),
            },
            state: BlueprintStateSchemaInit::default(),
            events: BlueprintEventSchemaInit::default(),
            functions: BlueprintFunctionsTemplateInit::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, ScryptoSbor, ManifestSbor)]
pub struct BlueprintStateSchemaInit {
    pub fields: Vec<FieldSchema>,
    pub collections: Vec<BlueprintCollectionSchema<LocalTypeIndex>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, ScryptoSbor, ManifestSbor)]
#[sbor(transparent)]
pub struct BlueprintEventSchemaInit {
    pub event_schema: BTreeMap<String, LocalTypeIndex>,
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct FunctionTemplateInit {
    pub receiver: Option<ReceiverInfo>,
    pub input: LocalTypeIndex,
    pub output: LocalTypeIndex,
    pub export: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Sbor)]
pub struct BlueprintFunctionsTemplateInit {
    pub functions: BTreeMap<String, FunctionTemplateInit>,
    pub virtual_lazy_load_functions: BTreeMap<u8, String>,
}

impl BlueprintFunctionsTemplateInit {
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
    Blueprint(T),
    Instance(u8),
}

impl<T> TypeRef<T> {
    pub fn map<U, F: Fn(T) -> U>(self, f: F) -> TypeRef<U> {
        match self {
            TypeRef::Blueprint(v) => TypeRef::Blueprint(f(v)),
            TypeRef::Instance(v) => TypeRef::Instance(v),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintKeyValueStoreSchema<T> {
    pub key: TypeRef<T>,
    pub value: TypeRef<T>,
    pub can_own: bool, // TODO: Can this be integrated with ScryptoSchema?
}

impl<T> BlueprintKeyValueStoreSchema<T> {
    pub fn map<U, F: Fn(T) -> U + Copy>(self, f: F) -> BlueprintKeyValueStoreSchema<U> {
        BlueprintKeyValueStoreSchema {
            key: self.key.map(f),
            value: self.value.map(f),
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
pub enum FeaturedSchema<V> {
    Normal { value: V },
    Conditional { feature: String, value: V },
}

impl<V> FeaturedSchema<V> {
    pub fn normal<I: Into<V>>(value: I) -> Self {
        FeaturedSchema::Normal {
            value: value.into(),
        }
    }

    pub fn value(&self) -> &V {
        match self {
            FeaturedSchema::Normal { value } => value,
            FeaturedSchema::Conditional { value, .. } => value,
        }
    }

    pub fn map<T, F: FnOnce(V) -> T>(self, f: F) -> FeaturedSchema<T> {
        match self {
            Self::Normal { value } => FeaturedSchema::Normal { value: f(value) },
            Self::Conditional { feature, value } => FeaturedSchema::Conditional {
                feature,
                value: f(value),
            },
        }
    }
}

pub type FieldSchema = FeaturedSchema<LocalTypeIndex>;

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
