use super::*;
use sbor::rust::collections::*;

pub fn generate_full_schema_from_single_type<T: Schema<C>, C: LinearizableCustomTypeSchema>(
) -> (SchemaLocalTypeRef, FullTypeSchema<C::Linearized>) {
    let mut aggregator = SchemaAggregator::new();
    let type_index = aggregator.add_child_type_and_descendents::<T>();
    (type_index, generate_full_schema(aggregator))
}

pub fn generate_full_schema<C: LinearizableCustomTypeSchema>(
    aggregator: SchemaAggregator<C>,
) -> FullTypeSchema<C::Linearized> {
    let schema_lookup = IndexSet::from_iter(aggregator.types.keys().map(|k| k.clone()));

    let mapped = aggregator
        .types
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

fn linearize<C: LinearizableCustomTypeSchema>(
    schemas: &IndexSet<ComplexTypeHash>,
    type_schema: TypeSchema<C::CustomTypeId, C, GlobalTypeRef>,
) -> TypeSchema<C::CustomTypeId, C::Linearized, SchemaLocalTypeRef> {
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
            element_type: resolve_local_type_ref(schemas, &element_type),
            length_validation,
        },
        TypeSchema::Tuple { element_types } => TypeSchema::Tuple {
            element_types: element_types
                .into_iter()
                .map(|t| resolve_local_type_ref(schemas, &t))
                .collect(),
        },
        TypeSchema::Enum { variants } => TypeSchema::Enum {
            variants: variants
                .into_iter()
                .map(|(k, v)| (k, resolve_local_type_ref(schemas, &v)))
                .collect(),
        },
        TypeSchema::Custom(custom_type_schema) => {
            TypeSchema::Custom(custom_type_schema.linearize(schemas))
        }
    }
}

pub fn resolve_local_type_ref(
    schemas: &IndexSet<ComplexTypeHash>,
    type_ref: &GlobalTypeRef,
) -> SchemaLocalTypeRef {
    match type_ref {
        GlobalTypeRef::WellKnown([well_known_index]) => {
            SchemaLocalTypeRef::WellKnown(*well_known_index)
        }
        GlobalTypeRef::Complex(type_hash) => {
            SchemaLocalTypeRef::SchemaLocal(resolve_index(schemas, type_hash))
        }
    }
}

fn resolve_index(schemas: &IndexSet<ComplexTypeHash>, type_hash: &ComplexTypeHash) -> usize {
    schemas.get_index_of(type_hash).unwrap_or_else(|| {
        panic!(
            "Something went wrong in the schema aggregation process - type hash wasn't added: {:?}",
            type_hash
        )
    })
}

pub struct SchemaAggregator<C: CustomTypeSchema> {
    pub already_read_dependencies: HashSet<ComplexTypeHash>,
    pub types: IndexMap<ComplexTypeHash, LocalTypeData<C, GlobalTypeRef>>,
}

impl<C: CustomTypeSchema> SchemaAggregator<C> {
    pub fn new() -> Self {
        Self {
            types: IndexMap::new(),
            already_read_dependencies: HashSet::new(),
        }
    }

    /// Adds the dependent type (and its dependencies) to the `SchemaAggregator`.
    pub fn add_child_type_and_descendents<T: Schema<C>>(&mut self) -> SchemaLocalTypeRef {
        let schema_type_index =
            self.add_child_type(T::SCHEMA_TYPE_REF, || T::get_local_type_data());
        self.add_schema_descendents::<T>();
        schema_type_index
    }

    /// Adds the type's `LocalTypeData` to the `SchemaAggregator`.
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
        type_ref: GlobalTypeRef,
        get_type_data: impl FnOnce() -> Option<LocalTypeData<C, GlobalTypeRef>>,
    ) -> SchemaLocalTypeRef {
        let complex_type_hash = match type_ref {
            GlobalTypeRef::WellKnown([well_known_type_index]) => {
                return SchemaLocalTypeRef::WellKnown(well_known_type_index);
            }
            GlobalTypeRef::Complex(complex_type_hash) => complex_type_hash,
        };

        if let Some(index) = self.types.get_index_of(&complex_type_hash) {
            return SchemaLocalTypeRef::SchemaLocal(index);
        }

        let local_type_data =
            get_type_data().expect("Schema with a complex TypeRef did not have a LocalTypeData");

        self.types.insert(complex_type_hash, local_type_data);
        let new_type_index = self
            .types
            .get_index_of(&complex_type_hash)
            .expect("Schema that was just inserted isn't in map");

        SchemaLocalTypeRef::SchemaLocal(new_type_index)
    }

    /// Adds the type's descendent types to the `SchemaAggregator`, if they've not already been added.
    ///
    /// Typically you should use [`add_schema_and_descendents`], unless you're customising the schemas you add -
    /// in which case, you likely wish to call [`add_child_type`] and [`add_schema_descendents`] separately.
    ///
    /// [`add_child_type`]: #method.add_child_type
    /// [`add_schema_descendents`]: #method.add_schema_descendents
    /// [`add_schema_and_descendents`]: #method.add_schema_and_descendents
    pub fn add_schema_descendents<T: Schema<C>>(&mut self) -> bool {
        let GlobalTypeRef::Complex(complex_type_hash) = T::SCHEMA_TYPE_REF else {
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
