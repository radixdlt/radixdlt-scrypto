use super::*;

/// Designed for basic comparisons between two types, e.g. equality checking.
///
/// ## Example usage
/// ```no_run
/// # use sbor::prelude::*;
/// # use sbor::schema::*;
/// # use sbor::NoCustomSchema;
/// # type ScryptoCustomSchema = NoCustomSchema;
/// # #[derive(BasicSbor)]
/// # struct MyType;
/// let base = SingleTypeSchema::from("5c....");
/// let current = SingleTypeSchema::for_type::<MyType>();
/// compare_single_type_schemas::<ScryptoCustomSchema>(
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

/// Designed for basic comparisons between two type collection schemas, e.g. equality checking.
///
/// ## Example usage
/// ```no_run
/// # use sbor::prelude::*;
/// # use sbor::schema::*;
/// # use sbor::NoCustomSchema;
/// # type ScryptoCustomSchema = NoCustomSchema;
/// # let type_aggregator_with_named_types = TypeAggregator::<<ScryptoCustomSchema as CustomSchema>::CustomAggregatorTypeKind>::new();
/// let base = TypeCollectionSchema::from("5c....");
/// let current = TypeCollectionSchema::from_aggregator(type_aggregator_with_named_types);
/// compare_type_collection_schemas::<ScryptoCustomSchema>(
///     &SchemaComparisonSettings::require_equality(),
///     &base,
///     &current,
/// ).assert_valid("base", "compared");
/// ```
pub fn compare_type_collection_schemas<'s, S: CustomSchema>(
    comparison_settings: &SchemaComparisonSettings,
    base: &'s TypeCollectionSchema<S>,
    compared: &'s TypeCollectionSchema<S>,
) -> SchemaComparisonResult<'s, S> {
    base.compare_with(compared, comparison_settings)
}

pub struct TypeCompatibilityParameters<S: CustomSchema, C: ComparableSchema<S>> {
    pub comparison_between_versions: SchemaComparisonSettings,
    pub comparison_between_current_and_latest: SchemaComparisonSettings,
    pub named_versions: NamedSchemaVersions<S, C>,
}

pub type SingleTypeSchemaCompatibilityParameters<S> =
    TypeCompatibilityParameters<S, SingleTypeSchema<S>>;
pub type TypeCollectionSchemaCompatibilityParameters<S> =
    TypeCompatibilityParameters<S, TypeCollectionSchema<S>>;

impl<S: CustomSchema, C: ComparableSchema<S>> TypeCompatibilityParameters<S, C> {
    pub fn new() -> Self {
        Self {
            comparison_between_versions: SchemaComparisonSettings::allow_extension(),
            comparison_between_current_and_latest: SchemaComparisonSettings::require_equality(),
            named_versions: NamedSchemaVersions::new(),
        }
    }

    pub fn with_comparison_between_versions(
        mut self,
        builder: impl FnOnce(SchemaComparisonSettings) -> SchemaComparisonSettings,
    ) -> Self {
        self.comparison_between_versions = builder(self.comparison_between_versions);
        self
    }

    pub fn with_comparison_between_current_and_latest(
        mut self,
        builder: impl FnOnce(SchemaComparisonSettings) -> SchemaComparisonSettings,
    ) -> Self {
        self.comparison_between_current_and_latest =
            builder(self.comparison_between_current_and_latest);
        self
    }

    pub fn replace_versions_with(
        mut self,
        named_schema_versions: NamedSchemaVersions<S, C>,
    ) -> Self {
        self.named_versions = named_schema_versions;
        self
    }

    pub fn register_version(
        mut self,
        name: impl AsRef<str>,
        version: impl IntoComparableSchema<C, S>,
    ) -> Self {
        self.named_versions = self.named_versions.register_version(name, version);
        self
    }
}

