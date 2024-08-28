use super::*;

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
    pub(crate) base_schema: &'s Schema<S>,
    pub(crate) compared_schema: &'s Schema<S>,
    pub(crate) errors: Vec<SchemaComparisonError<S>>,
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
            "Schema comparison FAILED between base schema ({}) and compared schema ({}) with {} {}:",
            base_schema_name,
            compared_schema_name,
            self.errors.len(),
            if self.errors.len() == 1 { "error" } else { "errors" },
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
    pub(crate) error_detail: SchemaComparisonErrorDetail<S>,
    pub(crate) example_location: Option<TypeFullPath>,
}

impl<S: CustomSchema> SchemaComparisonError<S> {
    fn write_against_schemas<F: Write>(
        &self,
        f: &mut F,
        base_schema: &Schema<S>,
        compared_schema: &Schema<S>,
    ) -> core::fmt::Result {
        if let Some(location) = &self.example_location {
            let (base_type_kind, base_metadata, _) = base_schema
                .resolve_type_data(location.leaf_base_type_id)
                .expect("Invalid base schema - Could not find data for base type");
            let (compared_type_kind, compared_metadata, _) = compared_schema
                .resolve_type_data(location.leaf_compared_type_id)
                .expect("Invalid compared schema - Could not find data for compared type");

            self.error_detail.write_with_context(
                f,
                base_metadata,
                base_type_kind,
                compared_metadata,
                compared_type_kind,
            )?;
            write!(f, " under {} at path ", location.root_type_identifier)?;
            (location, base_schema, compared_schema, &self.error_detail).write_path(f)?;
        } else {
            write!(f, "{:?}", &self.error_detail)?;
        }
        Ok(())
    }
}

