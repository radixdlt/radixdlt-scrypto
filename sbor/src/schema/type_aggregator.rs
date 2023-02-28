use super::*;
use sbor::rust::prelude::*;

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
    let type_count = aggregator.types.len();
    let type_indices = IndexSet::from_iter(aggregator.types.keys().map(|k| k.clone()));

    let mut type_kinds = Vec::with_capacity(type_count);
    let mut type_metadata = Vec::with_capacity(type_count);
    let mut type_validations = Vec::with_capacity(type_count);
    for (_type_hash, type_data) in aggregator.types {
        type_kinds.push(linearize::<C::CustomTypeExtension>(
            type_data.kind,
            &type_indices,
        ));
        type_metadata.push(type_data.metadata);
        type_validations.push(type_data.validation);
    }

    Schema {
        type_kinds,
        type_metadata,
        type_validations,
    }
}

fn linearize<E: CustomTypeExtension>(
    type_kind: TypeKind<E::CustomValueKind, E::CustomTypeKind<GlobalTypeId>, GlobalTypeId>,
    type_indices: &IndexSet<TypeHash>,
) -> TypeKind<E::CustomValueKind, E::CustomTypeKind<LocalTypeIndex>, LocalTypeIndex> {
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
            element_type: resolve_local_type_index(type_indices, &element_type),
        },
        TypeKind::Tuple { field_types } => TypeKind::Tuple {
            field_types: field_types
                .into_iter()
                .map(|t| resolve_local_type_index(type_indices, &t))
                .collect(),
        },
        TypeKind::Enum { variants } => TypeKind::Enum {
            variants: variants
                .into_iter()
                .map(|(variant_index, field_types)| {
                    let new_field_types = field_types
                        .into_iter()
                        .map(|t| resolve_local_type_index(type_indices, &t))
                        .collect();
                    (variant_index, new_field_types)
                })
                .collect(),
        },
        TypeKind::Map {
            key_type,
            value_type,
        } => TypeKind::Map {
            key_type: resolve_local_type_index(type_indices, &key_type),
            value_type: resolve_local_type_index(type_indices, &value_type),
        },
        TypeKind::Custom(custom_type_kind) => {
            TypeKind::Custom(E::linearize_type_kind(custom_type_kind, type_indices))
        }
    }
}

pub fn resolve_local_type_index(
    type_indices: &IndexSet<TypeHash>,
    type_index: &GlobalTypeId,
) -> LocalTypeIndex {
    match type_index {
        GlobalTypeId::WellKnown([well_known_index]) => LocalTypeIndex::WellKnown(*well_known_index),
        GlobalTypeId::Novel(type_hash) => {
            LocalTypeIndex::SchemaLocalIndex(resolve_index(type_indices, type_hash))
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

pub struct TypeAggregator<C: CustomTypeKind<GlobalTypeId>> {
    already_read_dependencies: BTreeSet<TypeHash>,
    types: IndexMap<TypeHash, TypeData<C, GlobalTypeId>>,
}

impl<C: CustomTypeKind<GlobalTypeId>> TypeAggregator<C> {
    pub fn new() -> Self {
        Self {
            already_read_dependencies: BTreeSet::new(),
            types: IndexMap::default(),
        }
    }

    /// Adds the dependent type (and its dependencies) to the `TypeAggregator`.
    pub fn add_child_type_and_descendents<T: Describe<C>>(&mut self) -> LocalTypeIndex {
        let schema_type_index = self.add_child_type(T::TYPE_ID, || T::type_data());
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
        type_index: GlobalTypeId,
        get_type_data: impl FnOnce() -> Option<TypeData<C, GlobalTypeId>>,
    ) -> LocalTypeIndex {
        let complex_type_hash = match type_index {
            GlobalTypeId::WellKnown([well_known_type_index]) => {
                return LocalTypeIndex::WellKnown(well_known_type_index);
            }
            GlobalTypeId::Novel(complex_type_hash) => complex_type_hash,
        };

        if let Some(index) = self.types.get_index_of(&complex_type_hash) {
            return LocalTypeIndex::SchemaLocalIndex(index);
        }

        let new_index = self.types.len();
        let local_type_data =
            get_type_data().expect("Schema with a complex TypeRef did not have a TypeData");
        self.types.insert(complex_type_hash, local_type_data);
        LocalTypeIndex::SchemaLocalIndex(new_index)
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
        let GlobalTypeId::Novel(complex_type_hash) = T::TYPE_ID else {
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
