use super::*;

/// Designed for basic comparisons between two types, e.g. equality checking.
///
/// ## Example usage
/// ```no_run
/// # use radix_rust::prelude::*;
/// # use sbor::NoCustomSchema;
/// # use sbor::SchemaComparison::*;
/// # type ScryptoCustomSchema = NoCustomSchema;
/// # struct MyType;
/// let base = SingleTypeSchema::from("5b....");
/// let current = SingleTypeSchema::for_type::<MyType>();
/// compare_single_type_schema::<ScryptoCustomSchema>(
///     &SchemaComparisonSettings::require_equality(),
///     &base,
///     &current,
/// ).assert_valid("base", "compared");
/// ```
pub fn compare_single_type_schemas<'s, S: CustomSchema>(
    comparison_settings: &SchemaComparisonSettings,
    base: &'s SingleTypeSchema<S>,
    compared: &'s SingleTypeSchema<S>,
) -> SchemaComparisonResult<'s, S> {
    base.compare_with(compared, comparison_settings)
}

/// Designed for ensuring a type is only altered in ways which ensure
/// backwards compatibility in SBOR serialization (i.e. that old payloads
/// can be deserialized correctly by the latest type).
///
/// This function:
/// * Checks that the type's current schema is equal to the latest version
/// * Checks that each schema is consistent with the previous schema - but
///   can be an extension (e.g. enums can have new variants)
///
/// The version registry is be a map from a version name to some encoding
/// of a `SingleTypeSchema` - including as-is, or hex-encoded sbor-encoded.
/// The version name is only used for a more useful message on error.
///
/// ## Example usage
/// ```no_run
/// # use radix_rust::prelude::*;
/// # use sbor::NoCustomSchema;
/// # use sbor::SchemaComparison::*;
/// # type ScryptoCustomSchema = NoCustomSchema;
/// # struct MyType;
/// assert_type_backwards_compatibility::<ScryptoCustomSchema, MyType>(
///     |v| {
///         v.register_version("babylon_launch", "5b...")
///          .register_version("bottlenose", "5b...")
///     },
/// );
/// ```
/// ## Setup
/// To generate the encoded schema, just run the method with an empty `indexmap!`
/// and the assertion will include the encoded schemas, for copying into the assertion.
///
/// ```
pub fn assert_type_backwards_compatibility<
    S: CustomSchema,
    T: Describe<S::CustomAggregatorTypeKind>,
>(
    versions_builder: impl FnOnce(
        NamedSchemaVersions<S, SingleTypeSchema<S>>,
    ) -> NamedSchemaVersions<S, SingleTypeSchema<S>>,
) {
    assert_type_compatibility::<S, T>(
        &SchemaComparisonSettings::allow_extension(),
        versions_builder,
    )
}

/// Designed for ensuring a type is only altered in ways which ensure
/// backwards compatibility in SBOR serialization (i.e. that old payloads
/// can be deserialized correctly by the latest type).
///
/// This function:
/// * Checks that the type's current schema is equal to the latest version
/// * Checks that each schema is consistent with the previous schema - but
///   can be an extension (e.g. enums can have new variants)
///
/// The version registry is be a map from a version name to some encoding
/// of a `SingleTypeSchema` - including as-is, or hex-encoded sbor-encoded.
/// The version name is only used for a more useful message on error.
///
/// ## Example usage
/// ```no_run
/// # use radix_rust::prelude::*;
/// # use sbor::NoCustomSchema;
/// # use sbor::SchemaComparison::*;
/// # type ScryptoCustomSchema = NoCustomSchema;
/// # struct MyType;
/// assert_type_compatibility::<ScryptoCustomSchema, MyType>(
///     SchemaComparisonSettings::allow_extension(),
///     |v| {
///         v.register_version("babylon_launch", "5b...")
///          .register_version("bottlenose", "5b...")
///     },
/// );
/// ```
/// ## Setup
/// To generate the encoded schema, just run the method with an empty `indexmap!`
/// and the assertion will include the encoded schemas, for copying into the assertion.
///
/// ```
pub fn assert_type_compatibility<S: CustomSchema, T: Describe<S::CustomAggregatorTypeKind>>(
    comparison_settings: &SchemaComparisonSettings,
    versions_builder: impl FnOnce(
        NamedSchemaVersions<S, SingleTypeSchema<S>>,
    ) -> NamedSchemaVersions<S, SingleTypeSchema<S>>,
) {
    let current = generate_single_type_schema::<T, S>();
    assert_schema_compatibility(
        comparison_settings,
        &current,
        &versions_builder(NamedSchemaVersions::new()),
    )
}

