use crate::internal_prelude::*;

use crate::constants::*;

#[repr(u8)]
#[cfg_attr(
    feature = "radix_engine_fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    FromRepr,
    EnumIter,
    ManifestSbor,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
)]
pub enum ModuleId {
    Main,
    Metadata,
    Royalty,
    RoleAssignment,
}

/// Notes: This is to be deprecated, please use `ModuleId` instead
pub type ObjectModuleId = ModuleId;

impl Describe<ScryptoCustomTypeKind> for ModuleId {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::MODULE_ID_TYPE);

    fn type_data() -> ScryptoTypeData<RustTypeId> {
        well_known_scrypto_custom_types::module_id_type_data()
    }
}

impl From<Option<AttachedModuleId>> for ModuleId {
    fn from(value: Option<AttachedModuleId>) -> Self {
        match value {
            None => ModuleId::Main,
            Some(AttachedModuleId::Metadata) => ModuleId::Metadata,
            Some(AttachedModuleId::Royalty) => ModuleId::Royalty,
            Some(AttachedModuleId::RoleAssignment) => ModuleId::RoleAssignment,
        }
    }
}

impl Into<Option<AttachedModuleId>> for ModuleId {
    fn into(self) -> Option<AttachedModuleId> {
        match self {
            ModuleId::Main => None,
            ModuleId::Metadata => Some(AttachedModuleId::Metadata),
            ModuleId::Royalty => Some(AttachedModuleId::Royalty),
            ModuleId::RoleAssignment => Some(AttachedModuleId::RoleAssignment),
        }
    }
}

impl ModuleId {
    pub fn base_partition_num(&self) -> PartitionNumber {
        match self {
            ModuleId::Metadata => METADATA_BASE_PARTITION,
            ModuleId::Royalty => ROYALTY_BASE_PARTITION,
            ModuleId::RoleAssignment => ROLE_ASSIGNMENT_BASE_PARTITION,
            ModuleId::Main => MAIN_BASE_PARTITION,
        }
    }

    pub fn static_blueprint(&self) -> Option<BlueprintId> {
        match self {
            ModuleId::Metadata => Some(BlueprintId::new(
                &METADATA_MODULE_PACKAGE,
                METADATA_BLUEPRINT,
            )),
            ModuleId::Royalty => Some(BlueprintId::new(
                &ROYALTY_MODULE_PACKAGE,
                COMPONENT_ROYALTY_BLUEPRINT,
            )),
            ModuleId::RoleAssignment => Some(BlueprintId::new(
                &ROLE_ASSIGNMENT_MODULE_PACKAGE,
                ROLE_ASSIGNMENT_BLUEPRINT,
            )),
            ModuleId::Main => None,
        }
    }
}

#[repr(u8)]
#[cfg_attr(
    feature = "radix_engine_fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    FromRepr,
    EnumIter,
    ManifestSbor,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
)]
#[sbor(use_repr_discriminators)]
pub enum AttachedModuleId {
    Metadata = 1,
    Royalty = 2,
    RoleAssignment = 3,
}

impl Describe<ScryptoCustomTypeKind> for AttachedModuleId {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::ATTACHED_MODULE_ID_TYPE);

    fn type_data() -> ScryptoTypeData<RustTypeId> {
        well_known_scrypto_custom_types::attached_module_id_type_data()
    }
}

impl AttachedModuleId {
    pub fn static_blueprint(&self) -> BlueprintId {
        match self {
            AttachedModuleId::Metadata => {
                BlueprintId::new(&METADATA_MODULE_PACKAGE, METADATA_BLUEPRINT)
            }
            AttachedModuleId::Royalty => {
                BlueprintId::new(&ROYALTY_MODULE_PACKAGE, COMPONENT_ROYALTY_BLUEPRINT)
            }
            AttachedModuleId::RoleAssignment => {
                BlueprintId::new(&ROLE_ASSIGNMENT_MODULE_PACKAGE, ROLE_ASSIGNMENT_BLUEPRINT)
            }
        }
    }
}

impl Into<ModuleId> for AttachedModuleId {
    fn into(self) -> ModuleId {
        match self {
            AttachedModuleId::Metadata => ModuleId::Metadata,
            AttachedModuleId::Royalty => ModuleId::Royalty,
            AttachedModuleId::RoleAssignment => ModuleId::RoleAssignment,
        }
    }
}
