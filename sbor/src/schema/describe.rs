use crate::*;

/// The `Describe` trait allows a type to describe how to interpret and validate a corresponding SBOR payload.
///
/// Each unique interpretation/validation of a type should have its own distinct type in the schema.
/// Uniqueness of a type in the schema is defined by its `GlobalTypeId`.
#[allow(unused_variables)]
pub trait Describe<C: CustomTypeKind<GlobalTypeId>> {
    /// The `TYPE_ID` should give a unique identifier for its SBOR schema type.
    /// An SBOR schema type capture details about the SBOR payload, how it should be interpreted, validated and displayed.
    ///
    /// Conceptually, each type should have a unique id based on:
    /// * Its SBOR type, structure and child types
    /// * Any validation that should be applied so that the codec can decode a payload successfully
    /// * How it should be named or its contents be displayed
    /// * Any additional data associated with the type which may be added in future (eg i18n or further validation)
    ///
    /// For example:
    /// * An Array<u32> and Array<u64> are different types because they have different structures
    /// * Two types named "Content" may be in different namepaces, and wrap different kinds of content, so be different types
    /// * The tuple `(T1, T2)` is a different type for each `T1` and `T2` because they have different structures
    /// * Types which are intended to be "transparent" to SBOR such as pointers/smart pointers/etc are equivalent
    ///   to their wrapper type, so should inherit the `TYPE_ID` of the wrapped type.
    ///
    /// Most basic types without additional validation have an associated "Well Known" type, which is intended to save
    /// room in the schema. Any non-well known types are "Novel" and should be generated for each type.
    ///
    /// If needing to generate a novel type id, this can be generated via helper methods on [`GlobalTypeId`]:
    /// ```ignore
    /// impl Describe<C: CustomTypeSchema, T1: Describe<C>> for MyType<T1> {
    ///     const TYPE_ID: GlobalTypeId = GlobalTypeId::complex(stringify!(MyType), &[T1::TYPE_ID]);
    /// #   fn type_data() -> Option<TypeData<C, GlobalTypeId>> { todo!() }
    /// }
    /// ```
    const TYPE_ID: GlobalTypeId;

    /// Returns the local schema for the given type. Should return `Some(_)` if `TYPE_ID` is Novel, else it should return `None`.
    fn type_data() -> Option<TypeData<C, GlobalTypeId>> {
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
    ///   - Create a new mutated `mutated_type_index` for the underlying type plus its mutation
    ///   - Use `mutated_type_index` in the relevant place/s in `get_local_type_data`
    ///   - In `add_all_dependencies` add a line `aggregator.add_child_type(mutated_type_index, mutated_local_type_data)`
    ///
    /// - Step 2: For each (base/unmutated) type dependency `D`:
    ///   - In `add_all_dependencies` add a line `aggregator.add_schema_descendents::<D>()`
    fn add_all_dependencies(aggregator: &mut TypeAggregator<C>) {}
}
