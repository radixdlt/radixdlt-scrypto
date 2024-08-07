use super::*;
use sbor::rust::prelude::*;

pub fn generate_full_schema_from_single_type<
    T: Describe<S::CustomAggregatorTypeKind> + ?Sized,
    S: CustomSchema,
>() -> (LocalTypeId, VersionedSchema<S>) {
    let mut aggregator = TypeAggregator::new();
    let type_id = aggregator.add_child_type_and_descendents::<T>();
    (type_id, generate_full_schema(aggregator))
}

pub fn generate_single_type_schema<
    T: Describe<S::CustomAggregatorTypeKind> + ?Sized,
    S: CustomSchema,
>() -> SingleTypeSchema<S> {
    let (type_id, schema) = generate_full_schema_from_single_type::<T, S>();
    SingleTypeSchema::new(schema, type_id)
}

/// You may wish to use the newer `aggregator.generate_type_collection_schema()`
/// which, in tandom with `add_named_root_type_and_descendents`
/// also captures named root types to give more structure to enable schema
/// comparisons over time.
pub fn generate_full_schema<S: CustomSchema>(
    aggregator: TypeAggregator<S::CustomAggregatorTypeKind>,
) -> VersionedSchema<S> {
    generate_schema_from_types(aggregator.types)
}

fn generate_schema_from_types<S: CustomSchema>(
    types: IndexMap<TypeHash, AggregatorTypeData<S>>,
) -> VersionedSchema<S> {
    let type_count = types.len();
    let type_indices = IndexSet::from_iter(types.keys().map(|k| k.clone()));

    let mut type_kinds = Vec::with_capacity(type_count);
    let mut type_metadata = Vec::with_capacity(type_count);
    let mut type_validations = Vec::with_capacity(type_count);
    for (_type_hash, type_data) in types {
        type_kinds.push(linearize::<S>(type_data.kind, &type_indices));
        type_metadata.push(type_data.metadata);
        type_validations.push(type_data.validation);
    }

    Schema {
        type_kinds,
        type_metadata,
        type_validations,
    }
    .into_versioned()
}

pub fn localize_well_known_type_data<S: CustomSchema>(
    type_data: AggregatorTypeData<S>,
) -> LocalTypeData<S> {
    let TypeData {
        kind,
        metadata,
        validation,
    } = type_data;
    TypeData {
        kind: linearize::<S>(kind, &indexset!()),
        metadata,
        validation,
    }
}

pub fn localize_well_known<S: CustomSchema>(type_kind: AggregatorTypeKind<S>) -> LocalTypeKind<S> {
    linearize::<S>(type_kind, &indexset!())
}

fn linearize<S: CustomSchema>(
    type_kind: AggregatorTypeKind<S>,
    type_indices: &IndexSet<TypeHash>,
) -> LocalTypeKind<S> {
    match type_kind {
        TypeKind::Any => TypeKind::Any,
        TypeKind::Bool => TypeKind::Bool,
        TypeKind::I8 => TypeKind::I8,
        TypeKind::I16 => TypeKind::I16,
        TypeKind::I32 => TypeKind::I32,
        TypeKind::I64 => TypeKind::I64,
        TypeKind::I128 => TypeKind::I128,
        TypeKind::U8 => TypeKind::U8,
        TypeKind::U16 => TypeKind::U16,
        TypeKind::U32 => TypeKind::U32,
        TypeKind::U64 => TypeKind::U64,
        TypeKind::U128 => TypeKind::U128,
        TypeKind::String => TypeKind::String,
        TypeKind::Array { element_type } => TypeKind::Array {
            element_type: resolve_local_type_id(type_indices, &element_type),
        },
        TypeKind::Tuple { field_types } => TypeKind::Tuple {
            field_types: field_types
                .into_iter()
                .map(|t| resolve_local_type_id(type_indices, &t))
                .collect(),
        },
        TypeKind::Enum { variants } => TypeKind::Enum {
            variants: variants
                .into_iter()
                .map(|(variant_index, field_types)| {
                    let new_field_types = field_types
                        .into_iter()
                        .map(|t| resolve_local_type_id(type_indices, &t))
                        .collect();
                    (variant_index, new_field_types)
                })
                .collect(),
        },
        TypeKind::Map {
            key_type,
            value_type,
        } => TypeKind::Map {
            key_type: resolve_local_type_id(type_indices, &key_type),
            value_type: resolve_local_type_id(type_indices, &value_type),
        },
        TypeKind::Custom(custom_type_kind) => {
            TypeKind::Custom(S::linearize_type_kind(custom_type_kind, type_indices))
        }
    }
}

