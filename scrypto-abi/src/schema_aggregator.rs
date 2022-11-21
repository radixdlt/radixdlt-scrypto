use crate::v2::*;
use indexmap::{IndexMap, IndexSet};
use sbor::rust::collections::*;

pub fn generate_full_linear_schema<T: Schema>() -> FullTypeSchema {
    let mut aggregator = SchemaAggregator::new();
    aggregator.attempt_add_schema(T::SCHEMA_TYPE_REF, T::get_local_type_data());
    T::add_all_dependencies(&mut aggregator);

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
        TypeSchema::I8 => TypeSchema::I8,
        TypeSchema::I16 => TypeSchema::I16,
        TypeSchema::I32 => TypeSchema::I32,
        TypeSchema::I64 => TypeSchema::I64,
        TypeSchema::I128 => TypeSchema::I128,
        TypeSchema::U8 => TypeSchema::U8,
        TypeSchema::U16 => TypeSchema::U16,
        TypeSchema::U32 => TypeSchema::U32,
        TypeSchema::U64 => TypeSchema::U64,
        TypeSchema::U128 => TypeSchema::U128,
        TypeSchema::String => TypeSchema::String,
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
        TypeSchema::Struct { element_types } => TypeSchema::Struct {
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

pub struct SchemaAggregator {
    pub already_read_descendents: HashSet<ComplexTypeHash>,
    pub schemas: IndexMap<ComplexTypeHash, LocalTypeData<TypeRef>>,
}

impl SchemaAggregator {
    pub fn new() -> Self {
        Self {
            schemas: IndexMap::new(),
            already_read_descendents: HashSet::new(),
        }
    }

    pub fn attempt_add_schema(
        &mut self,
        type_hash: TypeRef,
        schema: LocalTypeData<TypeRef>,
    ) -> bool {
        let TypeRef::Complex(complex_type_hash) = type_hash else {
            return false;
        };

        if self.schemas.contains_key(&complex_type_hash) {
            return false;
        }
        self.schemas.insert(complex_type_hash, schema);
        return true;
    }

    pub fn should_read_descendents(&mut self, type_hash: TypeRef) -> bool {
        let TypeRef::Complex(complex_type_hash) = type_hash else {
            return false;
        };

        if self.already_read_descendents.contains(&complex_type_hash) {
            return false;
        }

        self.already_read_descendents.insert(complex_type_hash);
        return true;
    }
}