fn combine_optional_names(base_name: Option<&str>, compared_name: Option<&str>) -> Option<String> {
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

fn combine_type_names<S: CustomSchema>(
    base_metadata: &TypeMetadata,
    base_type_kind: &LocalTypeKind<S>,
    compared_metadata: &TypeMetadata,
    compared_type_kind: &LocalTypeKind<S>,
) -> String {
    if let Some(combined_name) =
        combine_optional_names(base_metadata.get_name(), compared_metadata.get_name())
    {
        return combined_name;
    }
    let base_category_name = base_type_kind.category_name();
    let compared_category_name = compared_type_kind.category_name();
    if base_category_name == compared_category_name {
        base_category_name.to_string()
    } else {
        format!("{base_category_name}|{compared_category_name}")
    }
}

impl<'s, 'a, S: CustomSchema> PathAnnotate
    for (
        &'a TypeFullPath,
        &'a Schema<S>,
        &'a Schema<S>,
        &'a SchemaComparisonErrorDetail<S>,
    )
{
    fn iter_ancestor_path(&self) -> Box<dyn Iterator<Item = AnnotatedSborAncestor<'_>> + '_> {
        let (full_path, base_schema, compared_schema, _error_detail) = *self;

        let iterator = full_path.ancestor_path.iter().map(|path_segment| {
            let base_type_id = path_segment.parent_base_type_id;
            let compared_type_id = path_segment.parent_compared_type_id;

            let (base_type_kind, base_metadata, _) = base_schema
                .resolve_type_data(base_type_id)
                .expect("Invalid base schema - Could not find data for base type");
            let (compared_type_kind, compared_metadata, _) = compared_schema
                .resolve_type_data(compared_type_id)
                .expect("Invalid compared schema - Could not find data for compared type");

            let name = Cow::Owned(combine_type_names::<S>(
                base_metadata,
                base_type_kind,
                compared_metadata,
                compared_type_kind,
            ));

            let container = match path_segment.child_locator {
                ChildTypeLocator::Tuple { field_index } => {
                    let field_name = combine_optional_names(
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
                    let variant_name = combine_optional_names(
                        base_variant_metadata.get_name(),
                        compared_variant_metadata.get_name(),
                    )
                    .map(Cow::Owned);
                    let field_name = combine_optional_names(
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
        let (full_path, base_schema, compared_schema, error_detail) = *self;
        let base_type_id = full_path.leaf_base_type_id;
        let compared_type_id = full_path.leaf_compared_type_id;

        let (base_type_kind, base_metadata, _) = base_schema
            .resolve_type_data(base_type_id)
            .expect("Invalid base schema - Could not find data for base type");
        let (compared_type_kind, compared_metadata, _) = compared_schema
            .resolve_type_data(compared_type_id)
            .expect("Invalid compared schema - Could not find data for compared type");

        Some(error_detail.resolve_annotated_leaf(
            base_metadata,
            base_type_kind,
            compared_metadata,
            compared_type_kind,
        ))
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

impl<S: CustomSchema> SchemaComparisonErrorDetail<S> {
    fn resolve_annotated_leaf(
        &self,
        base_metadata: &TypeMetadata,
        base_type_kind: &LocalTypeKind<S>,
        compared_metadata: &TypeMetadata,
        compared_type_kind: &LocalTypeKind<S>,
    ) -> AnnotatedSborPartialLeaf<'_> {
        AnnotatedSborPartialLeaf {
            name: Cow::Owned(combine_type_names::<S>(
                base_metadata,
                base_type_kind,
                compared_metadata,
                compared_type_kind,
            )),
            partial_leaf_locator: self
                .resolve_partial_leaf_locator(base_metadata, compared_metadata),
        }
    }

    fn resolve_partial_leaf_locator(
        &self,
        base_metadata: &TypeMetadata,
        compared_metadata: &TypeMetadata,
    ) -> Option<AnnotatedSborPartialLeafLocator<'static>> {
        match *self {
            SchemaComparisonErrorDetail::TypeKindMismatch { .. } => None,
            SchemaComparisonErrorDetail::TupleFieldCountMismatch { .. } => None,
            SchemaComparisonErrorDetail::EnumSupportedVariantsMismatch { .. } => {
                // This error handles multiple variants, so we can't list them here - instead we handle it in the custom debug print
                None
            }
            SchemaComparisonErrorDetail::EnumVariantFieldCountMismatch {
                variant_discriminator,
                ..
            } => {
                let base_variant = base_metadata
                    .get_enum_variant_data(variant_discriminator)
                    .expect("Invalid base schema - Could not find metadata for enum variant");
                let compared_variant = compared_metadata
                    .get_enum_variant_data(variant_discriminator)
                    .expect("Invalid compared schema - Could not find metadata for enum variant");
                Some(AnnotatedSborPartialLeafLocator::EnumVariant {
                    variant_discriminator: Some(variant_discriminator),
                    variant_name: combine_optional_names(
                        base_variant.get_name(),
                        compared_variant.get_name(),
                    )
                    .map(Cow::Owned),
                    field_index: None,
                    field_name: None,
                })
            }
            SchemaComparisonErrorDetail::TypeNameChangeError(_) => None,
            SchemaComparisonErrorDetail::FieldNameChangeError { field_index, .. } => {
                let base_field_name = base_metadata.get_field_name(field_index);
                let compared_field_name = compared_metadata.get_field_name(field_index);
                Some(AnnotatedSborPartialLeafLocator::Tuple {
                    field_index: Some(field_index),
                    field_name: combine_optional_names(base_field_name, compared_field_name)
                        .map(Cow::Owned),
                })
            }
            SchemaComparisonErrorDetail::EnumVariantNameChangeError {
                variant_discriminator,
                ..
            } => {
                let base_variant = base_metadata
                    .get_enum_variant_data(variant_discriminator)
                    .expect("Invalid base schema - Could not find metadata for enum variant");
                let compared_variant = compared_metadata
                    .get_enum_variant_data(variant_discriminator)
                    .expect("Invalid compared schema - Could not find metadata for enum variant");
                Some(AnnotatedSborPartialLeafLocator::EnumVariant {
                    variant_discriminator: Some(variant_discriminator),
                    variant_name: combine_optional_names(
                        base_variant.get_name(),
                        compared_variant.get_name(),
                    )
                    .map(Cow::Owned),
                    field_index: None,
                    field_name: None,
                })
            }
            SchemaComparisonErrorDetail::EnumVariantFieldNameChangeError {
                variant_discriminator,
                field_index,
                ..
            } => {
                let base_variant = base_metadata
                    .get_enum_variant_data(variant_discriminator)
                    .expect("Invalid base schema - Could not find metadata for enum variant");
                let compared_variant = compared_metadata
                    .get_enum_variant_data(variant_discriminator)
                    .expect("Invalid compared schema - Could not find metadata for enum variant");
                let base_field_name = base_variant.get_field_name(field_index);
                let compared_field_name = compared_variant.get_field_name(field_index);
                Some(AnnotatedSborPartialLeafLocator::EnumVariant {
                    variant_discriminator: Some(variant_discriminator),
                    variant_name: combine_optional_names(
                        base_variant.get_name(),
                        compared_metadata.get_name(),
                    )
                    .map(Cow::Owned),
                    field_index: Some(field_index),
                    field_name: combine_optional_names(base_field_name, compared_field_name)
                        .map(Cow::Owned),
                })
            }
            SchemaComparisonErrorDetail::TypeValidationChangeError { .. } => None,
            SchemaComparisonErrorDetail::NamedRootTypeMissingInComparedSchema { .. } => None,
            SchemaComparisonErrorDetail::DisallowedNewRootTypeInComparedSchema { .. } => None,
            SchemaComparisonErrorDetail::TypeUnreachableFromRootInBaseSchema { .. } => None,
            SchemaComparisonErrorDetail::TypeUnreachableFromRootInComparedSchema { .. } => None,
        }
    }

    fn write_with_context<F: Write>(
        &self,
        f: &mut F,
        base_metadata: &TypeMetadata,
        base_type_kind: &LocalTypeKind<S>,
        compared_metadata: &TypeMetadata,
        compared_type_kind: &LocalTypeKind<S>,
    ) -> core::fmt::Result {
        self.resolve_annotated_leaf(
            base_metadata,
            base_type_kind,
            compared_metadata,
            compared_type_kind,
        )
        .write(f, true)?;
        write!(f, " - ")?;

        match self {
            // Handle any errors where we can add extra detail
            SchemaComparisonErrorDetail::EnumSupportedVariantsMismatch {
                base_variants_missing_in_compared,
                compared_variants_missing_in_base,
            } => {
                write!(
                    f,
                    "EnumSupportedVariantsMismatch {{ base_variants_missing_in_compared: {{"
                )?;
                let mut is_first = true;
                for variant_discriminator in base_variants_missing_in_compared {
                    let variant_data = base_metadata
                        .get_enum_variant_data(*variant_discriminator)
                        .unwrap();
                    if is_first {
                        write!(f, " ")?;
                    } else {
                        write!(f, ", ")?;
                    }
                    write!(
                        f,
                        "{variant_discriminator}|{}",
                        variant_data.get_name().unwrap_or("anon")
                    )?;
                    is_first = false;
                }
                if !is_first {
                    write!(f, " ")?;
                }
                write!(f, "}}, compared_variants_missing_in_base: {{")?;
                let mut is_first = true;
                for variant_discriminator in compared_variants_missing_in_base {
                    let variant_data = compared_metadata
                        .get_enum_variant_data(*variant_discriminator)
                        .unwrap();
                    if is_first {
                        write!(f, " ")?;
                    } else {
                        write!(f, ", ")?;
                    }
                    write!(
                        f,
                        "{variant_discriminator}|{}",
                        variant_data.get_name().unwrap_or("anon")
                    )?;
                    is_first = false;
                }
                if !is_first {
                    write!(f, " ")?;
                }
                write!(f, "}} }}")?;
            }
            // All other errors already have their context added in printing the annotated leaf
            _ => {
                write!(f, "{self:?}")?;
            }
        }

        Ok(())
    }
}
