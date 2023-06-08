#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

use bitflags::bitflags;
use radix_engine_common::data::scrypto::{ScryptoCustomTypeKind, ScryptoDescribe, ScryptoSchema};
use radix_engine_common::prelude::replace_self_package_address;
use radix_engine_common::types::{GlobalAddress, PackageAddress};
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

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub struct SchemaMethodKey {
    pub module_id: u8,
    pub ident: String,
}

impl SchemaMethodKey {
    pub fn main<S: ToString>(method_ident: S) -> Self {
        Self {
            module_id: 0u8,
            ident: method_ident.to_string(),
        }
    }

    pub fn metadata<S: ToString>(method_ident: S) -> Self {
        Self {
            module_id: 1u8,
            ident: method_ident.to_string(),
        }
    }

    pub fn royalty<S: ToString>(method_ident: S) -> Self {
        Self {
            module_id: 2u8,
            ident: method_ident.to_string(),
        }
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum SchemaMethodPermission {
    Public,
    Protected(Vec<String>),
}

impl<const N: usize> From<[&str; N]> for SchemaMethodPermission {
    fn from(value: [&str; N]) -> Self {
        SchemaMethodPermission::Protected(
            value.to_vec().into_iter().map(|s| s.to_string()).collect(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintSchema {
    pub outer_blueprint: Option<String>,

    /// State Schema
    pub fields: Vec<FieldSchema>,
    pub collections: Vec<BlueprintCollectionSchema>,

    pub dependencies: BTreeSet<GlobalAddress>,
    pub features: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub enum TypeRef {
    Blueprint(LocalTypeIndex),
    Instance(u8),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintKeyValueStoreSchema {
    pub key: TypeRef,
    pub value: TypeRef,
    pub can_own: bool, // TODO: Can this be integrated with ScryptoSchema?
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintIndexSchema {}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintSortedIndexSchema {}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub enum BlueprintCollectionSchema {
    KeyValueStore(BlueprintKeyValueStoreSchema),
    Index(BlueprintIndexSchema),
    SortedIndex(BlueprintSortedIndexSchema),
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
}

pub type FieldSchema = FeaturedSchema<LocalTypeIndex>;
pub type ExportSchema = FeaturedSchema<String>;

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct FunctionSchema {
    pub receiver: Option<ReceiverInfo>,
    pub input: LocalTypeIndex,
    pub output: LocalTypeIndex,
    pub export: ExportSchema,
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

impl Default for BlueprintSchema {
    fn default() -> Self {
        Self {
            outer_blueprint: None,
            fields: Vec::default(),
            collections: Vec::default(),
            dependencies: BTreeSet::default(),
            features: BTreeSet::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct InstanceSchema {
    pub schema: ScryptoSchema,
    pub type_index: Vec<LocalTypeIndex>,
}
