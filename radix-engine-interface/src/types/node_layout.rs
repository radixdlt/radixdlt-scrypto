use crate::types::*;
use crate::*;
use sbor::rust::prelude::*;

//=========================================================================
// Please update REP-60 after updating types/configs defined in this file!
//=========================================================================

pub const TYPE_INFO_FIELD_PARTITION: PartitionNumber = PartitionNumber(0u8);
pub const METADATA_KV_STORE_PARTITION: PartitionNumber = PartitionNumber(1u8);

pub const ROYALTY_BASE_PARTITION: PartitionNumber = PartitionNumber(2u8);
pub const ROYALTY_FIELDS_PARTITION_OFFSET: PartitionOffset = PartitionOffset(0u8);
pub const ROYALTY_CONFIG_PARTITION_OFFSET: PartitionOffset = PartitionOffset(1u8);

pub const ACCESS_RULES_BASE_PARTITION: PartitionNumber = PartitionNumber(4u8);
pub const ACCESS_RULES_MUTABILITY_PARTITION_OFFSET: PartitionOffset = PartitionOffset(1u8);

pub const ACCESS_RULES_MUTABILITY_PARTITION: PartitionNumber = PartitionNumber(5u8);

pub const MAIN_BASE_PARTITION: PartitionNumber = PartitionNumber(64u8);

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum TypeInfoField {
    TypeInfo,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum RoyaltyField {
    RoyaltyAccumulator,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum ComponentField {
    State0,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum PackageField {
    Code,
    Royalty,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum FungibleResourceManagerField {
    Divisibility,
    TotalSupply,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum PackagePartitionOffset {
    Fields,
    Blueprints,
    BlueprintDependencies,
    Royalty,
    AuthFunctionTemplates,
    AuthMethodTemplates,
}

impl TryFrom<u8> for PackagePartitionOffset {
    type Error = ();

    fn try_from(offset: u8) -> Result<Self, Self::Error> {
        Self::from_repr(offset).ok_or(())
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum NonFungibleResourceManagerPartitionOffset {
    ResourceManager,
    NonFungibleData,
}

impl TryFrom<u8> for NonFungibleResourceManagerPartitionOffset {
    type Error = ();

    fn try_from(offset: u8) -> Result<Self, Self::Error> {
        Self::from_repr(offset).ok_or(())
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum NonFungibleResourceManagerField {
    IdType,
    MutableFields,
    TotalSupply,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum FungibleVaultField {
    LiquidFungible,
    LockedFungible,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum NonFungibleVaultPartitionOffset {
    Balance,
    NonFungibles,
}

impl TryFrom<u8> for NonFungibleVaultPartitionOffset {
    type Error = ();

    fn try_from(offset: u8) -> Result<Self, Self::Error> {
        Self::from_repr(offset).ok_or(())
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum NonFungibleVaultField {
    LiquidNonFungible,
    LockedNonFungible,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum ConsensusManagerPartitionOffset {
    ConsensusManager,
    RegisteredValidatorsByStakeIndex,
}

impl TryFrom<u8> for ConsensusManagerPartitionOffset {
    type Error = ();

    fn try_from(offset: u8) -> Result<Self, Self::Error> {
        Self::from_repr(offset).ok_or(())
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum ConsensusManagerField {
    Config,
    ConsensusManager,
    CurrentValidatorSet,
    CurrentProposalStatistic,
    CurrentTimeRoundedToMinutes,
    CurrentTime,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum ValidatorField {
    Validator,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum FungibleBucketField {
    Liquid,
    Locked,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum NonFungibleBucketField {
    Liquid,
    Locked,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum FungibleProofField {
    Moveable,
    ProofRefs,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum NonFungibleProofField {
    Moveable,
    ProofRefs,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum WorktopField {
    Worktop,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum AccessControllerField {
    AccessController,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum AuthZoneField {
    AuthZone,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum AccountPartitionOffset {
    Account,
    AccountVaultsByResourceAddress,
    AccountResourceDepositRuleByAddress,
}

impl From<AccountPartitionOffset> for PartitionOffset {
    fn from(value: AccountPartitionOffset) -> Self {
        PartitionOffset(value as u8)
    }
}

impl TryFrom<u8> for AccountPartitionOffset {
    type Error = ();

    fn try_from(offset: u8) -> Result<Self, Self::Error> {
        Self::from_repr(offset).ok_or(())
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum AccountField {
    Account,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum OneResourcePoolField {
    OneResourcePool,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum TwoResourcePoolField {
    TwoResourcePool,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum MultiResourcePoolField {
    MultiResourcePool,
}

macro_rules! substate_key {
    ($t:ty) => {
        impl From<$t> for SubstateKey {
            fn from(value: $t) -> Self {
                SubstateKey::Tuple(value as u8)
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
                    SubstateKey::Tuple(x) => Self::from_repr(*x).ok_or(()),
                    _ => Err(()),
                }
            }
        }
    };
}

substate_key!(TypeInfoField);
substate_key!(RoyaltyField);
substate_key!(ComponentField);
substate_key!(PackageField);
substate_key!(FungibleResourceManagerField);
substate_key!(FungibleVaultField);
substate_key!(FungibleBucketField);
substate_key!(FungibleProofField);
substate_key!(NonFungibleResourceManagerField);
substate_key!(NonFungibleVaultField);
substate_key!(NonFungibleBucketField);
substate_key!(NonFungibleProofField);
substate_key!(ConsensusManagerField);
substate_key!(ValidatorField);
substate_key!(AccessControllerField);
substate_key!(AccountField);
substate_key!(OneResourcePoolField);
substate_key!(TwoResourcePoolField);
substate_key!(MultiResourcePoolField);

// Transient
substate_key!(WorktopField);
substate_key!(AuthZoneField);
