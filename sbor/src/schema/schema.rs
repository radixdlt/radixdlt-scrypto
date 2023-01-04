use crate::rust::borrow::Cow;
use crate::rust::collections::BTreeMap;
use crate::rust::string::String;
use crate::rust::vec;
use crate::rust::vec::Vec;
use crate::*;

/// The `Schema` trait allows a type to describe how to interpret and validate a corresponding SBOR payload.
///
/// Each unique interpretation/validation of a type should have its own distinct type in the schema.
/// Uniqueness of a type in the schema is defined by its `GlobalTypeRef`.
#[allow(unused_variables)]
pub trait Schema<C: CustomTypeSchema> {
    /// The `TYPE_REF` should denote a unique identifier for this type (once turned into a payload)
    ///
    /// In particular, it should capture the uniqueness of anything relevant to the codec/payload, for example:
    /// * The payloads the codec can decode
    /// * The uniqueness of display instructions applied to the payload. EG if a wrapper type is intended to give
    ///   the value a different display interpretation, this should create a unique identifier.
    ///
    /// Note however that entirely "transparent" types such as pointers/smart pointers/etc are intended to be
    /// transparent to the schema, so should inherit the `GlobalTypeRef` of the wrapped type.
    ///
    /// If needing to generate a new type id, this can be generated via something like:
    /// ```
    /// impl Schema<C: CustomTypeSchema, T1: Schema<C>> for MyType<T1> {
    ///     const SCHEMA_TYPE_REF: GlobalTypeRef = GlobalTypeRef::complex(stringify!(MyType), &[T1::SCHEMA_TYPE_REF]);
    /// #   fn get_local_type_data() { todo!() }
    /// }
    /// ```
    const SCHEMA_TYPE_REF: GlobalTypeRef;

    /// Returns the local schema for the given type, if the TypeRef is Custom
    fn get_local_type_data() -> Option<LocalTypeData<C, GlobalTypeRef>> {
        None
    }

    /// For each type referenced in `get_local_type_data`, we need to ensure that the type and all of its own references
    /// get added to the aggregator.
    ///
    /// For direct/simple type dependencies, simply call `aggregator.add_child_type_and_descendents::<D>()`
    /// for each dependency.
    ///
    /// For more complicated type dependencies, where new types are being created (EG where a dependent type
    /// is being customised/mutated via annotations on the parent type - such as a TypeName override),
    /// then the algorithm should be:
    ///
    /// - Step 1: For each (possibly customised) type dependency needed directly by this type:
    ///   - Create a new mutated `mutated_type_ref` for the underlying type plus its mutation
    ///   - Use `mutated_type_ref` in the relevant place/s in `get_local_type_data`
    ///   - In `add_all_dependencies` add a line `aggregator.add_child_type(mutated_type_ref, mutated_local_type_data)`
    ///
    /// - Step 2: For each (base/unmutated) type dependency `D`:
    ///   - In `add_all_dependencies` add a line `aggregator.add_schema_descendents::<D>()`
    fn add_all_dependencies(aggregator: &mut SchemaAggregator<C>) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalTypeData<C: CustomTypeSchema, L: TypeLink + TypeId<C::CustomTypeId>> {
    pub schema: TypeSchema<C::CustomTypeId, C, L>,
    pub naming: TypeNaming,
}

impl<C: CustomTypeSchema, L: TypeLink + TypeId<C::CustomTypeId>> LocalTypeData<C, L> {
    pub fn new(naming: TypeNaming, schema: TypeSchema<C::CustomTypeId, C, L>) -> Self {
        Self { schema, naming }
    }

    pub fn named_no_child_names(
        name: &'static str,
        schema: TypeSchema<C::CustomTypeId, C, L>,
    ) -> Self {
        Self::new(TypeNaming::named_no_child_names(name), schema)
    }

    pub fn named_unit(name: &'static str) -> Self {
        Self::new(TypeNaming::named_no_child_names(name), TypeSchema::Unit)
    }

    pub fn named_tuple(name: &'static str, field_types: Vec<L>) -> Self {
        Self::new(
            TypeNaming::named_no_child_names(name),
            TypeSchema::Tuple { field_types },
        )
    }

