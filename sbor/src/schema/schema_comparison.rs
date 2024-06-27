use basic_well_known_types::ANY_TYPE;

use crate::internal_prelude::*;
use crate::schema::*;
use crate::traversal::AnnotatedSborAncestor;
use crate::traversal::AnnotatedSborAncestorContainer;
use crate::traversal::AnnotatedSborPartialLeaf;
use crate::traversal::MapEntryPart;
use crate::traversal::PathAnnotate;
use crate::BASIC_SBOR_V1_MAX_DEPTH;
use radix_rust::rust::fmt::Write;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SchemaComparisonSettings {
    completeness: SchemaComparisonCompletenessSettings,
    structure: SchemaComparisonStructureSettings,
    metadata: SchemaComparisonMetadataSettings,
    validation: SchemaComparisonValidationSettings,
}

impl SchemaComparisonSettings {
    /// A set of defaults intended to enforce effective equality of the schemas,
    /// but with clear error messages if they diverge
    pub const fn require_equality() -> Self {
        Self {
            completeness: SchemaComparisonCompletenessSettings::enforce_type_roots_cover_schema_disallow_new_root_types(),
            structure: SchemaComparisonStructureSettings::equality(),
            metadata: SchemaComparisonMetadataSettings::equality(),
            validation: SchemaComparisonValidationSettings::equality(),
        }
    }

    /// A set of defaults intended to capture a pretty tight definition of structural extension.
    ///
    /// This captures that:
    /// * Payloads which are valid/decodable against the old schema are valid against the new schema
    /// * Programmatic SBOR JSON is unchanged (that is, type/field/variant names are also unchanged)
    ///
    /// Notably:
    /// * Type roots can be added in the compared schema, but we check that the type roots
    ///   provided completely cover both schemas
    /// * Types must be structurally identical on their intersection, except new enum variants can be added
    /// * Type metadata (e.g. names) must be identical on their intersection
    /// * Type validation must be equal or strictly weaker in the new schema
    pub const fn allow_extension() -> Self {
        Self {
            completeness: SchemaComparisonCompletenessSettings::enforce_type_roots_cover_schema_allow_new_root_types(),
            structure: SchemaComparisonStructureSettings::allow_extension(),
            metadata: SchemaComparisonMetadataSettings::equality(),
            validation: SchemaComparisonValidationSettings::allow_weakening(),
        }
    }

    pub const fn completeness_settings(
        mut self,
        checks: SchemaComparisonCompletenessSettings,
    ) -> Self {
        self.completeness = checks;
        self
    }

    pub const fn structure_settings(mut self, checks: SchemaComparisonStructureSettings) -> Self {
        self.structure = checks;
        self
    }

    pub const fn metadata_settings(mut self, checks: SchemaComparisonMetadataSettings) -> Self {
        self.metadata = checks;
        self
    }