pub fn compare_type_collection_schemas<'s, S: CustomSchema>(
    comparison_settings: &SchemaComparisonSettings,
    base: &'s TypeCollectionSchema<S>,
    compared: &'s TypeCollectionSchema<S>,
) -> SchemaComparisonResult<'s, S> {
    base.compare_with(compared, comparison_settings)
}

pub fn assert_type_collection_backwards_compatibility<S: CustomSchema>(
    current: TypeCollectionSchema<S>,
    versions_builder: impl FnOnce(
        NamedSchemaVersions<S, TypeCollectionSchema<S>>,
    ) -> NamedSchemaVersions<S, TypeCollectionSchema<S>>,
) {
    assert_type_collection_compatibility(
        &SchemaComparisonSettings::allow_extension(),
        current,
        versions_builder,
    )
}

pub fn assert_type_collection_compatibility<S: CustomSchema>(
    comparison_settings: &SchemaComparisonSettings,
    current: TypeCollectionSchema<S>,
    versions_builder: impl FnOnce(
        NamedSchemaVersions<S, TypeCollectionSchema<S>>,
    ) -> NamedSchemaVersions<S, TypeCollectionSchema<S>>,
) {
    assert_schema_compatibility(
        comparison_settings,
        &current,
        &versions_builder(NamedSchemaVersions::new()),
    )
}

fn assert_schema_compatibility<S: CustomSchema, C: ComparableSchema<S>>(
    schema_comparison_settings: &SchemaComparisonSettings,
    current: &C,
    named_versions: &NamedSchemaVersions<S, C>,
) {
    let named_versions = named_versions.get_versions();

    // Part 0 - Check that there is at least one named_historic_schema_versions,
    //          if not, output latest encoded.
    let Some((latest_version_name, latest_schema_version)) = named_versions.last() else {
        let mut error = String::new();
        writeln!(
            &mut error,
            "You must provide at least one named schema version."
        )
        .unwrap();
        writeln!(&mut error, "Use a relevant name (for example, the current software version name), and save the current schema as follows:").unwrap();
        writeln!(&mut error, "{}", current.encode_to_hex()).unwrap();
        panic!("{error}");
    };

    // Part 1 - Check that latest is equal to the last historic schema version
    let result =
        latest_schema_version.compare_with(&current, &SchemaComparisonSettings::require_equality());

    if let Some(error_message) = result.error_message(latest_version_name, "current") {
        let mut error = String::new();
        writeln!(&mut error, "The most recent named version ({latest_version_name}) DOES NOT PASS CHECKS, likely because it is not equal to the current version.").unwrap();
        writeln!(&mut error).unwrap();
        write!(&mut error, "{error_message}").unwrap();
        writeln!(&mut error).unwrap();
        writeln!(
            &mut error,
            "You will likely want to do one of the following:"
        )
        .unwrap();
        writeln!(&mut error, "(A) Revert an unintended change to some model.").unwrap();
        writeln!(
            &mut error,
            "(B) Add a new named version to the list, to be supported going forward. You must then generate its schema with `#[sbor_assert(backwards_compatible(..), generate)]`, running the test, and removing `generate`."
        )
        .unwrap();
        writeln!(
            &mut error,
            "(C) If the latest version is under development, and has not been used / release, you can regenerate it with `#[sbor_assert(backwards_compatible(..), regenerate)]`, running the test, and removing `regenerate`."
        )
        .unwrap();
        panic!("{error}");
    }

    // Part 2 - Check that (N, N + 1) schemas respect the comparison settings, pairwise
    for i in 0..named_versions.len() - 1 {
        let (previous_version_name, previous_schema) = named_versions.get_index(i).unwrap();
        let (next_version_name, next_schema) = named_versions.get_index(i + 1).unwrap();

        previous_schema
            .compare_with(next_schema, schema_comparison_settings)
            .assert_valid(previous_version_name, &next_version_name);
    }
}
