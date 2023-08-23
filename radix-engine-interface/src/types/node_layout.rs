use crate::api::*;
use crate::types::*;
use crate::*;
use sbor::rust::prelude::*;

//=========================================================================
// Please update REP-60 after updating types/configs defined in this file!
//=========================================================================

//============================
// System Partitions + Modules
//============================

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

#[repr(u8)]
#[derive(Debug, Copy, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum RoleAssignmentField {
    OwnerRole,
}

impl TryFrom<u8> for RoleAssignmentField {
    type Error = ();

    fn try_from(offset: u8) -> Result<Self, Self::Error> {
        Self::from_repr(offset).ok_or(())
    }
}

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

//===========
// Blueprints
//===========

#[repr(u8)]
#[derive(Debug, Copy, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum ComponentField {
    State0,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum FungibleResourceManagerField {
    Divisibility,
    TotalSupply,
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

#[repr(u8)]
#[derive(Debug, Copy, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum NonFungibleResourceManagerField {
    IdType,
    MutableFields,
    TotalSupply,
}

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
        ConsensusManager,
        RegisteredValidatorsByStakeIndex,
    }
);

#[repr(u8)]
#[derive(Debug, Copy, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr, EnumCount)]
pub enum ConsensusManagerField {
    Config,
    ConsensusManager,
    ValidatorRewards,
    CurrentValidatorSet,
    CurrentProposalStatistic,
    CurrentTimeRoundedToMinutes,
    CurrentTime,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr, EnumCount)]
pub enum ValidatorField {
    Validator,
    ProtocolUpdateReadinessSignal,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum AccessControllerField {
    AccessController,
}

blueprint_partition_offset!(
    pub enum AccountPartitionOffset {
        Field,
        ResourceVaultKeyValue,
        ResourcePreferenceKeyValue,
        AuthorizedDepositorKeyValue,
    }
);

#[repr(u8)]
#[derive(Debug, Copy, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum OneResourcePoolField {
    OneResourcePool,
}

blueprint_partition_offset!(
    pub enum OneResourcePoolPartitionOffset {
        Field,
    }
);

#[repr(u8)]
#[derive(Debug, Copy, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum TwoResourcePoolField {
    TwoResourcePool,
}

blueprint_partition_offset!(
    pub enum TwoResourcePoolPartitionOffset {
        Field,
    }
);

#[repr(u8)]
#[derive(Debug, Copy, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum MultiResourcePoolField {
    MultiResourcePool,
}

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

substate_key!(TypeInfoField);
substate_key!(RoyaltyField);
substate_key!(RoleAssignmentField);
substate_key!(ComponentField);
substate_key!(FungibleResourceManagerField);
substate_key!(FungibleBucketField);
substate_key!(FungibleProofField);
substate_key!(NonFungibleResourceManagerField);
substate_key!(NonFungibleBucketField);
substate_key!(NonFungibleProofField);
substate_key!(ConsensusManagerField);
substate_key!(ValidatorField);
substate_key!(AccessControllerField);
substate_key!(OneResourcePoolField);
substate_key!(TwoResourcePoolField);
substate_key!(MultiResourcePoolField);
substate_key!(TransactionTrackerField);

// Transient
substate_key!(WorktopField);
substate_key!(AuthZoneField);