    pub const fn validation_settings(mut self, checks: SchemaComparisonValidationSettings) -> Self {
        self.validation = checks;
        self
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct SchemaComparisonCompletenessSettings {
    allow_root_unreachable_types_in_base_schema: bool,
    allow_root_unreachable_types_in_compared_schema: bool,
    /// This is only relevant in the "multiple named roots" mode
    allow_compared_to_have_more_root_types: bool,
}

impl SchemaComparisonCompletenessSettings {
    pub const fn allow_type_roots_not_to_cover_schema() -> Self {
        Self {
            allow_root_unreachable_types_in_base_schema: true,
            allow_root_unreachable_types_in_compared_schema: true,
            allow_compared_to_have_more_root_types: true,
        }
    }

    pub const fn enforce_type_roots_cover_schema_allow_new_root_types() -> Self {
        Self {
            allow_root_unreachable_types_in_base_schema: false,
            allow_root_unreachable_types_in_compared_schema: false,
            allow_compared_to_have_more_root_types: true,
        }
    }

    pub const fn enforce_type_roots_cover_schema_disallow_new_root_types() -> Self {
        Self {
            allow_root_unreachable_types_in_base_schema: false,
            allow_root_unreachable_types_in_compared_schema: false,
            allow_compared_to_have_more_root_types: false,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct SchemaComparisonStructureSettings {
    allow_new_enum_variants: bool,
    allow_replacing_with_any: bool,
}

impl SchemaComparisonStructureSettings {
    pub const fn equality() -> Self {
        Self {
            allow_new_enum_variants: false,
            allow_replacing_with_any: false,
        }
    }

    pub const fn allow_extension() -> Self {
        Self {
            allow_new_enum_variants: true,
            allow_replacing_with_any: true,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SchemaComparisonMetadataSettings {
    type_name_changes: NameChangeRule,
    field_name_changes: NameChangeRule,
    variant_name_changes: NameChangeRule,
}

impl SchemaComparisonMetadataSettings {
    pub const fn equality() -> Self {
        Self {
            type_name_changes: NameChangeRule::equality(),
            field_name_changes: NameChangeRule::equality(),
            variant_name_changes: NameChangeRule::equality(),
        }
    }

    pub const fn allow_adding_names() -> Self {
        Self {
            type_name_changes: NameChangeRule::AllowAddingNames,
            field_name_changes: NameChangeRule::AllowAddingNames,
            variant_name_changes: NameChangeRule::AllowAddingNames,
        }
    }

    pub const fn allow_all_changes() -> Self {
        Self {
            type_name_changes: NameChangeRule::AllowAllChanges,
            field_name_changes: NameChangeRule::AllowAllChanges,
            variant_name_changes: NameChangeRule::AllowAllChanges,
        }
    }

    fn checks_required(&self) -> bool {
        let everything_allowed = self.type_name_changes == NameChangeRule::AllowAllChanges
            && self.field_name_changes == NameChangeRule::AllowAllChanges
            && self.variant_name_changes == NameChangeRule::AllowAllChanges;
        !everything_allowed
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NameChangeRule {
    DisallowAllChanges,
    AllowAddingNames,
    AllowAllChanges,
}

impl NameChangeRule {
    pub const fn equality() -> Self {
        Self::DisallowAllChanges
    }
}

pub enum NameChange<'a> {
    Unchanged,
    NameAdded {
        new_name: &'a str,
    },
    NameRemoved {
        old_name: &'a str,
    },
    NameChanged {
        old_name: &'a str,
        new_name: &'a str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameChangeError {
    change: OwnedNameChange,
    rule_broken: NameChangeRule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnedNameChange {
    Unchanged,
    NameAdded { new_name: String },
    NameRemoved { old_name: String },
    NameChanged { old_name: String, new_name: String },
}

impl<'a> NameChange<'a> {
    pub fn of_changed_option(from: Option<&'a str>, to: Option<&'a str>) -> Self {
        match (from, to) {
            (Some(old_name), Some(new_name)) if old_name == new_name => NameChange::Unchanged,
            (Some(old_name), Some(new_name)) => NameChange::NameChanged { old_name, new_name },
            (Some(old_name), None) => NameChange::NameRemoved { old_name },
            (None, Some(new_name)) => NameChange::NameAdded { new_name },
            (None, None) => NameChange::Unchanged,
        }
    }

    pub fn validate(&self, rule: NameChangeRule) -> Result<(), NameChangeError> {
        let passes = match (self, rule) {
            (NameChange::Unchanged, _) => true,
            (_, NameChangeRule::AllowAllChanges) => true,
            (_, NameChangeRule::DisallowAllChanges) => false,
            (NameChange::NameAdded { .. }, NameChangeRule::AllowAddingNames) => true,
            (NameChange::NameRemoved { .. }, NameChangeRule::AllowAddingNames) => false,
            (NameChange::NameChanged { .. }, NameChangeRule::AllowAddingNames) => false,
        };
        if passes {
            Ok(())
        } else {
            Err(NameChangeError {
                rule_broken: rule,
                change: self.into_owned(),
            })
        }
    }

    fn into_owned(&self) -> OwnedNameChange {
        match *self {
            NameChange::Unchanged => OwnedNameChange::Unchanged,
            NameChange::NameAdded { new_name } => OwnedNameChange::NameAdded {
                new_name: new_name.to_string(),
            },
            NameChange::NameRemoved { old_name } => OwnedNameChange::NameRemoved {
                old_name: old_name.to_string(),
            },
            NameChange::NameChanged { old_name, new_name } => OwnedNameChange::NameChanged {
                old_name: old_name.to_string(),
                new_name: new_name.to_string(),
            },
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SchemaComparisonValidationSettings {
    allow_validation_weakening: bool,
}

impl SchemaComparisonValidationSettings {
    pub const fn equality() -> Self {
        Self {
            allow_validation_weakening: false,
        }
    }

    pub const fn allow_weakening() -> Self {
        Self {
            allow_validation_weakening: true,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ValidationChange {
    Unchanged,
    Strengthened,
    Weakened,
    Incomparable,
}

impl ValidationChange {
    pub fn combine(self, other: ValidationChange) -> Self {
        match (self, other) {
            (ValidationChange::Incomparable, _) => ValidationChange::Incomparable,
            (_, ValidationChange::Incomparable) => ValidationChange::Incomparable,
            (ValidationChange::Unchanged, other) => other,
            (other, ValidationChange::Unchanged) => other,
            (ValidationChange::Strengthened, ValidationChange::Strengthened) => {
                ValidationChange::Strengthened
            }
            (ValidationChange::Strengthened, ValidationChange::Weakened) => {
                ValidationChange::Incomparable
            }
            (ValidationChange::Weakened, ValidationChange::Strengthened) => {
                ValidationChange::Incomparable
            }
            (ValidationChange::Weakened, ValidationChange::Weakened) => ValidationChange::Weakened,
        }
    }
}

#[must_use = "You must read / handle the comparison result"]
pub struct SchemaComparisonResult<'s, S: CustomSchema> {
    base_schema: &'s Schema<S>,
    compared_schema: &'s Schema<S>,
    errors: Vec<SchemaComparisonError<S>>,
}

impl<'s, S: CustomSchema> SchemaComparisonResult<'s, S> {
    pub fn is_valid(&self) -> bool {
        self.errors.len() == 0
    }

    pub fn error_message(
        &self,
        base_schema_name: &str,
        compared_schema_name: &str,
    ) -> Option<String> {
        if self.errors.len() == 0 {
            return None;
        }
        let mut output = String::new();
        writeln!(
            &mut output,
            "Schema comparison FAILED between base schema ({}) and compared schema ({}) with {} errors:",
            base_schema_name,
            compared_schema_name,
            self.errors.len(),
        ).unwrap();
        for error in &self.errors {
            write!(&mut output, "- ").unwrap();
            error
                .write_against_schemas(&mut output, &self.base_schema, &self.compared_schema)
                .unwrap();
            writeln!(&mut output).unwrap();
        }
        Some(output)
    }

    pub fn assert_valid(&self, base_schema_name: &str, compared_schema_name: &str) {
        if let Some(error_message) = self.error_message(base_schema_name, compared_schema_name) {
            panic!("{}", error_message);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaComparisonError<S: CustomSchema> {
    error_detail: SchemaComparisonErrorDetail<S>,
    example_location: Option<TypeFullPath>,
}

impl<S: CustomSchema> SchemaComparisonError<S> {
    fn write_against_schemas<F: Write>(
        &self,
        f: &mut F,
        base_schema: &Schema<S>,
        compared_schema: &Schema<S>,
    ) -> core::fmt::Result {
        if let Some(location) = &self.example_location {
            write!(
                f,
                "{:?} under {} at type path ",
                &self.error_detail, location.root_type_identifier,
            )?;
            (location, base_schema, compared_schema).write_path(f)?;
        } else {
            write!(f, "{:?}", &self.error_detail)?;
        }
        Ok(())
    }
}

fn combine_names(base_name: Option<&str>, compared_name: Option<&str>) -> Option<String> {
    match (base_name, compared_name) {
        (Some(base_name), Some(compared_name)) if base_name == compared_name => {
            Some(base_name.to_string())
        }
        (Some(base_name), Some(compared_name)) => Some(format!("{base_name}|{compared_name}")),
        (Some(base_name), None) => Some(format!("{base_name}|anon")),
        (None, Some(compared_name)) => Some(format!("anon|{compared_name}")),
        (None, None) => None,
    }
}

impl<'s, 'a, S: CustomSchema> PathAnnotate for (&'a TypeFullPath, &'a Schema<S>, &'a Schema<S>) {
    fn iter_ancestor_path(&self) -> Box<dyn Iterator<Item = AnnotatedSborAncestor<'_>> + '_> {
        let (full_path, base_schema, compared_schema) = *self;

        let iterator = full_path.ancestor_path.iter().map(|path_segment| {
            let base_metadata = base_schema
                .resolve_type_metadata(path_segment.parent_base_type_id)
                .expect("Invalid base schema - Could not find metadata for base type");
            let compared_metadata = compared_schema
                .resolve_type_metadata(path_segment.parent_compared_type_id)
                .expect("Invalid compared schema - Could not find metadata for compared type");

            let name = Cow::Owned(
                combine_names(base_metadata.get_name(), compared_metadata.get_name())
                    .unwrap_or_else(|| {
                        combine_names(
                            Some(
                                base_schema
                                    .resolve_type_kind(path_segment.parent_base_type_id)
                                    .unwrap()
                                    .category_name(),
                            ),
                            Some(
                                compared_schema
                                    .resolve_type_kind(path_segment.parent_compared_type_id)
                                    .unwrap()
                                    .category_name(),
                            ),
                        )
                        .unwrap()
                    }),
            );

            let container = match path_segment.child_locator {
                ChildTypeLocator::Tuple { field_index } => {
                    let field_name = combine_names(
                        base_metadata.get_field_name(field_index),
                        compared_metadata.get_field_name(field_index),
                    )
                    .map(Cow::Owned);
                    AnnotatedSborAncestorContainer::Tuple {
                        field_index,
                        field_name,
                    }
                }
                ChildTypeLocator::EnumVariant {
                    discriminator,
                    field_index,
                } => {
                    let base_variant_metadata = base_metadata
                        .get_enum_variant_data(discriminator)
                        .expect("Base schema has variant names");
                    let compared_variant_metadata = compared_metadata
                        .get_enum_variant_data(discriminator)
                        .expect("Compared schema has variant names");
                    let variant_name = combine_names(
                        base_variant_metadata.get_name(),
                        compared_variant_metadata.get_name(),
                    )
                    .map(Cow::Owned);
                    let field_name = combine_names(
                        base_variant_metadata.get_field_name(field_index),
                        compared_variant_metadata.get_field_name(field_index),
                    )
                    .map(Cow::Owned);
                    AnnotatedSborAncestorContainer::EnumVariant {
                        discriminator,
                        variant_name,
                        field_index,
                        field_name,
                    }
                }
                ChildTypeLocator::Array {} => AnnotatedSborAncestorContainer::Array { index: None },
                ChildTypeLocator::Map { entry_part } => AnnotatedSborAncestorContainer::Map {
                    index: None,
                    entry_part,
                },
            };

            AnnotatedSborAncestor { name, container }
        });

        Box::new(iterator)
    }

    fn annotated_leaf(&self) -> Option<AnnotatedSborPartialLeaf<'_>> {
        let (full_path, base_schema, compared_schema) = *self;
        let base_type_id = full_path.leaf_base_type_id;
        let compared_type_id = full_path.leaf_compared_type_id;

        let base_metadata = base_schema
            .resolve_type_metadata(base_type_id)
            .expect("Invalid base schema - Could not find metadata for base type");
        let compared_metadata = compared_schema
            .resolve_type_metadata(compared_type_id)
            .expect("Invalid compared schema - Could not find metadata for compared type");

        let name = Cow::Owned(
            combine_names(base_metadata.get_name(), compared_metadata.get_name()).unwrap_or_else(
                || {
                    combine_names(
                        Some(
                            base_schema
                                .resolve_type_kind(base_type_id)
                                .unwrap()
                                .category_name(),
                        ),
                        Some(
                            compared_schema
                                .resolve_type_kind(compared_type_id)
                                .unwrap()
                                .category_name(),
                        ),
                    )
                    .unwrap()
                },
            ),
        );

        Some(AnnotatedSborPartialLeaf {
            name,
            partial_leaf_locator: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SchemaComparisonPathSegment {
    parent_base_type_id: LocalTypeId,
    parent_compared_type_id: LocalTypeId,
    child_locator: ChildTypeLocator,
}

impl SchemaComparisonPathSegment {
    pub fn of(
        parent_base_type_id: &LocalTypeId,
        parent_compared_type_id: &LocalTypeId,
        child_locator: ChildTypeLocator,
    ) -> Self {
        Self {
            parent_base_type_id: *parent_base_type_id,
            parent_compared_type_id: *parent_compared_type_id,
            child_locator,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaComparisonErrorDetail<S: CustomSchema> {
    // Type kind errors
    TypeKindMismatch {
        base: TypeKindLabel<S::CustomTypeKindLabel>,
        compared: TypeKindLabel<S::CustomTypeKindLabel>,
    },
    TupleFieldCountMismatch {
        base_field_count: usize,
        compared_field_count: usize,
    },
    EnumSupportedVariantsMismatch {
        base_variants_missing_in_compared: IndexSet<u8>,
        compared_variants_missing_in_base: IndexSet<u8>,
    },
    EnumVariantFieldCountMismatch {
        base_field_count: usize,
        compared_field_count: usize,
        variant_discriminator: u8,
    },
    // Type metadata errors
    TypeNameChangeError(NameChangeError),
    FieldNameChangeError {
        error: NameChangeError,
        field_index: usize,
    },
    EnumVariantNameChangeError {
        error: NameChangeError,
        variant_discriminator: u8,
    },
    EnumVariantFieldNameChangeError {
        error: NameChangeError,
        variant_discriminator: u8,
        field_index: usize,
    },
    // Type validation error
    TypeValidationChangeError {
        change: ValidationChange,
        old: TypeValidation<S::CustomTypeValidation>,
        new: TypeValidation<S::CustomTypeValidation>,
    },
    // Completeness errors
    NamedRootTypeMissingInComparedSchema {
        root_type_name: String,
    },
    DisallowedNewRootTypeInComparedSchema {
        root_type_name: String,
    },
    TypeUnreachableFromRootInBaseSchema {
        local_type_index: usize,
        type_name: Option<String>,
    },
    TypeUnreachableFromRootInComparedSchema {
        local_type_index: usize,
        type_name: Option<String>,
    },
}

struct TypeKindComparisonResult<S: CustomSchema> {
    children_needing_checking: Vec<(ChildTypeLocator, LocalTypeId, LocalTypeId)>,
    errors: Vec<SchemaComparisonErrorDetail<S>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ChildTypeLocator {
    Tuple {
        field_index: usize,
    },
    EnumVariant {
        discriminator: u8,
        field_index: usize,
    },
    Array {}, // Unlike values, we don't have an index
    Map {
        entry_part: MapEntryPart,
    }, // Unlike values, we don't have an index
}

impl<S: CustomSchema> TypeKindComparisonResult<S> {
    fn new() -> Self {
        Self {
            children_needing_checking: vec![],
            errors: vec![],
        }
    }

    fn add_error(&mut self, error: SchemaComparisonErrorDetail<S>) {
        self.errors.push(error)
    }

    fn with_mismatch_error(
        mut self,
        base_type_kind: &LocalTypeKind<S>,
        compared_type_kind: &LocalTypeKind<S>,
    ) -> Self {
        self.add_error(SchemaComparisonErrorDetail::TypeKindMismatch {
            base: base_type_kind.label(),
            compared: compared_type_kind.label(),
        });
        self
    }

    fn with_error(mut self, error: SchemaComparisonErrorDetail<S>) -> Self {
        self.add_error(error);
        self
    }

    fn add_child_to_check(
        &mut self,
        child_locator: ChildTypeLocator,
        base_type_id: LocalTypeId,
        compared_type_id: LocalTypeId,
    ) {
        self.children_needing_checking
            .push((child_locator, base_type_id, compared_type_id));
    }
}

struct TypeMetadataComparisonResult<S: CustomSchema> {
    errors: Vec<SchemaComparisonErrorDetail<S>>,
}

impl<S: CustomSchema> TypeMetadataComparisonResult<S> {
    fn new() -> Self {
        Self { errors: vec![] }
    }

    fn add_error(&mut self, error: SchemaComparisonErrorDetail<S>) {
        self.errors.push(error)
    }
}

struct TypeValidationComparisonResult<S: CustomSchema> {
    errors: Vec<SchemaComparisonErrorDetail<S>>,
}

impl<S: CustomSchema> TypeValidationComparisonResult<S> {
    fn new() -> Self {
        Self { errors: vec![] }
    }

    fn add_error(&mut self, error: SchemaComparisonErrorDetail<S>) {
        self.errors.push(error)
    }
}

struct ErrorsAggregator<S: CustomSchema> {
    errors: Vec<SchemaComparisonError<S>>,
}

impl<S: CustomSchema> ErrorsAggregator<S> {
    fn new() -> Self {
        Self { errors: vec![] }
    }

    fn record_error(
        &mut self,
        error_detail: SchemaComparisonErrorDetail<S>,
        example_location: &TypeAncestorPath,
        base_type_id: LocalTypeId,
        compared_type_id: LocalTypeId,
    ) {
        self.errors.push(SchemaComparisonError {
            error_detail,
            example_location: Some(TypeFullPath {
                root_type_identifier: example_location.root_type_identifier.clone(),
                ancestor_path: example_location.ancestor_path.clone(),
                leaf_base_type_id: base_type_id,
                leaf_compared_type_id: compared_type_id,
            }),
        })
    }

    fn record_error_with_unvisited_location(
        &mut self,
        error_detail: SchemaComparisonErrorDetail<S>,
    ) {
        self.errors.push(SchemaComparisonError {
            error_detail,
            example_location: None,
        })
    }
}

struct SchemaComparisonKernel<'s, 'o, S: CustomSchema> {
    base_schema: &'s Schema<S>,
    compared_schema: &'s Schema<S>,
    settings: &'o SchemaComparisonSettings,
    /// A matrix tracking if two types have been compared shallowly
    cached_located_type_comparisons:
        NonIterMap<(LocalTypeId, LocalTypeId), LocatedTypeComparisonResult>,
    /// A list of pending comparisons
    pending_comparison_work_list: Vec<PendingComparisonRequest>,
    /// Used to cheaply capture whether we've seen a local type, for completeness checking
    base_local_types_reachable_from_a_root: NonIterMap<usize, ()>,
    /// Used to cheaply capture whether we've seen a local type, for completeness checking
    compared_local_types_reachable_from_a_root: NonIterMap<usize, ()>,

    /// Tracking all the errors discovered
    errors: ErrorsAggregator<S>,
}

impl<'s, 'o, S: CustomSchema> SchemaComparisonKernel<'s, 'o, S> {
    /// This assumes the schemas provided are valid, and can panic if they're not.
    ///
    /// NOTE: This is NOT designed to be used:
    /// * In situations where the schemas are untrusted.
    ///   The worst case runtime performance here for malicious schemas is O((N + W)^2)
    ///   where N is the number of schema types and W is the number of well known types.
    /// * In situations where performance matters.
    ///   Whilst the expected performance for normal schemas is O(N), this
    ///   isn't designed in a very optimal way (e.g. there are lots of allocations, some
    ///   cloning etc).
    fn new(
        base_schema: &'s Schema<S>,
        compared_schema: &'s Schema<S>,
        settings: &'o SchemaComparisonSettings,
    ) -> Self {
        Self {
            base_schema,
            compared_schema,
            settings,
            cached_located_type_comparisons: Default::default(),
            pending_comparison_work_list: Default::default(),
            base_local_types_reachable_from_a_root: Default::default(),
            compared_local_types_reachable_from_a_root: Default::default(),
            errors: ErrorsAggregator::new(),
        }
    }

    pub fn compare_using_fixed_type_roots(
        mut self,
        type_roots: &[ComparisonTypeRoot],
    ) -> SchemaComparisonResult<'s, S> {
        // NOTE: While providing 0 type_roots is typically an accident, it isn't technically incorrect.
        //       There are some auto-generated cases (e.g. an empty interface) where it may make sense / be easiest
        //       to check an empty list of type roots.
        for ComparisonTypeRoot {
            name,
            base_type_id,
            compared_type_id,
        } in type_roots.iter()
        {
            self.deep_compare_root_types(name, base_type_id, compared_type_id);
            self.mark_root_reachable_base_types(base_type_id);
            self.mark_root_reachable_compared_types(compared_type_id);
        }

        self.check_for_completeness();
        self.into_result()
    }

    pub fn compare_using_named_type_roots(
        mut self,
        base_type_roots: &IndexMap<String, LocalTypeId>,
        compared_type_roots: &IndexMap<String, LocalTypeId>,
    ) -> SchemaComparisonResult<'s, S> {
        // First, let's loop through the base types, and compare them against the corresponding compared types.
        // It is an error for a base named type not to exist in the corresponding compared list.
        for (base_root_type_name, base_type_id) in base_type_roots.iter() {
            if let Some(compared_type_id) = compared_type_roots.get(base_root_type_name) {
                self.deep_compare_root_types(base_root_type_name, base_type_id, compared_type_id);
                self.mark_root_reachable_base_types(base_type_id);
                self.mark_root_reachable_compared_types(compared_type_id);
            } else {
                self.errors.record_error_with_unvisited_location(
                    SchemaComparisonErrorDetail::NamedRootTypeMissingInComparedSchema {
                        root_type_name: base_root_type_name.clone(),
                    },
                );
                self.mark_root_reachable_base_types(base_type_id);
            }
        }

        // We now loop through the compared types not covered in the above loop over base types
        for (compared_root_type_name, compared_type_id) in compared_type_roots.iter() {
            if !base_type_roots.contains_key(compared_root_type_name) {
                if !self
                    .settings
                    .completeness
                    .allow_compared_to_have_more_root_types
                {
                    self.errors.record_error_with_unvisited_location(
                        SchemaComparisonErrorDetail::DisallowedNewRootTypeInComparedSchema {
                            root_type_name: compared_root_type_name.clone(),
                        },
                    );
                }
                self.mark_root_reachable_compared_types(compared_type_id);
            }
        }

        self.check_for_completeness();
        self.into_result()
    }

    fn deep_compare_root_types(
        &mut self,
        root_type_identifier: &str,
        base_type_id: &LocalTypeId,
        compared_type_id: &LocalTypeId,
    ) {
        self.pending_comparison_work_list
            .push(PendingComparisonRequest {
                base_type_id: *base_type_id,
                compared_type_id: *compared_type_id,
                ancestor_path: TypeAncestorPath {
                    root_type_identifier: root_type_identifier.to_string(),
                    ancestor_path: vec![],
                },
            });
        // Run all comparison analysis we can perform.
        // Due to the cache of shallow results over (TypesInBase * TypesInCompared), this must end.
        while let Some(request) = self.pending_comparison_work_list.pop() {
            self.run_single_type_comparison(request);
        }
    }

    fn mark_root_reachable_base_types(&mut self, root_base_type_id: &LocalTypeId) {
        // Due to the cache, we do max O(TypesInBase) work.
        // Note that reachability analysis needs to be performed separately to comparison analysis, because
        // sometimes with comparisons of MyTuple(A) and MyTuple(B1, B2), we still want to perform reachability
        // analysis on A, B1 and B2; but we can't make any sensible comparisons between them.
        let LocalTypeId::SchemaLocalIndex(root_base_local_index) = root_base_type_id else {
            return;
        };
        let mut base_reachability_work_list = vec![*root_base_local_index];
        while let Some(base_type_index) = base_reachability_work_list.pop() {
            match self
                .base_local_types_reachable_from_a_root
                .entry(base_type_index)
            {
                hash_map::Entry::Occupied(_) => continue,
                hash_map::Entry::Vacant(vacant_entry) => vacant_entry.insert(()),
            };
            let type_id = LocalTypeId::SchemaLocalIndex(base_type_index);
            let type_kind = self
                .base_schema
                .resolve_type_kind(type_id)
                .unwrap_or_else(|| {
                    panic!("Invalid base schema - type kind for {type_id:?} not found")
                });
            visit_type_kind_children(type_kind, |_child_locator, child_type_kind| {
                if let LocalTypeId::SchemaLocalIndex(local_index) = child_type_kind {
                    base_reachability_work_list.push(local_index);
                };
            })
        }
    }

    fn mark_root_reachable_compared_types(&mut self, root_compared_type_id: &LocalTypeId) {
        let LocalTypeId::SchemaLocalIndex(root_compared_local_index) = root_compared_type_id else {
            return;
        };
        let mut compared_reachability_work_list = vec![*root_compared_local_index];
        while let Some(compared_local_index) = compared_reachability_work_list.pop() {
            match self
                .compared_local_types_reachable_from_a_root
                .entry(compared_local_index)
            {
                hash_map::Entry::Occupied(_) => continue,
                hash_map::Entry::Vacant(vacant_entry) => vacant_entry.insert(()),
            };
            let type_id = LocalTypeId::SchemaLocalIndex(compared_local_index);
            let type_kind = self
                .compared_schema
                .resolve_type_kind(type_id)
                .unwrap_or_else(|| {
                    panic!("Invalid compared schema - type kind for {type_id:?} not found")
                });
            visit_type_kind_children(type_kind, |_child_locator, child_type_kind| {
                if let LocalTypeId::SchemaLocalIndex(local_index) = child_type_kind {
                    compared_reachability_work_list.push(local_index);
                };
            })
        }
    }

    fn run_single_type_comparison(&mut self, request: PendingComparisonRequest) {
        let PendingComparisonRequest {
            base_type_id,
            compared_type_id,
            ancestor_path: example_location,
        } = request;
        let status_key = (base_type_id, compared_type_id);

        if self
            .cached_located_type_comparisons
            .contains_key(&status_key)
        {
            return;
        }

        let result = self.compare_types_internal(&example_location, base_type_id, compared_type_id);
        for (child_locator, child_base_type_id, child_compared_type_id) in
            result.child_checks_required
        {
            if self
                .cached_located_type_comparisons
                .contains_key(&(child_base_type_id, child_compared_type_id))
            {
                continue;
            }
            let child_example_location = TypeAncestorPath {
                root_type_identifier: example_location.root_type_identifier.clone(),
                ancestor_path: {
                    let mut path = example_location.ancestor_path.clone();
                    path.push(SchemaComparisonPathSegment::of(
                        &base_type_id,
                        &compared_type_id,
                        child_locator,
                    ));
                    path
                },
            };
            self.pending_comparison_work_list
                .push(PendingComparisonRequest {
                    base_type_id: child_base_type_id,
                    compared_type_id: child_compared_type_id,
                    ancestor_path: child_example_location,
                })
        }
        let located_result = LocatedTypeComparisonResult {
            shallow_status: result.shallow_status,
            example_location,
        };
        self.cached_located_type_comparisons
            .insert(status_key, located_result);
    }

    fn compare_types_internal(
        &mut self,
        example_location: &TypeAncestorPath,
        base_type_id: LocalTypeId,
        compared_type_id: LocalTypeId,
    ) -> ShallowTypeComparisonResult {
        // Quick short-circuit when comparing equal well-known types
        match (base_type_id, compared_type_id) {
            (
                LocalTypeId::WellKnown(base_well_known),
                LocalTypeId::WellKnown(compared_well_known),
            ) => {
                if base_well_known == compared_well_known {
                    return ShallowTypeComparisonResult::no_child_checks_required(
                        TypeComparisonStatus::Pass,
                    );
                }
            }
            _ => {}
        }

        // Load type data from each schema
        let (base_type_kind, base_type_metadata, base_type_validation) = self
            .base_schema
            .resolve_type_data(base_type_id)
            .unwrap_or_else(|| {
                panic!("Base schema was not valid - no type data for {base_type_id:?}")
            });
        let (compared_type_kind, compared_type_metadata, compared_type_validation) = self
            .compared_schema
            .resolve_type_data(compared_type_id)
            .unwrap_or_else(|| {
                panic!("Compared schema was not valid - no type data for {compared_type_id:?}")
            });

        // Type Kind Comparison
        let further_checks_required = {
            let TypeKindComparisonResult {
                errors,
                children_needing_checking,
            } = self.compare_type_kind_internal(base_type_kind, compared_type_kind);

            if errors.len() > 0 {
                for error in errors {
                    self.errors.record_error(
                        error,
                        example_location,
                        base_type_id,
                        compared_type_id,
                    );
                }
                // If the type kind comparison fails, then the metadata and validation comparisons aren't helpful information,
                // so we can abort here without further tests.
                return ShallowTypeComparisonResult {
                    shallow_status: TypeComparisonStatus::Failure,
                    child_checks_required: children_needing_checking,
                };
            }

            children_needing_checking
        };

        let mut error_recorded = false;

        // Type Metadata Comparison
        {
            let TypeMetadataComparisonResult { errors } = self.compare_type_metadata_internal(
                base_type_kind,
                base_type_metadata,
                compared_type_metadata,
            );

            for error in errors {
                error_recorded = true;
                self.errors
                    .record_error(error, example_location, base_type_id, compared_type_id);
            }
        }

        // Type Validation Comparison
        {
            let TypeValidationComparisonResult { errors } = self
                .compare_type_validation_internal(base_type_validation, compared_type_validation);

            for error in errors {
                error_recorded = true;
                self.errors
                    .record_error(error, example_location, base_type_id, compared_type_id);
            }
        }

        return ShallowTypeComparisonResult {
            shallow_status: if error_recorded {
                TypeComparisonStatus::Failure
            } else {
                TypeComparisonStatus::Pass
            },
            child_checks_required: further_checks_required,
        };
    }

    fn compare_type_kind_internal(
        &self,
        base_type_kind: &LocalTypeKind<S>,
        compared_type_kind: &LocalTypeKind<S>,
    ) -> TypeKindComparisonResult<S> {
        // The returned children to check should be driven from the base type kind,
        // because these are the children where we have to maintain backwards-compatibility

        let mut result = TypeKindComparisonResult::new();
        let settings = self.settings.structure;
        if *compared_type_kind == TypeKind::Any
            && *base_type_kind != TypeKind::Any
            && settings.allow_replacing_with_any
        {
            // If we allow replacing any type with TypeKind::Any, and the new schema is Any, then the check is valid.
            //
            // That said, we should still check any children against Any:
            // * In case they fail other checks (e.g. ancestor types on the base side required particular type names,
            //   which have now disappeared because the Compared side is Any)
            // * To ensure we pass completeness checks on the base side
            visit_type_kind_children(&base_type_kind, |child_type_locator, child_type_kind| {
                result.add_child_to_check(
                    child_type_locator,
                    child_type_kind,
                    LocalTypeId::WellKnown(ANY_TYPE),
                );
            });
            return result;
        }

        match base_type_kind {
            TypeKind::Any
            | TypeKind::Bool
            | TypeKind::I8
            | TypeKind::I16
            | TypeKind::I32
            | TypeKind::I64
            | TypeKind::I128
            | TypeKind::U8
            | TypeKind::U16
            | TypeKind::U32
            | TypeKind::U64
            | TypeKind::U128
            | TypeKind::String => {
                if compared_type_kind != base_type_kind {
                    return result.with_mismatch_error(base_type_kind, compared_type_kind);
                }
            }
            TypeKind::Array {
                element_type: base_element_type,
            } => {
                let TypeKind::Array {
                    element_type: compared_element_type,
                } = compared_type_kind
                else {
                    return result.with_mismatch_error(base_type_kind, compared_type_kind);
                };
                result.add_child_to_check(
                    ChildTypeLocator::Array {},
                    *base_element_type,
                    *compared_element_type,
                );
            }
            TypeKind::Tuple {
                field_types: base_field_types,
            } => {
                let TypeKind::Tuple {
                    field_types: compared_field_types,
                } = compared_type_kind
                else {
                    return result.with_mismatch_error(base_type_kind, compared_type_kind);
                };
                if base_field_types.len() != compared_field_types.len() {
                    return result.with_error(
                        SchemaComparisonErrorDetail::TupleFieldCountMismatch {
                            base_field_count: base_field_types.len(),
                            compared_field_count: compared_field_types.len(),
                        },
                    );
                }
                let matched_field_types = base_field_types
                    .iter()
                    .cloned()
                    .zip(compared_field_types.iter().cloned())
                    .enumerate();
                for (field_index, (base, compared)) in matched_field_types {
                    result.add_child_to_check(
                        ChildTypeLocator::Tuple { field_index },
                        base,
                        compared,
                    );
                }
            }
            TypeKind::Enum {
                variants: base_variants,
            } => {
                let TypeKind::Enum {
                    variants: compared_variants,
                } = compared_type_kind
                else {
                    return result.with_mismatch_error(base_type_kind, compared_type_kind);
                };

                let base_variants_missing_in_compared: IndexSet<_> = base_variants
                    .keys()
                    .filter(|base_variant_id| !compared_variants.contains_key(*base_variant_id))
                    .cloned()
                    .collect();
                let compared_variants_missing_in_base: IndexSet<_> = compared_variants
                    .keys()
                    .filter(|compared_variant_id| !base_variants.contains_key(*compared_variant_id))
                    .cloned()
                    .collect();

                if base_variants_missing_in_compared.len() > 0
                    || (compared_variants_missing_in_base.len() > 0
                        && !settings.allow_new_enum_variants)
                {
                    result.add_error(SchemaComparisonErrorDetail::EnumSupportedVariantsMismatch {
                        base_variants_missing_in_compared,
                        compared_variants_missing_in_base,
                    });
                }

                for (discriminator, base_field_type_ids) in base_variants.iter() {
                    let Some(compared_field_type_ids) = compared_variants.get(discriminator) else {
                        // We have already output a EnumSupportedVariantsMismatch error above for this.
                        // But let's continue to see if we can match / compare further variants structurally,
                        // to get as many errors as we can.
                        continue;
                    };
                    let discriminator = *discriminator;

                    if base_field_type_ids.len() != compared_field_type_ids.len() {
                        result.add_error(
                            SchemaComparisonErrorDetail::EnumVariantFieldCountMismatch {
                                variant_discriminator: discriminator,
                                base_field_count: base_field_type_ids.len(),
                                compared_field_count: compared_field_type_ids.len(),
                            },
                        );
                    } else {
                        let paired_child_ids = base_field_type_ids
                            .iter()
                            .zip(compared_field_type_ids.iter())
                            .enumerate();
                        for (field_index, (base_child_type_id, compared_child_type_id)) in
                            paired_child_ids
                        {
                            result.add_child_to_check(
                                ChildTypeLocator::EnumVariant {
                                    discriminator,
                                    field_index,
                                },
                                *base_child_type_id,
                                *compared_child_type_id,
                            );
                        }
                    }
                }
            }
            TypeKind::Map {
                key_type: base_key_type,
                value_type: base_value_type,
            } => {
                let TypeKind::Map {
                    key_type: compared_key_type,
                    value_type: compared_value_type,
                } = compared_type_kind
                else {
                    return result.with_mismatch_error(base_type_kind, compared_type_kind);
                };

                result.add_child_to_check(
                    ChildTypeLocator::Map {
                        entry_part: MapEntryPart::Key,
                    },
                    *base_key_type,
                    *compared_key_type,
                );
                result.add_child_to_check(
                    ChildTypeLocator::Map {
                        entry_part: MapEntryPart::Value,
                    },
                    *base_value_type,
                    *compared_value_type,
                );
            }
            // Assume for now that custom types are leaf types.
            // Therefore we can directly run equality on the types, like the simple types.
            TypeKind::Custom(_) => {
                if compared_type_kind != base_type_kind {
                    return result.with_mismatch_error(base_type_kind, compared_type_kind);
                }
            }
        }

        result
    }

    fn compare_type_metadata_internal(
        &self,
        base_type_kind: &LocalTypeKind<S>,
        base_type_metadata: &TypeMetadata,
        compared_type_metadata: &TypeMetadata,
    ) -> TypeMetadataComparisonResult<S> {
        let settings = self.settings.metadata;
        let mut result = TypeMetadataComparisonResult::new();
        if !settings.checks_required() {
            return result;
        }
        if let Err(error) = NameChange::of_changed_option(
            base_type_metadata.type_name.as_deref(),
            compared_type_metadata.type_name.as_deref(),
        )
        .validate(settings.type_name_changes)
        {
            result.add_error(SchemaComparisonErrorDetail::TypeNameChangeError(error));
        }

        // NOTE: For these tests, we assume that the schema is valid - that is, that the type metadata
        // aligns with the underlying type kinds.
        // Also, we have already tested for consistency of the compared type kind against the base type kind.
        // So we can drive field/variant metadata iteration off the base type kind.
        match base_type_kind {
            TypeKind::Tuple { field_types } => {
                for field_index in 0..field_types.len() {
                    if let Err(error) = NameChange::of_changed_option(
                        base_type_metadata.get_field_name(field_index),
                        compared_type_metadata.get_field_name(field_index),
                    )
                    .validate(settings.field_name_changes)
                    {
                        result.add_error(SchemaComparisonErrorDetail::FieldNameChangeError {
                            field_index,
                            error,
                        });
                    }
                }
            }
            TypeKind::Enum { variants } => {
                for (variant_discriminator, base_variant_types) in variants.iter() {
                    let variant_discriminator = *variant_discriminator;
                    let base_variant_metadata = base_type_metadata
                        .get_enum_variant_data(variant_discriminator)
                        .expect("Base schema was not valid - base did not have enum child names for an enum variant");
                    let compared_variant_metadata = compared_type_metadata
                        .get_enum_variant_data(variant_discriminator)
                        .expect("Compared schema was not valid - base and compared agreed on structural equality of an enum, but compared did not have variant metadata for a base variant");

                    if let Err(error) = NameChange::of_changed_option(
                        base_variant_metadata.type_name.as_deref(),
                        compared_variant_metadata.type_name.as_deref(),
                    )
                    .validate(settings.field_name_changes)
                    {
                        result.add_error(SchemaComparisonErrorDetail::EnumVariantNameChangeError {
                            variant_discriminator,
                            error,
                        });
                    }

                    for field_index in 0..base_variant_types.len() {
                        if let Err(error) = NameChange::of_changed_option(
                            base_variant_metadata.get_field_name(field_index),
                            compared_variant_metadata.get_field_name(field_index),
                        )
                        .validate(settings.field_name_changes)
                        {
                            result.add_error(
                                SchemaComparisonErrorDetail::EnumVariantFieldNameChangeError {
                                    variant_discriminator,
                                    field_index,
                                    error,
                                },
                            );
                        }
                    }
                }
            }
            _ => {
                // We can assume the schema is valid, therefore the only valid value is ChildNames::None
                // So validation passes trivially
            }
        }

        result
    }

    fn compare_type_validation_internal(
        &self,
        base_type_validation: &TypeValidation<S::CustomTypeValidation>,
        compared_type_validation: &TypeValidation<S::CustomTypeValidation>,
    ) -> TypeValidationComparisonResult<S> {
        let settings = self.settings.validation;
        let mut result = TypeValidationComparisonResult::new();

        let validation_change = match (base_type_validation, compared_type_validation) {
            (TypeValidation::None, TypeValidation::None) => ValidationChange::Unchanged,
            // Strictly a provided validation might be equivalent to None, for example:
            // (for example NumericValidation { min: None, max: None } or NumericValidation::<I8> { min: 0, max: 255 })
            // but for now assume that it's different
            (_, TypeValidation::None) => ValidationChange::Weakened,
            (TypeValidation::None, _) => ValidationChange::Strengthened,
            // Now test equal validations
            (TypeValidation::I8(base), TypeValidation::I8(compared)) => {
                NumericValidation::compare(base, compared)
            }
            (TypeValidation::I16(base), TypeValidation::I16(compared)) => {
                NumericValidation::compare(base, compared)
            }
            (TypeValidation::I32(base), TypeValidation::I32(compared)) => {
                NumericValidation::compare(base, compared)
            }
            (TypeValidation::I64(base), TypeValidation::I64(compared)) => {
                NumericValidation::compare(base, compared)
            }
            (TypeValidation::I128(base), TypeValidation::I128(compared)) => {
                NumericValidation::compare(base, compared)
            }
            (TypeValidation::U8(base), TypeValidation::U8(compared)) => {
                NumericValidation::compare(base, compared)
            }
            (TypeValidation::U16(base), TypeValidation::U16(compared)) => {
                NumericValidation::compare(base, compared)
            }
            (TypeValidation::U32(base), TypeValidation::U32(compared)) => {
                NumericValidation::compare(base, compared)
            }
            (TypeValidation::U64(base), TypeValidation::U64(compared)) => {
                NumericValidation::compare(base, compared)
            }
            (TypeValidation::U128(base), TypeValidation::U128(compared)) => {
                NumericValidation::compare(base, compared)
            }
            (TypeValidation::String(base), TypeValidation::String(compared)) => {
                LengthValidation::compare(base, compared)
            }
            (TypeValidation::Array(base), TypeValidation::Array(compared)) => {
                LengthValidation::compare(base, compared)
            }
            (TypeValidation::Map(base), TypeValidation::Map(compared)) => {
                LengthValidation::compare(base, compared)
            }
            (TypeValidation::Custom(base), TypeValidation::Custom(compared)) => {
                <<S as CustomSchema>::CustomTypeValidation as CustomTypeValidation>::compare(
                    base, compared,
                )
            }
            // Otherwise assume they are incomparable
            _ => ValidationChange::Incomparable,
        };
        let is_valid = match validation_change {
            ValidationChange::Unchanged => true,
            ValidationChange::Strengthened => false,
            ValidationChange::Weakened => settings.allow_validation_weakening,
            ValidationChange::Incomparable => false,
        };
        if !is_valid {
            result.add_error(SchemaComparisonErrorDetail::TypeValidationChangeError {
                change: validation_change,
                old: base_type_validation.clone(),
                new: compared_type_validation.clone(),
            })
        }
        result
    }

    fn check_for_completeness(&mut self) {
        if !self
            .settings
            .completeness
            .allow_root_unreachable_types_in_base_schema
        {
            if self.base_local_types_reachable_from_a_root.len()
                < self.base_schema.type_metadata.len()
            {
                for (local_type_index, metadata) in
                    self.base_schema.type_metadata.iter().enumerate()
                {
                    if !self
                        .base_local_types_reachable_from_a_root
                        .contains_key(&local_type_index)
                    {
                        let type_name = metadata.type_name.as_ref().map(|n| n.clone().into_owned());
                        self.errors.record_error_with_unvisited_location(
                            SchemaComparisonErrorDetail::TypeUnreachableFromRootInBaseSchema {
                                local_type_index,
                                type_name,
                            },
                        )
                    }
                }
            }
        }
        if !self
            .settings
            .completeness
            .allow_root_unreachable_types_in_compared_schema
        {
            if self.compared_local_types_reachable_from_a_root.len()
                < self.compared_schema.type_metadata.len()
            {
                for (local_type_index, metadata) in
                    self.compared_schema.type_metadata.iter().enumerate()
                {
                    if !self
                        .compared_local_types_reachable_from_a_root
                        .contains_key(&local_type_index)
                    {
                        let type_name = metadata.type_name.as_ref().map(|n| n.clone().into_owned());
                        self.errors.record_error_with_unvisited_location(
                            SchemaComparisonErrorDetail::TypeUnreachableFromRootInComparedSchema {
                                local_type_index,
                                type_name,
                            },
                        )
                    }
                }
            }
        }
    }

    fn into_result(self) -> SchemaComparisonResult<'s, S> {
        SchemaComparisonResult {
            base_schema: self.base_schema,
            compared_schema: self.compared_schema,
            errors: self.errors.errors,
        }
    }
}

fn visit_type_kind_children<T: CustomTypeKind<LocalTypeId>>(
    type_kind: &TypeKind<T, LocalTypeId>,
    mut visitor: impl FnMut(ChildTypeLocator, LocalTypeId),
) {
    return match type_kind {
        TypeKind::Any
        | TypeKind::Bool
        | TypeKind::I8
        | TypeKind::I16
        | TypeKind::I32
        | TypeKind::I64
        | TypeKind::I128
        | TypeKind::U8
        | TypeKind::U16
        | TypeKind::U32
        | TypeKind::U64
        | TypeKind::U128
        | TypeKind::String => {}
        TypeKind::Array { element_type } => {
            visitor(ChildTypeLocator::Array {}, *element_type);
        }
        TypeKind::Tuple { field_types } => {
            for (field_index, field_type) in field_types.iter().enumerate() {
                visitor(ChildTypeLocator::Tuple { field_index }, *field_type)
            }
        }
        TypeKind::Enum { variants } => {
            for (discriminator, field_types) in variants {
                for (field_index, field_type) in field_types.iter().enumerate() {
                    visitor(
                        ChildTypeLocator::EnumVariant {
                            discriminator: *discriminator,
                            field_index,
                        },
                        *field_type,
                    )
                }
            }
        }
        TypeKind::Map {
            key_type,
            value_type,
        } => {
            visitor(
                ChildTypeLocator::Map {
                    entry_part: MapEntryPart::Key,
                },
                *key_type,
            );
            visitor(
                ChildTypeLocator::Map {
                    entry_part: MapEntryPart::Value,
                },
                *value_type,
            );
        }
        // At present, assume that custom types are leaf types.
        TypeKind::Custom(_) => {}
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingComparisonRequest {
    base_type_id: LocalTypeId,
    compared_type_id: LocalTypeId,
    ancestor_path: TypeAncestorPath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TypeAncestorPath {
    root_type_identifier: String,
    ancestor_path: Vec<SchemaComparisonPathSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TypeFullPath {
    root_type_identifier: String,
    ancestor_path: Vec<SchemaComparisonPathSegment>,
    leaf_base_type_id: LocalTypeId,
    leaf_compared_type_id: LocalTypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocatedTypeComparisonResult {
    shallow_status: TypeComparisonStatus,
    example_location: TypeAncestorPath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShallowTypeComparisonResult {
    shallow_status: TypeComparisonStatus,
    child_checks_required: Vec<(ChildTypeLocator, LocalTypeId, LocalTypeId)>,
}

impl ShallowTypeComparisonResult {
    pub fn no_child_checks_required(status: TypeComparisonStatus) -> Self {
        Self {
            shallow_status: status,
            child_checks_required: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TypeComparisonStatus {
    Pass,
    Failure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparisonTypeRoot {
    name: String,
    base_type_id: LocalTypeId,
    compared_type_id: LocalTypeId,
}

pub struct NamedSchemaVersions<S: CustomSchema, C: ComparisonSchema<S>> {
    ordered_versions: IndexMap<String, C>,
    custom_schema: PhantomData<S>,
}

impl<S: CustomSchema, C: ComparisonSchema<S>> NamedSchemaVersions<S, C> {
    pub fn new() -> Self {
        Self {
            ordered_versions: Default::default(),
            custom_schema: Default::default(),
        }
    }

    pub fn register_version(
        mut self,
        name: impl AsRef<str>,
        version: impl IntoSchema<C, S>,
    ) -> Self {
        self.ordered_versions
            .insert(name.as_ref().to_string(), version.into_schema());
        self
    }

    pub fn get_versions(&self) -> &IndexMap<String, C> {
        &self.ordered_versions
    }
}

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
/// let current = SingleTypeSchema::of_type::<MyType>();
/// assert_single_type_comparison::<ScryptoCustomSchema>(
///     SchemaComparisonSettings::equality(),
///     &base,
///     &current,
/// );
/// ```
pub fn assert_single_type_comparison<S: CustomSchema>(
    comparison_settings: &SchemaComparisonSettings,
    base: &SingleTypeSchema<S>,
    compared: &SingleTypeSchema<S>,
) {
    base.compare_with(compared, comparison_settings)
        .assert_valid("base", "compared");
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

pub fn assert_type_collection_comparison<S: CustomSchema>(
    comparison_settings: &SchemaComparisonSettings,
    base: &NamedTypesSchema<S>,
    compared: &NamedTypesSchema<S>,
) {
    base.compare_with(compared, comparison_settings)
        .assert_valid("base", "compared");
}

pub fn assert_type_collection_backwards_compatibility<S: CustomSchema>(
    current: NamedTypesSchema<S>,
    versions_builder: impl FnOnce(
        NamedSchemaVersions<S, NamedTypesSchema<S>>,
    ) -> NamedSchemaVersions<S, NamedTypesSchema<S>>,
) {
    assert_type_collection_compatibility(
        &SchemaComparisonSettings::allow_extension(),
        current,
        versions_builder,
    )
}

pub fn assert_type_collection_compatibility<S: CustomSchema>(
    comparison_settings: &SchemaComparisonSettings,
    current: NamedTypesSchema<S>,
    versions_builder: impl FnOnce(
        NamedSchemaVersions<S, NamedTypesSchema<S>>,
    ) -> NamedSchemaVersions<S, NamedTypesSchema<S>>,
) {
    assert_schema_compatibility(
        comparison_settings,
        &current,
        &versions_builder(NamedSchemaVersions::new()),
    )
}

fn assert_schema_compatibility<S: CustomSchema, C: ComparisonSchema<S>>(
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
            "You must provide at least one named versioned schema to use this method."
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
        writeln!(&mut error, "You will either want to:").unwrap();
        writeln!(
            &mut error,
            "(A) Add a new named version to the list, to be supported going forward."
        )
        .unwrap();
        writeln!(
            &mut error,
            "(B) Replace the latest version. ONLY do this if the version has not yet been in use."
        )
        .unwrap();
        writeln!(&mut error).unwrap();
        writeln!(&mut error, "The latest version is:").unwrap();
        writeln!(&mut error, "{}", current.encode_to_hex()).unwrap();
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

/// A serializable record of the schema of a single type.
/// Intended for historical backwards compatibility checking of a single type.
#[derive(Debug, Clone, Sbor)]
#[sbor(child_types = "S::CustomLocalTypeKind, S::CustomTypeValidation")]
pub struct SingleTypeSchema<S: CustomSchema> {
    pub schema: VersionedSchema<S>,
    pub type_id: LocalTypeId,
}

impl<S: CustomSchema> SingleTypeSchema<S> {
    pub fn new(schema: VersionedSchema<S>, type_id: LocalTypeId) -> Self {
        Self { schema, type_id }
    }

    pub fn from<T: IntoSchema<Self, S>>(from: &T) -> Self {
        from.into_schema()
    }

    pub fn for_type<T: Describe<S::CustomAggregatorTypeKind> + ?Sized>() -> Self {
        generate_single_type_schema::<T, S>()
    }
}

impl<S: CustomSchema> ComparisonSchema<S> for SingleTypeSchema<S> {
    fn compare_with<'s>(
        &'s self,
        compared: &'s Self,
        settings: &SchemaComparisonSettings,
    ) -> SchemaComparisonResult<'s, S> {
        SchemaComparisonKernel::new(
            &self.schema.as_unique_version(),
            &compared.schema.as_unique_version(),
            settings,
        )
        .compare_using_fixed_type_roots(&[ComparisonTypeRoot {
            name: "root".to_string(),
            base_type_id: self.type_id,
            compared_type_id: compared.type_id,
        }])
    }
}

impl<S: CustomSchema> IntoSchema<Self, S> for SingleTypeSchema<S> {
    fn into_schema(&self) -> Self {
        self.clone()
    }
}

/// A serializable record of the schema of a set of named types.
/// Intended for historical backwards compatibility of a collection
/// of types in a single schema.
///
/// For example, traits, or blueprint interfaces.
#[derive(Debug, Clone, Sbor)]
#[sbor(child_types = "S::CustomLocalTypeKind, S::CustomTypeValidation")]
pub struct NamedTypesSchema<S: CustomSchema> {
    pub schema: VersionedSchema<S>,
    pub type_ids: IndexMap<String, LocalTypeId>,
}

impl<S: CustomSchema> NamedTypesSchema<S> {
    pub fn new(schema: VersionedSchema<S>, type_ids: IndexMap<String, LocalTypeId>) -> Self {
        Self { schema, type_ids }
    }

    pub fn from<T: IntoSchema<Self, S>>(from: &T) -> Self {
        from.into_schema()
    }

    pub fn from_aggregator(aggregator: TypeAggregator<S::CustomAggregatorTypeKind>) -> Self {
        aggregator.generate_named_types_schema::<S>()
    }
}

impl<S: CustomSchema> ComparisonSchema<S> for NamedTypesSchema<S> {
    fn compare_with<'s>(
        &'s self,
        compared: &'s Self,
        settings: &SchemaComparisonSettings,
    ) -> SchemaComparisonResult<'s, S> {
        SchemaComparisonKernel::new(
            &self.schema.as_unique_version(),
            &compared.schema.as_unique_version(),
            settings,
        )
        .compare_using_named_type_roots(&self.type_ids, &compared.type_ids)
    }
}

impl<S: CustomSchema> IntoSchema<Self, S> for NamedTypesSchema<S> {
    fn into_schema(&self) -> Self {
        self.clone()
    }
}

// Marker trait
pub trait ComparisonSchema<S: CustomSchema>: Clone + VecSbor<S::DefaultCustomExtension> {
    fn encode_to_bytes(&self) -> Vec<u8> {
        vec_encode::<S::DefaultCustomExtension, Self>(self, BASIC_SBOR_V1_MAX_DEPTH).unwrap()
    }

    fn encode_to_hex(&self) -> String {
        hex::encode(&self.encode_to_bytes())
    }

    fn decode_from_bytes(bytes: &[u8]) -> Self {
        vec_decode::<S::DefaultCustomExtension, Self>(bytes, BASIC_SBOR_V1_MAX_DEPTH).unwrap()
    }

    fn decode_from_hex(hex: &str) -> Self {
        Self::decode_from_bytes(&hex::decode(hex).unwrap())
    }

    fn compare_with<'s>(
        &'s self,
        compared: &'s Self,
        settings: &SchemaComparisonSettings,
    ) -> SchemaComparisonResult<'s, S>;
}

pub trait IntoSchema<C: ComparisonSchema<S>, S: CustomSchema> {
    fn into_schema(&self) -> C;
}

impl<'a, C: ComparisonSchema<S>, S: CustomSchema, T: IntoSchema<C, S> + ?Sized> IntoSchema<C, S>
    for &'a T
{
    fn into_schema(&self) -> C {
        <T as IntoSchema<C, S>>::into_schema(*self)
    }
}

impl<C: ComparisonSchema<S>, S: CustomSchema> IntoSchema<C, S> for [u8] {
    fn into_schema(&self) -> C {
        C::decode_from_bytes(self)
    }
}

impl<C: ComparisonSchema<S>, S: CustomSchema> IntoSchema<C, S> for Vec<u8> {
    fn into_schema(&self) -> C {
        C::decode_from_bytes(self)
    }
}

impl<C: ComparisonSchema<S>, S: CustomSchema> IntoSchema<C, S> for String {
    fn into_schema(&self) -> C {
        C::decode_from_hex(self)
    }
}

impl<C: ComparisonSchema<S>, S: CustomSchema> IntoSchema<C, S> for str {
    fn into_schema(&self) -> C {
        C::decode_from_hex(self)
    }
}

// NOTE: Types are in sbor-tests/tests/schema_comparison.rs
