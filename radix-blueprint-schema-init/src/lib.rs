#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

use bitflags::bitflags;
use radix_common::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct KeyValueStoreGenericSubstitutions {
    pub key_generic_substitution: GenericSubstitution,
    pub value_generic_substitution: GenericSubstitution,
    pub allow_ownership: bool, // TODO: Can this be integrated with ScryptoSchema?
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub enum GenericBound {
    Any,
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum BlueprintHook {
    OnVirtualize,
    OnMove,
    OnDrop,
}

/// An initialization object which describes a blueprint's schema including interface, state, and events
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintSchemaInit {
    /// List of generic parameters which must be provided on component instantiation and the bounds of these generics
    pub generics: Vec<GenericBound>,
    /// Sbor schema which describes various types, each identified by a usize
    pub schema: VersionedScryptoSchema,
    /// Describes schema of state by mapping fields/collection indices as a generic or directly into the Sbor schema
    pub state: BlueprintStateSchemaInit,
    /// Describes schema of events by mapping event names as a generic or directly into the Sbor schema
    pub events: BlueprintEventSchemaInit,
    /// Describes schema of types by mapping types names directly into the Sbor schema
    /// These types are used for external generic substitution
    pub types: BlueprintTypeSchemaInit,
    /// Describes interface of function by mapping function names to input/output schema and the code exported function name it maps to
    pub functions: BlueprintFunctionsSchemaInit,
    /// Maps hooks to a code exported function name
    pub hooks: BlueprintHooksInit,
}

impl Default for BlueprintSchemaInit {
    fn default() -> Self {
        Self {
            generics: Vec::new(),
            schema: Schema {
                type_kinds: Vec::new(),
                type_metadata: Vec::new(),
                type_validations: Vec::new(),
            }
            .into_versioned(),
            state: BlueprintStateSchemaInit::default(),
            events: BlueprintEventSchemaInit::default(),
            types: BlueprintTypeSchemaInit::default(),
            functions: BlueprintFunctionsSchemaInit::default(),
            hooks: BlueprintHooksInit::default(),
        }
    }
}

/// Describes the fields and collections some Blueprint has as well
/// as the schema and properties of each field and collection
#[derive(Debug, Clone, PartialEq, Eq, Default, ScryptoSbor, ManifestSbor)]
pub struct BlueprintStateSchemaInit {
    pub fields: Vec<FieldSchema<TypeRef<LocalTypeId>>>,
    pub collections: Vec<BlueprintCollectionSchema<TypeRef<LocalTypeId>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, ScryptoSbor, ManifestSbor)]
#[sbor(transparent)]
pub struct BlueprintEventSchemaInit {
    pub event_schema: IndexMap<String, TypeRef<LocalTypeId>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, ScryptoSbor, ManifestSbor)]
#[sbor(transparent)]
pub struct BlueprintTypeSchemaInit {
    pub type_schema: IndexMap<String, LocalTypeId>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct FunctionSchemaInit {
    pub receiver: Option<ReceiverInfo>,
    pub input: TypeRef<LocalTypeId>,
    pub output: TypeRef<LocalTypeId>,
    pub export: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, ScryptoSbor, ManifestSbor)]
pub struct BlueprintFunctionsSchemaInit {
    pub functions: IndexMap<String, FunctionSchemaInit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, ScryptoSbor, ManifestSbor)]
pub struct BlueprintHooksInit {
    // TODO: allow registration of variant count if we make virtualizable entity type fully dynamic
    pub hooks: IndexMap<BlueprintHook, String>,
}

