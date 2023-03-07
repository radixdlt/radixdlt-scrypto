#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

use radix_engine_common::data::scrypto::{ScryptoCustomTypeKind, ScryptoDescribe, ScryptoSchema, ScryptoValue};
use sbor::rust::prelude::*;
use sbor::*;

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct NonFungibleSchema {
    pub schema: ScryptoSchema,
    pub non_fungible: LocalTypeIndex,
}

impl NonFungibleSchema {
    pub fn new() -> Self {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let non_fungible_type = aggregator.add_child_type_and_descendents::<ScryptoValue>();
        let schema = generate_full_schema(aggregator);
        Self {
            schema,
            non_fungible: non_fungible_type,
        }
    }

    pub fn new_schema<N: ScryptoDescribe>() -> Self {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let non_fungible_type = aggregator.add_child_type_and_descendents::<N>();
        let schema = generate_full_schema(aggregator);
        Self {
            schema,
            non_fungible: non_fungible_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct KeyValueStoreSchema {
    pub schema: ScryptoSchema,
    pub key: LocalTypeIndex,
    pub value: LocalTypeIndex,
}

impl KeyValueStoreSchema {
    pub fn new<K: ScryptoDescribe, V: ScryptoDescribe>() -> Self {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let key_type_index = aggregator.add_child_type_and_descendents::<K>();
        let value_type_index = aggregator.add_child_type_and_descendents::<Option<V>>();
        let schema = generate_full_schema(aggregator);
        Self {
            schema,
            key: key_type_index,
            value: value_type_index,
        }
    }
}

// We keep one self-contained schema per blueprint:
// - Easier macro to export schema, as they work at blueprint level
// - Can always combine multiple schemas into one for storage benefits

#[derive(Default, Debug, Clone, PartialEq, Eq, Sbor)]
pub struct PackageSchema {
    pub blueprints: BTreeMap<String, BlueprintSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct BlueprintSchema {
    pub schema: ScryptoSchema,
    /// For each offset, there is a [`LocalTypeIndex`]
    pub substates: BTreeMap<u8, LocalTypeIndex>,
    /// For each function, there is a [`FunctionSchema`]
    pub functions: BTreeMap<String, FunctionSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct FunctionSchema {
    pub receiver: Option<Receiver>,
    pub input: LocalTypeIndex,
    pub output: LocalTypeIndex,
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
            schema: ScryptoSchema {
                type_kinds: vec![],
                type_metadata: vec![],
                type_validations: vec![],
            },
            substates: BTreeMap::default(),
            functions: BTreeMap::default(),
        }
    }
}

impl BlueprintSchema {
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
}
