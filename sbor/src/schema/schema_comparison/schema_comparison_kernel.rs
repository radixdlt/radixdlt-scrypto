use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ComparisonTypeRoot {
    pub(crate) name: String,
    pub(crate) base_type_id: LocalTypeId,
    pub(crate) compared_type_id: LocalTypeId,
}

pub(crate) struct SchemaComparisonKernel<'s, 'o, S: CustomSchema> {
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
    pub fn new(
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingComparisonRequest {
    base_type_id: LocalTypeId,
    compared_type_id: LocalTypeId,
    ancestor_path: TypeAncestorPath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocatedTypeComparisonResult {
    shallow_status: TypeComparisonStatus,
    example_location: TypeAncestorPath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TypeAncestorPath {
    root_type_identifier: String,
    ancestor_path: Vec<SchemaComparisonPathSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SchemaComparisonPathSegment {
    pub(crate) parent_base_type_id: LocalTypeId,
    pub(crate) parent_compared_type_id: LocalTypeId,
    pub(crate) child_locator: ChildTypeLocator,
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
pub(crate) enum ChildTypeLocator {
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum TypeComparisonStatus {
    Pass,
    Failure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TypeFullPath {
    pub(crate) root_type_identifier: String,
    pub(crate) ancestor_path: Vec<SchemaComparisonPathSegment>,
    pub(crate) leaf_base_type_id: LocalTypeId,
    pub(crate) leaf_compared_type_id: LocalTypeId,
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

struct TypeKindComparisonResult<S: CustomSchema> {
    children_needing_checking: Vec<(ChildTypeLocator, LocalTypeId, LocalTypeId)>,
    errors: Vec<SchemaComparisonErrorDetail<S>>,
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
