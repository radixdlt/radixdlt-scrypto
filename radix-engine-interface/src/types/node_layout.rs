use crate::api::*;
use crate::internal_prelude::*;
use crate::types::*;
use sbor::rust::prelude::*;

//=========================================================================
// Please update REP-60 after updating types/configs defined in this file!
//=========================================================================

//============================
// System Partitions + Modules
//============================
/// Used only with TRANSACTION_TRACKER Node for boot loading
pub const BOOT_LOADER_PARTITION: PartitionNumber = PartitionNumber(32u8);

const BOOT_LOADER_KERNEL_BOOT_FIELD_KEY: FieldKey = 0u8;
const BOOT_LOADER_SYSTEM_BOOT_FIELD_KEY: FieldKey = 1u8;
const BOOT_LOADER_VM_BOOT_FIELD_KEY: FieldKey = 2u8;
const BOOT_LOADER_TRANSACTION_VALIDATION_CONFIGURATION_FIELD_KEY: FieldKey = 8u8;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum BootLoaderField {
    KernelBoot = BOOT_LOADER_KERNEL_BOOT_FIELD_KEY,
    SystemBoot = BOOT_LOADER_SYSTEM_BOOT_FIELD_KEY,
    VmBoot = BOOT_LOADER_VM_BOOT_FIELD_KEY,
    TransactionValidationConfiguration = BOOT_LOADER_TRANSACTION_VALIDATION_CONFIGURATION_FIELD_KEY,
}

/// Used only with TRANSACTION_TRACKER Node for protocol updating
pub const PROTOCOL_UPDATE_STATUS_PARTITION: PartitionNumber = PartitionNumber(33u8);
const PROTOCOL_UPDATE_STATUS_SUMMARY_FIELD_KEY: FieldKey = 0u8;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum ProtocolUpdateStatusField {
    Summary = PROTOCOL_UPDATE_STATUS_SUMMARY_FIELD_KEY,
}

pub const TYPE_INFO_FIELD_PARTITION: PartitionNumber = PartitionNumber(0u8);

#[repr(u8)]
#[derive(Debug, Copy, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum TypeInfoField {
    TypeInfo,
}

pub const SCHEMAS_PARTITION: PartitionNumber = PartitionNumber(1u8);

pub const METADATA_BASE_PARTITION: PartitionNumber = PartitionNumber(2u8);
pub const METADATA_KV_STORE_PARTITION_OFFSET: PartitionOffset = PartitionOffset(0u8);

pub const ROYALTY_BASE_PARTITION: PartitionNumber = PartitionNumber(3u8);
pub const ROYALTY_FIELDS_PARTITION: PartitionNumber = PartitionNumber(3u8);
pub const ROYALTY_FIELDS_PARTITION_OFFSET: PartitionOffset = PartitionOffset(0u8);
pub const ROYALTY_CONFIG_PARTITION: PartitionNumber = PartitionNumber(4u8);
pub const ROYALTY_CONFIG_PARTITION_OFFSET: PartitionOffset = PartitionOffset(1u8);

#[repr(u8)]
#[derive(Debug, Copy, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum RoyaltyField {
    RoyaltyAccumulator,
}

pub const ROLE_ASSIGNMENT_BASE_PARTITION: PartitionNumber = PartitionNumber(5u8);
pub const ROLE_ASSIGNMENT_FIELDS_PARTITION: PartitionNumber = PartitionNumber(5u8);
pub const ROLE_ASSIGNMENT_FIELDS_PARTITION_OFFSET: PartitionOffset = PartitionOffset(0u8);
pub const ROLE_ASSIGNMENT_ROLE_DEF_PARTITION: PartitionNumber = PartitionNumber(6u8);
pub const ROLE_ASSIGNMENT_ROLE_DEF_PARTITION_OFFSET: PartitionOffset = PartitionOffset(1u8);
pub const ROLE_ASSIGNMENT_MUTABILITY_PARTITION_OFFSET: PartitionOffset = PartitionOffset(2u8);

//=============================
// Blueprint partition - common
//=============================

pub const MAIN_BASE_PARTITION: PartitionNumber = PartitionNumber(64u8);

pub trait BlueprintPartitionOffset: Copy + Into<PartitionOffset> {
    fn as_partition_offset(self) -> PartitionOffset {
        self.into()
    }

    fn as_main_partition(self) -> PartitionNumber {
        self.as_partition(MAIN_BASE_PARTITION)
    }

    fn as_partition(self, base: PartitionNumber) -> PartitionNumber {
        base.at_offset(self.into())
            .expect("Offset larger than allowed value")
    }
}

