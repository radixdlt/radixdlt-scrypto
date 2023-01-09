use super::*;
use sbor::rust::collections::*;

pub fn generate_full_schema_from_single_type<
    T: Describe<E::CustomTypeKind<GlobalTypeId>>,
    E: CustomTypeExtension,
>() -> (LocalTypeIndex, Schema<E>) {
    let mut aggregator = TypeAggregator::new();
    let type_index = aggregator.add_child_type_and_descendents::<T>();
    (type_index, generate_full_schema(aggregator))
}

pub fn generate_full_schema<C: CustomTypeKind<GlobalTypeId>>(
    aggregator: TypeAggregator<C>,
) -> Schema<C::CustomTypeExtension> {
    let schema_lookup = IndexSet::from_iter(aggregator.types.keys().map(|k| k.clone()));

    let mapped = aggregator
        .types
        .into_iter()
        .map(|(type_hash, type_data)| {
            (
                linearize::<C::CustomTypeExtension>(type_data.kind, &schema_lookup),
                type_data.metadata.with_type_hash(type_hash),
            )
        })
        .unzip();

    Schema {
        type_kinds: mapped.0,
        type_metadata: mapped.1,
    }
}

fn linearize<E: CustomTypeExtension>(
    type_kind: TypeKind<E::CustomTypeId, E::CustomTypeKind<GlobalTypeId>, GlobalTypeId>,
    schemas: &IndexSet<TypeHash>,
) -> TypeKind<E::CustomTypeId, E::CustomTypeKind<LocalTypeIndex>, LocalTypeIndex> {
    match type_kind {
        TypeKind::Any => TypeKind::Any,
        TypeKind::Unit => TypeKind::Unit,
        TypeKind::Bool => TypeKind::Bool,
        TypeKind::I8 { validation } => TypeKind::I8 { validation },
        TypeKind::I16 { validation } => TypeKind::I16 { validation },
        TypeKind::I32 { validation } => TypeKind::I32 { validation },
        TypeKind::I64 { validation } => TypeKind::I64 { validation },
        TypeKind::I128 { validation } => TypeKind::I128 { validation },
        TypeKind::U8 { validation } => TypeKind::U8 { validation },
        TypeKind::U16 { validation } => TypeKind::U16 { validation },
        TypeKind::U32 { validation } => TypeKind::U32 { validation },
        TypeKind::U64 { validation } => TypeKind::U64 { validation },
        TypeKind::U128 { validation } => TypeKind::U128 { validation },
        TypeKind::String { length_validation } => TypeKind::String { length_validation },
        TypeKind::Array {
            element_type,
            length_validation,
        } => TypeKind::Array {
            element_type: resolve_local_type_ref(schemas, &element_type),
            length_validation,
        },
        TypeKind::Tuple { field_types } => TypeKind::Tuple {
            field_types: field_types
                .into_iter()
                .map(|t| resolve_local_type_ref(schemas, &t))
                .collect(),
        },
        TypeKind::Enum { variants } => TypeKind::Enum {
            variants: variants
                .into_iter()
                .map(|(variant_index, field_types)| {
                    let new_field_types = field_types
                        .into_iter()
                        .map(|t| resolve_local_type_ref(schemas, &t))
                        .collect();
                    (variant_index, new_field_types)
                })
                .collect(),
        },
        TypeKind::Custom(custom_type_schema) => {
            TypeKind::Custom(E::linearize_type_kind(custom_type_schema, schemas))
        }
    }
}

pub fn resolve_local_type_ref(
    schemas: &IndexSet<TypeHash>,
    type_ref: &GlobalTypeId,
) -> LocalTypeIndex {
    match type_ref {
        GlobalTypeId::WellKnown([well_known_index]) => LocalTypeIndex::WellKnown(*well_known_index),
        GlobalTypeId::Novel(type_hash) => {
            LocalTypeIndex::SchemaLocalIndex(resolve_index(schemas, type_hash))
        }
    }
}

