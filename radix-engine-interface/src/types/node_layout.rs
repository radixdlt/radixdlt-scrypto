use crate::types::*;
use crate::*;
use sbor::rust::prelude::*;

//=========================================================================
// Please update REP-60 after updating types/configs defined in this file!
//=========================================================================

pub const TYPE_INFO_BASE_MODULE: ModuleNumber = ModuleNumber(0u8);
pub const METADATA_BASE_MODULE: ModuleNumber = ModuleNumber(1u8);
pub const ROYALTY_BASE_MODULE: ModuleNumber = ModuleNumber(2u8);
pub const ACCESS_RULES_BASE_MODULE: ModuleNumber = ModuleNumber(3u8);
pub const OBJECT_BASE_MODULE: ModuleNumber = ModuleNumber(32u8);

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum AccessRulesOffset {
    AccessRules,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum TypeInfoOffset {
    TypeInfo,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum RoyaltyOffset {
    RoyaltyConfig,
    RoyaltyAccumulator,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum ComponentOffset {
    State0,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum PackageOffset {
    Info,
    CodeType,
    Code,
    Royalty,
    FunctionAccessRules,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum FungibleResourceManagerOffset {
    Divisibility,
    TotalSupply,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum NonFungibleResourceManagerModuleOffset {
    ResourceManager,
    NonFungibleData,
}

impl TryFrom<u8> for NonFungibleResourceManagerModuleOffset {
    type Error = ();

    fn try_from(offset: u8) -> Result<Self, Self::Error> {
        Self::from_repr(offset).ok_or(())
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum NonFungibleResourceManagerOffset {
    IdType,
    MutableFields,
    TotalSupply,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum FungibleVaultOffset {
    LiquidFungible,
    LockedFungible,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum NonFungibleVaultModuleOffset {
    Balance,
    NonFungibles,
}

impl TryFrom<u8> for NonFungibleVaultModuleOffset {
    type Error = ();

    fn try_from(offset: u8) -> Result<Self, Self::Error> {
        Self::from_repr(offset).ok_or(())
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum NonFungibleVaultOffset {
    LiquidNonFungible,
    LockedNonFungible,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum EpochManagerOffset {
    Config,
    EpochManager,
    CurrentValidatorSet,
    RegisteredValidators,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum ValidatorOffset {
    Validator,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum FungibleBucketOffset {
    Liquid,
    Locked,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum NonFungibleBucketOffset {
    Liquid,
    Locked,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum FungibleProofOffset {
    Moveable,
    ProofRefs,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum NonFungibleProofOffset {
    Moveable,
    ProofRefs,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum WorktopOffset {
    Worktop,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum ClockOffset {
    CurrentTimeRoundedToMinutes,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum AccessControllerOffset {
    AccessController,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum AuthZoneOffset {
    AuthZone,
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

substate_key!(AccessRulesOffset);
substate_key!(TypeInfoOffset);
substate_key!(RoyaltyOffset);
substate_key!(ComponentOffset);
substate_key!(PackageOffset);
substate_key!(FungibleResourceManagerOffset);
substate_key!(FungibleVaultOffset);
substate_key!(FungibleBucketOffset);
substate_key!(FungibleProofOffset);
substate_key!(NonFungibleResourceManagerOffset);
substate_key!(NonFungibleVaultOffset);
substate_key!(NonFungibleBucketOffset);
substate_key!(NonFungibleProofOffset);
substate_key!(EpochManagerOffset);
substate_key!(ValidatorOffset);

// Transient
substate_key!(WorktopOffset);
substate_key!(ClockOffset);
substate_key!(AccessControllerOffset);
substate_key!(AuthZoneOffset);
