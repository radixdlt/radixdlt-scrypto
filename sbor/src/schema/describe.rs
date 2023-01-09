use crate::*;

/// The `Describe` trait allows a type to describe how to interpret and validate a corresponding SBOR payload.
///
/// Each unique interpretation/validation of a type should have its own distinct type in the schema.
/// Uniqueness of a type in the schema is defined by its `GlobalTypeId`.
#[allow(unused_variables)]
pub trait NewDescribe<C: CustomTypeKind<GlobalTypeId>> {
    /// The `TYPE_REF` should denote a unique identifier for this type (once turned into a payload)
    ///
    /// In particular, it should capture the uniqueness of anything relevant to the codec/payload, for example:
    /// * The payloads the codec can decode
    /// * The uniqueness of display instructions applied to the payload. EG if a wrapper type is intended to give
    ///   the value a different display interpretation, this should create a unique identifier.
    ///
    /// Note however that entirely "transparent" types such as pointers/smart pointers/etc are intended to be
    /// transparent to the schema, so should inherit the `GlobalTypeId` of the wrapped type.
    ///
    /// If needing to generate a new type id, this can be generated via something like:
    /// ```
    /// impl NewDescribe<C: CustomTypeSchema, T1: NewDescribe<C>> for MyType<T1> {
    ///     const SCHEMA_TYPE_REF: GlobalTypeId = GlobalTypeId::complex(stringify!(MyType), &[T1::SCHEMA_TYPE_REF]);
    /// #   fn get_local_type_data() { todo!() }
    /// }
    /// ```
    const SCHEMA_TYPE_REF: GlobalTypeId;

    /// Returns the local schema for the given type, if the TypeRef is Custom
    fn get_local_type_data() -> Option<TypeData<C, GlobalTypeId>> {
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
    fn add_all_dependencies(aggregator: &mut TypeAggregator<C>) {}
}