fn resolve_index(schemas: &IndexSet<TypeHash>, type_hash: &TypeHash) -> usize {
    schemas.get_index_of(type_hash).unwrap_or_else(|| {
        panic!(
            "Fatal error in the schema aggregation process - this is likely due to a Schema impl missing a dependent type in add_all_dependencies. The following type hash wasn't added in add_all_dependencies: {:?}",
            type_hash
        )
    })
}

pub struct TypeAggregator<C: CustomTypeKind<GlobalTypeId>> {
    pub already_read_dependencies: HashSet<TypeHash>,
    pub types: IndexMap<TypeHash, TypeData<C, GlobalTypeId>>,
}

impl<C: CustomTypeKind<GlobalTypeId>> TypeAggregator<C> {
    pub fn new() -> Self {
        Self {
            types: IndexMap::new(),
            already_read_dependencies: HashSet::new(),
        }
    }

    /// Adds the dependent type (and its dependencies) to the `TypeAggregator`.
    pub fn add_child_type_and_descendents<T: Describe<C>>(&mut self) -> LocalTypeIndex {
        let schema_type_index =
            self.add_child_type(T::SCHEMA_TYPE_REF, || T::get_local_type_data());
        self.add_schema_descendents::<T>();
        schema_type_index
    }

    /// Adds the type's `TypeData` to the `TypeAggregator`.
    ///
    /// If the type is well known or already in the aggregator, this returns early with the existing index.
    ///
    /// Typically you should use [`add_schema_and_descendents`], unless you're customising the schemas you add -
    /// in which case, you likely wish to call [`add_child_type`] and [`add_schema_descendents`] separately.
    ///
    /// [`add_child_type`]: #method.add_child_type
    /// [`add_schema_descendents`]: #method.add_schema_descendents
    /// [`add_schema_and_descendents`]: #method.add_schema_and_descendents
    pub fn add_child_type(
        &mut self,
        type_ref: GlobalTypeId,
        get_type_data: impl FnOnce() -> Option<TypeData<C, GlobalTypeId>>,
    ) -> LocalTypeIndex {
        let complex_type_hash = match type_ref {
            GlobalTypeId::WellKnown([well_known_type_index]) => {
                return LocalTypeIndex::WellKnown(well_known_type_index);
            }
            GlobalTypeId::Novel(complex_type_hash) => complex_type_hash,
        };

        if let Some(index) = self.types.get_index_of(&complex_type_hash) {
            return LocalTypeIndex::SchemaLocalIndex(index);
        }

        let local_type_data =
            get_type_data().expect("Schema with a complex TypeRef did not have a TypeData");

        self.types.insert(complex_type_hash, local_type_data);
        let new_type_index = self
            .types
            .get_index_of(&complex_type_hash)
            .expect("Schema that was just inserted isn't in map");

        LocalTypeIndex::SchemaLocalIndex(new_type_index)
    }

    /// Adds the type's descendent types to the `TypeAggregator`, if they've not already been added.
    ///
    /// Typically you should use [`add_schema_and_descendents`], unless you're customising the schemas you add -
    /// in which case, you likely wish to call [`add_child_type`] and [`add_schema_descendents`] separately.
    ///
    /// [`add_child_type`]: #method.add_child_type
    /// [`add_schema_descendents`]: #method.add_schema_descendents
    /// [`add_schema_and_descendents`]: #method.add_schema_and_descendents
    pub fn add_schema_descendents<T: Describe<C>>(&mut self) -> bool {
        let GlobalTypeId::Novel(complex_type_hash) = T::SCHEMA_TYPE_REF else {
            return false;
        };

        if self.already_read_dependencies.contains(&complex_type_hash) {
            return false;
        }

        self.already_read_dependencies.insert(complex_type_hash);

        T::add_all_dependencies(self);

        return true;
    }
}