impl BlueprintSchemaInit {
    pub fn exports(&self) -> Vec<String> {
        self.functions
            .functions
            .values()
            .map(|t| t.export.clone())
            .chain(self.hooks.hooks.values().cloned())
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub enum TypeRef<T> {
    Static(T), // Fully Resolved type is defined in package
    Generic(u8), // Fully Resolved type is mapped directly to a generic
               // TODO: How to represent a structure containing a generic?
}

impl<T> TypeRef<T> {
    pub fn into_static(self) -> Option<T> {
        let Self::Static(value) = self else {
            return None;
        };
        Some(value)
    }

    pub fn assert_static(self) -> T {
        self.into_static().expect("Must be static")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct BlueprintKeyValueSchema<T> {
    pub key: T,
    pub value: T,
    pub allow_ownership: bool,
}

impl<T> BlueprintKeyValueSchema<T> {
    pub fn map<U, F: Fn(T) -> U + Copy>(self, f: F) -> BlueprintKeyValueSchema<U> {
        BlueprintKeyValueSchema {
            key: f(self.key),
            value: f(self.value),
            allow_ownership: self.allow_ownership,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub enum BlueprintCollectionSchema<T> {
    KeyValueStore(BlueprintKeyValueSchema<T>),
    Index(BlueprintKeyValueSchema<T>),
    SortedIndex(BlueprintKeyValueSchema<T>),
}

impl<T> BlueprintCollectionSchema<T> {
    pub fn map<U, F: Fn(T) -> U + Copy>(self, f: F) -> BlueprintCollectionSchema<U> {
        match self {
            BlueprintCollectionSchema::Index(schema) => {
                BlueprintCollectionSchema::Index(schema.map(f))
            }
            BlueprintCollectionSchema::SortedIndex(schema) => {
                BlueprintCollectionSchema::SortedIndex(schema.map(f))
            }
            BlueprintCollectionSchema::KeyValueStore(schema) => {
                BlueprintCollectionSchema::KeyValueStore(schema.map(f))
            }
        }
    }
}

pub trait BlueprintFeature {
    fn feature_name(&self) -> &'static str;
}

/// Expresses a condition based on features enabled on a component
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum Condition {
    Always,
    IfFeature(String),
    IfOuterFeature(String),
}

impl Condition {
    pub fn if_feature(feature: impl BlueprintFeature) -> Self {
        Self::IfFeature(feature.feature_name().into())
    }

    pub fn if_outer_feature(feature: impl BlueprintFeature) -> Self {
        Self::IfOuterFeature(feature.feature_name().into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum FieldTransience {
    NotTransient,
    // TODO: Will need to change this Vec<u8> to ScryptoValue to support default values with global references
    TransientStatic {
        /// The default value a transient substate will have on first read
        default_value: Vec<u8>,
    },
}

/// Describes a field for a Blueprint
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct FieldSchema<V> {
    pub field: V,
    /// Condition for this field to exist
    pub condition: Condition,
    /// Describes if this field should be persisted
    pub transience: FieldTransience,
}

impl FieldSchema<TypeRef<LocalTypeId>> {
    pub fn if_feature<I: Into<LocalTypeId>, S: ToString>(value: I, feature: S) -> Self {
        FieldSchema {
            field: TypeRef::Static(value.into()),
            condition: Condition::IfFeature(feature.to_string()),
            transience: FieldTransience::NotTransient,
        }
    }

    pub fn if_outer_feature<I: Into<LocalTypeId>, S: ToString>(value: I, feature: S) -> Self {
        FieldSchema {
            field: TypeRef::Static(value.into()),
            condition: Condition::IfOuterFeature(feature.to_string()),
            transience: FieldTransience::NotTransient,
        }
    }

    pub fn static_field<I: Into<LocalTypeId>>(value: I) -> Self {
        FieldSchema {
            field: TypeRef::Static(value.into()),
            condition: Condition::Always,
            transience: FieldTransience::NotTransient,
        }
    }

    pub fn transient_field<I: Into<LocalTypeId>, E: ScryptoEncode>(
        value: I,
        default_value: E,
    ) -> Self {
        FieldSchema {
            field: TypeRef::Static(value.into()),
            condition: Condition::Always,
            transience: FieldTransience::TransientStatic {
                default_value: scrypto_encode(&default_value).unwrap(),
            },
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