/// Designed for ensuring a type is only altered in ways which ensure
/// backwards compatibility in SBOR serialization (i.e. that old payloads
/// can be deserialized correctly by the latest type).
///
/// By default, this function:
/// * Checks that the type's current schema is equal to the latest version
/// * Checks that each schema is consistent with the previous schema - but
///   can be an extension (e.g. enums can have new variants)
///
/// The comparison settings used for the current and historic checks can be
/// changed by the builder, to, for example, ignore naming.
///
/// The version registry is a map from a version name to some encoding
/// of a `SingleTypeSchema` - including as-is, or hex-encoded sbor-encoded.
/// The version name is only used for a more useful message on error.
///
/// ## Example usage
///
/// ```no_run
/// # use sbor::prelude::*;
/// # use sbor::schema::*;
/// # use sbor::NoCustomSchema;
/// # type ScryptoCustomSchema = NoCustomSchema;
/// # #[derive(BasicSbor)]
/// # struct MyType;
/// assert_type_backwards_compatibility::<ScryptoCustomSchema, MyType>(
///     |v| {
///         v.register_version("babylon_launch", "5c...")
///          .register_version("bottlenose", "5c...")
///     },
/// );
/// ```
///
/// ## Setup
/// To generate the encoded schema, just run the method with an empty `indexmap!`
/// and the assertion will include the encoded schemas, for copying into the assertion.
pub fn assert_type_backwards_compatibility<
    S: CustomSchema,
    T: Describe<S::CustomAggregatorTypeKind>,
>(
    parameters_builder: impl FnOnce(
        TypeCompatibilityParameters<S, SingleTypeSchema<S>>,
    ) -> TypeCompatibilityParameters<S, SingleTypeSchema<S>>,
) {
    let current = generate_single_type_schema::<T, S>();
    assert_schema_compatibility(
        &current,
        &parameters_builder(TypeCompatibilityParameters::new()),
    )
}

/// Designed for ensuring a type collection is only altered in ways which ensure
/// backwards compatibility in SBOR serialization (i.e. that old payloads of
/// named types can be deserialized correctly by the latest schema).
///
/// By default, this function:
/// * Checks that the current schema is equal to the latest configured version
/// * Checks that each schema is consistent with the previous schema - but
///   can be an extension (e.g. enums can have new variants, and new named
///   types can be added)
///
/// The comparison settings used for the current and historic checks can be
/// changed by the builder, to, for example, ignore naming.
///
/// The version registry is a map from a version name to some encoding
/// of a `TypeCollectionSchema<S>` - including as-is, or hex-encoded sbor-encoded.
/// The version name is only used for a more useful message on error.
///
/// ## Example usage
///
/// ```no_run
/// # use radix_rust::prelude::*;
/// # use sbor::NoCustomSchema;
/// # use sbor::schema::*;
/// # type ScryptoCustomSchema = NoCustomSchema;
/// # let type_aggregator_with_named_types = TypeAggregator::<<ScryptoCustomSchema as CustomSchema>::CustomAggregatorTypeKind>::new();
/// let current = TypeCollectionSchema::from_aggregator(type_aggregator_with_named_types);
/// assert_type_collection_backwards_compatibility::<ScryptoCustomSchema>(
///     &current,
///     |v| {
///         v.register_version("babylon_launch", "5c...")
///          .register_version("bottlenose", "5c...")
///          .with_comparison_between_current_and_latest(|settings| settings.allow_all_name_changes())
///          .with_comparison_between_versions(|settings| settings.allow_all_name_changes())
///     },
/// );
/// ```
///
/// ## Setup
/// To generate the encoded schema, just run the method with an empty `indexmap!`
/// and the assertion will include the encoded schemas, for copying into the assertion.
pub fn assert_type_collection_backwards_compatibility<S: CustomSchema>(
    current: &TypeCollectionSchema<S>,
    parameters_builder: impl FnOnce(
        TypeCompatibilityParameters<S, TypeCollectionSchema<S>>,
    ) -> TypeCompatibilityParameters<S, TypeCollectionSchema<S>>,
) {
    assert_schema_compatibility(
        current,
        &parameters_builder(TypeCompatibilityParameters::new()),
    )
}

fn assert_schema_compatibility<S: CustomSchema, C: ComparableSchema<S>>(
    current: &C,
    parameters: &TypeCompatibilityParameters<S, C>,
) {
    let named_versions = parameters.named_versions.get_versions();

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
    let result = latest_schema_version
        .compare_with(&current, &parameters.comparison_between_current_and_latest);

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
            .compare_with(next_schema, &parameters.comparison_between_versions)
            .assert_valid(previous_version_name, &next_version_name);
    }
}