macro_rules! blueprint_partition_offset {
    (
        $(#[$attributes:meta])*
        $vis:vis enum $t:ident {
            $(
                $(#[$variant_attributes:meta])*
                $variant:ident
            ),*
            $(,)?
        }
    ) => {
        #[repr(u8)]
        #[derive(Debug, Copy, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
        $(#[$attributes])*
        $vis enum $t {
            $(
                $(#[$variant_attributes])*
                $variant,
            )*
        }

        impl BlueprintPartitionOffset for $t {}

        impl $t {
            // Implemented as const, unlike the trait version
            pub const fn as_partition(&self, base: PartitionNumber) -> PartitionNumber {
                match base.at_offset(PartitionOffset(*self as u8)) {
                    // This match works around unwrap/expect on Option not being const
                    Some(x) => x,
                    None => panic!("Offset larger than allowed value")
                }
            }
        }

        impl From<$t> for PartitionOffset {
            fn from(value: $t) -> Self {
                PartitionOffset(value as u8)
            }
        }

        impl TryFrom<PartitionOffset> for $t {
            type Error = ();

            fn try_from(offset: PartitionOffset) -> Result<Self, Self::Error> {
                Self::from_repr(offset.0).ok_or(())
            }
        }

        impl TryFrom<u8> for $t {
            type Error = ();

            fn try_from(offset: u8) -> Result<Self, Self::Error> {
                Self::from_repr(offset).ok_or(())
            }
        }
    };
}

blueprint_partition_offset!(
    pub enum ComponentRoyaltyPartitionOffset {
        Field,
        MethodAmountKeyValue,
    }
);

blueprint_partition_offset!(
    pub enum RoleAssignmentPartitionOffset {
        Field,
        AccessRuleKeyValue,
    }
);

blueprint_partition_offset!(
    pub enum MetadataPartitionOffset {
        EntryKeyValue,
    }
);

//===========
// Blueprints
//===========

#[repr(u8)]
#[derive(Debug, Copy, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum ComponentField {
    State0,
}

blueprint_partition_offset!(
    pub enum FungibleResourceManagerPartitionOffset {
        Field,
    }
);

blueprint_partition_offset!(
    pub enum PackagePartitionOffset {
        Field,
        BlueprintVersionDefinitionKeyValue,
        BlueprintVersionDependenciesKeyValue,
        BlueprintVersionRoyaltyConfigKeyValue,
        BlueprintVersionAuthConfigKeyValue,
        CodeVmTypeKeyValue,
        CodeOriginalCodeKeyValue,
        CodeInstrumentedCodeKeyValue,
    }
);

blueprint_partition_offset!(
    pub enum NonFungibleResourceManagerPartitionOffset {
        Field,
        DataKeyValue,
    }
);

blueprint_partition_offset!(
    pub enum FungibleVaultPartitionOffset {
        Field,
    }
);

blueprint_partition_offset!(
    pub enum NonFungibleVaultPartitionOffset {
        Field,
        NonFungibleIndex,
    }
);

#[repr(u8)]
#[derive(Debug, Copy, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum FungibleBucketField {
    Liquid,
    Locked,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum NonFungibleBucketField {
    Liquid,
    Locked,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum FungibleProofField {
    Moveable,
    ProofRefs,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum NonFungibleProofField {
    Moveable,
    ProofRefs,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum WorktopField {
    Worktop,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum AuthZoneField {
    AuthZone,
}

// CONSENSUS MANAGER PACKAGE

blueprint_partition_offset!(
    pub enum ConsensusManagerPartitionOffset {
        Field,
        RegisteredValidatorByStakeSortedIndex,
    }
);

blueprint_partition_offset!(
    pub enum ValidatorPartitionOffset {
        Field,
    }
);

blueprint_partition_offset!(
    pub enum AccessControllerPartitionOffset {
        Field,
    }
);

blueprint_partition_offset!(
    pub enum AccountPartitionOffset {
        Field,
        ResourceVaultKeyValue,
        ResourcePreferenceKeyValue,
        AuthorizedDepositorKeyValue,
    }
);

blueprint_partition_offset!(
    pub enum AccountLockerPartitionOffset {
        AccountClaimsKeyValue,
    }
);

blueprint_partition_offset!(
    pub enum OneResourcePoolPartitionOffset {
        Field,
    }
);

blueprint_partition_offset!(
    pub enum TwoResourcePoolPartitionOffset {
        Field,
    }
);

blueprint_partition_offset!(
    pub enum MultiResourcePoolPartitionOffset {
        Field,
    }
);

#[repr(u8)]
#[derive(Debug, Copy, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum TransactionTrackerField {
    TransactionTracker,
}

macro_rules! substate_key {
    ($t:ty) => {
        impl From<$t> for SubstateKey {
            fn from(value: $t) -> Self {
                SubstateKey::Field(value as u8)
            }
        }

        impl From<$t> for u8 {
            fn from(value: $t) -> Self {
                value as u8
            }
        }

        impl TryFrom<&SubstateKey> for $t {
            type Error = ();

            fn try_from(key: &SubstateKey) -> Result<Self, Self::Error> {
                match key {
                    SubstateKey::Field(x) => Self::from_repr(*x).ok_or(()),
                    _ => Err(()),
                }
            }
        }

        impl FieldDescriptor for $t {
            fn field_index(&self) -> FieldIndex {
                *self as u8
            }
        }
    };
}

pub trait FieldDescriptor {
    fn field_index(&self) -> FieldIndex;
}

impl FieldDescriptor for FieldIndex {
    fn field_index(&self) -> FieldIndex {
        *self
    }
}

pub trait CollectionDescriptor {
    fn collection_index(&self) -> CollectionIndex;
}

impl CollectionDescriptor for CollectionIndex {
    fn collection_index(&self) -> CollectionIndex {
        *self
    }
}

substate_key!(BootLoaderField);
substate_key!(ProtocolUpdateStatusField);
substate_key!(TypeInfoField);
substate_key!(RoyaltyField);
substate_key!(ComponentField);
substate_key!(TransactionTrackerField);

// Transient
substate_key!(FungibleBucketField);
substate_key!(FungibleProofField);
substate_key!(NonFungibleBucketField);
substate_key!(NonFungibleProofField);
substate_key!(WorktopField);
substate_key!(AuthZoneField);
