use core::marker::PhantomData;

use crate::v2::*;
use indexmap::{IndexMap, IndexSet};
use sbor::{rust::collections::*, CustomTypeId};

pub fn generate_full_linear_schema<T: Schema<X>, X: CustomTypeId>() -> FullTypeSchema {
    let mut aggregator = SchemaAggregator::new();
    aggregator.attempt_add_schema_and_descendents::<T>();

    let schema_lookup = IndexSet::from_iter(aggregator.schemas.keys().map(|k| k.clone()));

    let mapped = aggregator
        .schemas
        .into_iter()
        .map(|(_, schema)| {
            // Map the LocalTypeData<SchemaTypeId> into LocalTypeData<usize>
            (linearize(&schema_lookup, schema.schema), schema.naming)
        })
        .unzip();

    FullTypeSchema {
        custom_types: mapped.0,
        naming: mapped.1,
    }
}

fn linearize(
    schemas: &IndexSet<ComplexTypeHash>,
    type_schema: TypeSchema<TypeRef>,
) -> TypeSchema<isize> {
    match type_schema {
        TypeSchema::Any => TypeSchema::Any,
        TypeSchema::Unit => TypeSchema::Unit,
        TypeSchema::Bool => TypeSchema::Bool,
        TypeSchema::I8 { validation } => TypeSchema::I8 { validation },
        TypeSchema::I16 { validation } => TypeSchema::I16 { validation },
        TypeSchema::I32 { validation } => TypeSchema::I32 { validation },
        TypeSchema::I64 { validation } => TypeSchema::I64 { validation },
        TypeSchema::I128 { validation } => TypeSchema::I128 { validation },
        TypeSchema::U8 { validation } => TypeSchema::U8 { validation },
        TypeSchema::U16 { validation } => TypeSchema::U16 { validation },
        TypeSchema::U32 { validation } => TypeSchema::U32 { validation },
        TypeSchema::U64 { validation } => TypeSchema::U64 { validation },
        TypeSchema::U128 { validation } => TypeSchema::U128 { validation },
        TypeSchema::String { length_validation } => TypeSchema::String { length_validation },
        TypeSchema::Array {
            element_sbor_type_id,
            element_type,
            length_validation,
        } => TypeSchema::Array {
            element_sbor_type_id,
            element_type: resolve(schemas, &element_type),
            length_validation,
        },
        TypeSchema::Tuple { element_types } => TypeSchema::Tuple {
            element_types: element_types
                .into_iter()
                .map(|t| resolve(schemas, &t))
                .collect(),
        },
        TypeSchema::Enum { variants } => TypeSchema::Enum {
            variants: variants
                .into_iter()
                .map(|(k, v)| (k, resolve(schemas, &v)))
                .collect(),
        },
        TypeSchema::PackageAddress => TypeSchema::PackageAddress,
        TypeSchema::ComponentAddress => TypeSchema::ComponentAddress,
        TypeSchema::ResourceAddress => TypeSchema::ResourceAddress,
        TypeSchema::SystemAddress => TypeSchema::SystemAddress,
        TypeSchema::Component => TypeSchema::Component,
        TypeSchema::KeyValueStore {
            key_type,
            value_type,
        } => TypeSchema::KeyValueStore {
            key_type: resolve(schemas, &key_type),
            value_type: resolve(schemas, &value_type),
        },
        TypeSchema::Bucket => TypeSchema::Bucket,
        TypeSchema::Proof => TypeSchema::Proof,
        TypeSchema::Vault => TypeSchema::Vault,
        TypeSchema::Expression => TypeSchema::Expression,
        TypeSchema::Blob => TypeSchema::Blob,
        TypeSchema::NonFungibleAddress => TypeSchema::NonFungibleAddress,
        TypeSchema::Hash => TypeSchema::Hash,
        TypeSchema::EcdsaSecp256k1PublicKey => TypeSchema::EcdsaSecp256k1PublicKey,
        TypeSchema::EcdsaSecp256k1Signature => TypeSchema::EcdsaSecp256k1Signature,
        TypeSchema::EddsaEd25519PublicKey => TypeSchema::EddsaEd25519PublicKey,
        TypeSchema::EddsaEd25519Signature => TypeSchema::EddsaEd25519Signature,
        TypeSchema::Decimal => TypeSchema::Decimal,
        TypeSchema::PreciseDecimal => TypeSchema::PreciseDecimal,
        TypeSchema::NonFungibleId => TypeSchema::NonFungibleId,
    }
}

fn resolve(schemas: &IndexSet<ComplexTypeHash>, type_ref: &TypeRef) -> isize {
    match type_ref {
        TypeRef::WellKnownType([well_known_index]) => well_known_index_to_isize(*well_known_index),
        TypeRef::Complex(type_hash) => resolve_index(schemas, type_hash),
    }
}

fn resolve_index(schemas: &IndexSet<ComplexTypeHash>, type_hash: &ComplexTypeHash) -> isize {
    schemas.get_index_of(type_hash)
        .unwrap_or_else(|| panic!("Something went wrong in the schema aggregation process - type hash wasn't added: {:?}", type_hash))
        .try_into()
        .unwrap_or_else(|err| panic!("Too many types to map usize into isize: {:?}", err))
}

pub struct SchemaAggregator<X: CustomTypeId> {
    pub already_read_descendents: HashSet<ComplexTypeHash>,
    pub schemas: IndexMap<ComplexTypeHash, LocalTypeData<TypeRef>>,
    custom_type_id: PhantomData<X>,
}

impl<X: CustomTypeId> SchemaAggregator<X> {
    pub fn new() -> Self {
        Self {
            schemas: IndexMap::new(),
            already_read_descendents: HashSet::new(),
            custom_type_id: PhantomData,
        }
    }


    /// Adds the dependent type (and its dependencies) to the SchemaAggregator.
    pub fn attempt_add_schema_and_descendents<T: Schema<X>>(&mut self) {
        self.attempt_add_local_schema(T::SCHEMA_TYPE_REF, || T::get_local_type_data());
        self.attempt_add_schema_descendents::<T>();
    }

    /// Adds the non-well-known type's `LocalTypeData` to the SchemaAggregator.
    ///
    /// Typically you should use `attempt_add_schema_and_descendents`, unless the local schema has been mutated.
    pub fn attempt_add_local_schema(
        &mut self,
        type_hash: TypeRef,
        get_schema: impl FnOnce() -> Option<LocalTypeData<TypeRef>>,
    ) -> bool {
        let TypeRef::Complex(complex_type_hash) = type_hash else {
            return false;
        };

        if self.schemas.contains_key(&complex_type_hash) {
            return false;
        }
        let schema = get_schema().expect("Schema with a complex TypeRef did not have a LocalTypeData");
        self.schemas.insert(complex_type_hash, schema);
        return true;
    }

    /// Adds the type's descendent types to the SchemaAggregator.
    ///
    /// Typically you should use `attempt_add_schema_and_descendents`, unless the local schema has been mutated.
    pub fn attempt_add_schema_descendents<T: Schema<X>>(&mut self) -> bool {
        let TypeRef::Complex(complex_type_hash) = T::SCHEMA_TYPE_REF else {
            return false;
        };

        if self.already_read_descendents.contains(&complex_type_hash) {
            return false;
        }

        self.already_read_descendents.insert(complex_type_hash);

        T::add_all_dependencies(self);

        return true;
    }
}
