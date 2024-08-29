use super::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SchemaComparisonSettings {
    pub(crate) completeness: SchemaComparisonCompletenessSettings,
    pub(crate) structure: SchemaComparisonStructureSettings,
    pub(crate) metadata: SchemaComparisonMetadataSettings,
    pub(crate) validation: SchemaComparisonValidationSettings,
}

impl SchemaComparisonSettings {
    /// A set of defaults intended to enforce effective equality of the schemas,
    /// but with clear error messages if they diverge
    pub const fn require_equality() -> Self {
        Self {
            completeness: SchemaComparisonCompletenessSettings::enforce_type_roots_cover_schema_disallow_new_root_types(),
            structure: SchemaComparisonStructureSettings::require_identical_structure(),
            metadata: SchemaComparisonMetadataSettings::require_identical_metadata(),
            validation: SchemaComparisonValidationSettings::require_identical_validation(),
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
            metadata: SchemaComparisonMetadataSettings::require_identical_metadata(),
            validation: SchemaComparisonValidationSettings::allow_weakening(),
        }
    }

    pub fn with_completeness(
        mut self,
        builder: impl FnOnce(
            SchemaComparisonCompletenessSettings,
        ) -> SchemaComparisonCompletenessSettings,
    ) -> Self {
        self.completeness = builder(self.completeness);
        self
    }

    pub const fn set_completeness(
        mut self,
        settings: SchemaComparisonCompletenessSettings,
    ) -> Self {
        self.completeness = settings;
        self
    }

    pub fn with_structure(
        mut self,
        builder: impl FnOnce(SchemaComparisonStructureSettings) -> SchemaComparisonStructureSettings,
    ) -> Self {
        self.structure = builder(self.structure);
        self
    }

    pub const fn set_structure(mut self, settings: SchemaComparisonStructureSettings) -> Self {
        self.structure = settings;
        self
    }

    pub fn with_metadata(
        mut self,
        builder: impl FnOnce(SchemaComparisonMetadataSettings) -> SchemaComparisonMetadataSettings,
    ) -> Self {
        self.metadata = builder(self.metadata);
        self
    }

    pub const fn set_metadata(mut self, settings: SchemaComparisonMetadataSettings) -> Self {
        self.metadata = settings;
        self
    }

    /// An easy method for a common setting change
    pub const fn allow_all_name_changes(mut self) -> Self {
        self.metadata = SchemaComparisonMetadataSettings::allow_all_changes();
        self
    }

    pub fn with_validation(
        mut self,
        builder: impl FnOnce(SchemaComparisonValidationSettings) -> SchemaComparisonValidationSettings,
    ) -> Self {
        self.validation = builder(self.validation);
        self
    }

    pub const fn set_validation(mut self, settings: SchemaComparisonValidationSettings) -> Self {
        self.validation = settings;
        self
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct SchemaComparisonCompletenessSettings {
    pub(crate) allow_root_unreachable_types_in_base_schema: bool,
    pub(crate) allow_root_unreachable_types_in_compared_schema: bool,
    /// This is only relevant in the "multiple named roots" mode
    pub(crate) allow_compared_to_have_more_root_types: bool,
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
    pub(crate) allow_new_enum_variants: bool,
    pub(crate) allow_replacing_with_any: bool,
}

impl SchemaComparisonStructureSettings {
    pub const fn require_identical_structure() -> Self {
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
    pub(crate) type_name_changes: NameChangeRule,
    pub(crate) field_name_changes: NameChangeRule,
    pub(crate) variant_name_changes: NameChangeRule,
}

impl SchemaComparisonMetadataSettings {
    pub const fn require_identical_metadata() -> Self {
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

    pub(crate) fn checks_required(&self) -> bool {
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SchemaComparisonValidationSettings {
    pub(crate) allow_validation_weakening: bool,
}

impl SchemaComparisonValidationSettings {
    pub const fn require_identical_validation() -> Self {
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