pub fn resolve_local_type_id(
    type_indices: &IndexSet<TypeHash>,
    type_id: &RustTypeId,
) -> LocalTypeId {
    match type_id {
        RustTypeId::WellKnown(well_known_type_id) => LocalTypeId::WellKnown(*well_known_type_id),
        RustTypeId::Novel(type_hash) => {
            LocalTypeId::SchemaLocalIndex(resolve_index(type_indices, type_hash))
        }
    }
}

fn resolve_index(type_indices: &IndexSet<TypeHash>, type_hash: &TypeHash) -> usize {
    type_indices.get_index_of(type_hash).unwrap_or_else(|| {
        panic!(
            "Fatal error in the type aggregation process - this is likely due to a type impl missing a dependent type in add_all_dependencies. The following type hash wasn't added in add_all_dependencies: {:?}",
            type_hash
        )
    })
}

pub struct TypeAggregator<C: CustomTypeKind<RustTypeId>> {
    already_read_dependencies: IndexSet<TypeHash>,
    named_root_types: IndexMap<String, LocalTypeId>,
    types: IndexMap<TypeHash, TypeData<C, RustTypeId>>,
}

impl<C: CustomTypeKind<RustTypeId>> TypeAggregator<C> {
    pub fn new() -> Self {
        Self {
            already_read_dependencies: index_set_new(),
            named_root_types: IndexMap::default(),
            types: IndexMap::default(),
        }
    }

    /// Adds the type (and its dependencies) to the `TypeAggregator`.
    /// Also tracks it as a named root type, which can be used e.g. in schema comparisons.
    ///
    /// This is only intended for use when adding root types to schemas,
    /// and should not be called from inside `Describe` implementations.
    pub fn add_root_type<T: Describe<C> + ?Sized>(
        &mut self,
        name: impl Into<String>,
    ) -> LocalTypeId {
        let local_type_id = self.add_child_type_and_descendents::<T>();
        self.named_root_types.insert(name.into(), local_type_id);
        local_type_id
    }

    /// Adds the dependent type (and its dependencies) to the `TypeAggregator`.
    pub fn add_child_type_and_descendents<T: Describe<C> + ?Sized>(&mut self) -> LocalTypeId {
        let schema_type_id = self.add_child_type(T::TYPE_ID, || T::type_data());
        self.add_schema_descendents::<T>();
        schema_type_id
    }

    /// Adds the type's `TypeData` to the `TypeAggregator`.
    ///
    /// If the type is well known or already in the aggregator, this returns early with the existing index.
    ///
    /// Typically you should use [`add_child_type_and_descendents`], unless you're replacing/mutating
    /// the child types somehow. In which case, you'll likely wish to call [`add_child_type`] and
    /// [`add_schema_descendents`] separately.
    ///
    /// [`add_child_type`]: #method.add_child_type
    /// [`add_schema_descendents`]: #method.add_schema_descendents
    /// [`add_child_type_and_descendents`]: #method.add_child_type_and_descendents
    pub fn add_child_type(
        &mut self,
        type_id: RustTypeId,
        get_type_data: impl FnOnce() -> TypeData<C, RustTypeId>,
    ) -> LocalTypeId {
        let complex_type_hash = match type_id {
            RustTypeId::WellKnown(well_known_type_id) => {
                return LocalTypeId::WellKnown(well_known_type_id);
            }
            RustTypeId::Novel(complex_type_hash) => complex_type_hash,
        };

        if let Some(index) = self.types.get_index_of(&complex_type_hash) {
            return LocalTypeId::SchemaLocalIndex(index);
        }

        let new_index = self.types.len();
        self.types.insert(complex_type_hash, get_type_data());
        LocalTypeId::SchemaLocalIndex(new_index)
    }

    /// Adds the type's descendent types to the `TypeAggregator`, if they've not already been added.
    ///
    /// Typically you should use [`add_child_type_and_descendents`], unless you're replacing/mutating
    /// the child types somehow. In which case, you'll likely wish to call [`add_child_type`] and
    /// [`add_schema_descendents`] separately.
    ///
    /// [`add_child_type`]: #method.add_child_type
    /// [`add_schema_descendents`]: #method.add_schema_descendents
    /// [`add_child_type_and_descendents`]: #method.add_child_type_and_descendents
    pub fn add_schema_descendents<T: Describe<C> + ?Sized>(&mut self) -> bool {
        let RustTypeId::Novel(complex_type_hash) = T::TYPE_ID else {
            return false;
        };

        if self.already_read_dependencies.contains(&complex_type_hash) {
            return false;
        }

        self.already_read_dependencies.insert(complex_type_hash);

        T::add_all_dependencies(self);

        return true;
    }

    pub fn generate_type_collection_schema<S: CustomSchema<CustomAggregatorTypeKind = C>>(
        self,
    ) -> TypeCollectionSchema<S> {
        TypeCollectionSchema::new(
            generate_schema_from_types(self.types),
            self.named_root_types,
        )
    }
}
