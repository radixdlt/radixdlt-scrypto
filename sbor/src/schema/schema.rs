use crate::*;
use sbor::rust::borrow::Cow;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

/// The `Schema` trait allows a type to describe how to interpret and validate a corresponding SBOR payload.
///
/// Each unique interpretation/validation of a type should have its own distinct type in the schema.
/// Uniqueness of a type in the schema is defined by its TypeRef.
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
    /// transparent to the schema, so should inherit their wrapped type id.
    ///
    /// If needing to generate a new type id, this can be generated via something like:
    /// ```
    /// impl Schema for MyType {
    ///     const SCHEMA_TYPE_REF: GlobalTypeRef = GlobalTypeRef::complex(stringify!(MyType), &[], &[]);
    /// #   fn get_local_type_data() { todo!() }
    /// }
    /// ```
    const SCHEMA_TYPE_REF: GlobalTypeRef;

    /// Returns the local schema for the given type, if the TypeRef is Custom
    fn get_local_type_data() -> Option<LocalTypeData<C, GlobalTypeRef>> {
        None
    }

    /// Should add all the dependent schemas, if the type depends on any.
    ///
    /// For direct/simple type dependencies, simply call `aggregator.add_child_type_and_descendents::<D>()`
    /// for each dependency.
    ///
    /// For more complicated type dependencies, where new types are being created (EG enum variants, or
    /// where a dependent type ie being customised/mutated via annotations), then the algorithm should be:
    ///
    /// - For each (possibly customised) type dependency needed directly by this type
    ///   - Ensure that if it's customised, then its `type_ref` is mutated from its underlying type
    ///   - Do `aggregator.add_child_type(type_ref, local_type_data)`
    ///
    /// - For each (base/unmutated) type dependency `D`:
    ///   - Call `aggregator.add_schema_descendents::<D>()`
    fn add_all_dependencies(aggregator: &mut SchemaAggregator<C>) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalTypeData<C: CustomTypeSchema, L: TypeLink + TypeId<C::CustomTypeId>> {
    pub schema: TypeSchema<C::CustomTypeId, C, L>,
    pub naming: TypeNaming,
}

impl<C: CustomTypeSchema, L: TypeLink + TypeId<C::CustomTypeId>> LocalTypeData<C, L> {
    pub const fn named(name: &'static str, schema: TypeSchema<C::CustomTypeId, C, L>) -> Self {
        Self {
            schema,
            naming: TypeNaming {
                type_name: Cow::Borrowed(name),
                field_names: None,
            },
        }
    }

    pub const fn named_unit(name: &'static str) -> Self {
        Self {
            schema: TypeSchema::Unit,
            naming: TypeNaming {
                type_name: Cow::Borrowed(name),
                field_names: None,
            },
        }
    }

    pub const fn named_tuple(name: &'static str, element_types: Vec<L>) -> Self {
        Self {
            schema: TypeSchema::Tuple { element_types },
            naming: TypeNaming {
                type_name: Cow::Borrowed(name),
                field_names: None,
            },
        }
    }

    pub fn named_tuple_named_fields(
        name: &'static str,
        element_types: Vec<L>,
        field_names: &[&'static str],
    ) -> Self {
        Self {
            schema: TypeSchema::Tuple { element_types },
            naming: TypeNaming {
                type_name: Cow::Borrowed(name),
                field_names: Some(field_names.iter().map(|x| x.to_string()).collect()),
            },
        }
    }
}

/// This enables the type to be represented as eg JSON
/// Also used to facilitate type reconstruction
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeNaming {
    pub type_name: Cow<'static, str>,
    pub field_names: Option<Vec<String>>,
}

impl TypeNaming {
    pub const fn named(name: &'static str) -> Self {
        Self {
            type_name: Cow::Borrowed(name),
            field_names: None,
        }
    }
}

/// An array of custom types, and associated extra information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullTypeSchema<C: CustomTypeSchema> {
    pub custom_types: Vec<TypeSchema<C::CustomTypeId, C, SchemaLocalTypeRef>>,
    pub naming: Vec<TypeNaming>,
}

pub struct ResolvedLocalTypeData<'a, C: CustomTypeSchema> {
    pub schema: Cow<'a, TypeSchema<C::CustomTypeId, C, SchemaLocalTypeRef>>,
    pub naming: Cow<'a, TypeNaming>,
}

impl<C: CustomTypeSchema> FullTypeSchema<C> {
    pub fn resolve<'a, W: CustomWellKnownType<CustomTypeSchema = C>>(
        &'a self,
        type_ref: SchemaLocalTypeRef,
    ) -> Option<ResolvedLocalTypeData<'a, C>> {
        match type_ref {
            SchemaLocalTypeRef::WellKnown(index) => {
                resolve_well_known_type_data::<W>(index).map(|local_type_data| {
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