    pub fn named_fields_tuple(name: &'static str, fields: Vec<(&'static str, L)>) -> Self {
        let (field_names, field_types): (Vec<_>, _) = fields.into_iter().unzip();
        Self::new(
            TypeNaming::named_with_fields(name, &field_names),
            TypeSchema::Tuple { field_types },
        )
    }

    pub fn named_enum(name: &'static str, variants: BTreeMap<String, LocalTypeData<C, L>>) -> Self {
        let (variant_naming, variant_tuple_schemas) = variants
            .into_iter()
            .map(|(k, variant_type_data)| {
                let variant_fields_schema = match variant_type_data.schema {
                    TypeSchema::Unit => vec![],
                    TypeSchema::Tuple { field_types } => field_types,
                    _ => panic!("Only Unit and Tuple are allowed in Enum variant LocalTypeData"),
                };
                (
                    (k.clone(), variant_type_data.naming),
                    (k, variant_fields_schema),
                )
            })
            .unzip();
        Self::new(
            TypeNaming::named_with_variants(name, variant_naming),
            TypeSchema::Enum {
                variants: variant_tuple_schemas,
            },
        )
    }
}

/// This enables the type to be represented as eg JSON
/// Also used to facilitate type reconstruction
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeNaming {
    pub type_name: Cow<'static, str>,
    pub child_names: ChildNames,
}

impl TypeNaming {
    pub fn named_no_child_names(name: &'static str) -> Self {
        Self {
            type_name: Cow::Borrowed(name),
            child_names: ChildNames::None,
        }
    }

    pub fn named_with_fields(name: &'static str, field_names: &[&'static str]) -> Self {
        let field_names = field_names
            .iter()
            .map(|field_name| Cow::Borrowed(*field_name))
            .collect();
        Self {
            type_name: Cow::Borrowed(name),
            child_names: ChildNames::FieldNames(field_names),
        }
    }

    pub fn named_with_variants(
        name: &'static str,
        variant_naming: BTreeMap<String, TypeNaming>,
    ) -> Self {
        Self {
            type_name: Cow::Borrowed(name),
            child_names: ChildNames::VariantNames(variant_naming),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ChildNames {
    #[default]
    None,
    FieldNames(Vec<Cow<'static, str>>),
    VariantNames(BTreeMap<String, TypeNaming>),
}

/// An array of custom types, and associated extra information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullTypeSchema<C: CompleteCustomTypeSchema> {
    pub custom_types: Vec<TypeSchema<C::CustomTypeId, C, SchemaLocalTypeRef>>,
    pub naming: Vec<TypeNaming>,
}

// TODO: Could get rid of the Cow by using some per-custom type once_cell to cache basic well-known-types,
//       and return references to the static cached values
pub struct ResolvedLocalTypeData<'a, C: CompleteCustomTypeSchema> {
    pub schema: Cow<'a, TypeSchema<C::CustomTypeId, C, SchemaLocalTypeRef>>,
    pub naming: Cow<'a, TypeNaming>,
}

impl<C: CompleteCustomTypeSchema> FullTypeSchema<C> {
    pub fn resolve<'a>(
        &'a self,
        type_ref: SchemaLocalTypeRef,
    ) -> Option<ResolvedLocalTypeData<'a, C>> {
        match type_ref {
            SchemaLocalTypeRef::WellKnown(index) => {
                resolve_well_known_type_data::<C::WellKnownTypes>(index).map(|local_type_data| {
                    ResolvedLocalTypeData {
                        schema: Cow::Owned(local_type_data.schema),
                        naming: Cow::Owned(local_type_data.naming),
                    }
                })
            }
            SchemaLocalTypeRef::SchemaLocal(index) => {
                match (self.custom_types.get(index), self.naming.get(index)) {
                    (Some(schema), Some(naming)) => Some(ResolvedLocalTypeData {
                        schema: Cow::Borrowed(schema),
                        naming: Cow::Borrowed(naming),
                    }),
                    (None, None) => None,
                    _ => panic!("Index existed in exactly one of schema and naming"),
                }
            }
        }
    }
}
