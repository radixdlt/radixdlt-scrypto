use crate::*;
use crate::types::*;
use sbor::rust::prelude::*;

//=========================================================================
// Please update REP-60 after updating types/configs defined in this file!
//=========================================================================

#[repr(u8)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    ScryptoSbor,
    ManifestSbor,
    FromRepr,
    EnumIter,
)]
pub enum SysModuleId {
    TypeInfo,
    Metadata,
    Royalty,
    AccessRules,
    Object,
    Virtualized,
}

impl From<SysModuleId> for ModuleId {
    fn from(value: SysModuleId) -> Self {
        Self(value as u8)
    }
}

impl TryFrom<ModuleId> for SysModuleId {
    type Error = ();

    fn try_from(key: ModuleId) -> Result<Self, Self::Error> {
        Self::from_repr(key.0).ok_or(())
    }
}

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
pub enum ResourceManagerOffset {
    ResourceManager,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum FungibleVaultOffset {
    Divisibility,
    LiquidFungible,
    LockedFungible,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum NonFungibleVaultOffset {
    IdType,
    LiquidNonFungible,
    LockedNonFungible,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum EpochManagerOffset {
    EpochManager,
    CurrentValidatorSet,
    RegisteredValidatorSet,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum ValidatorOffset {
    Validator,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum BucketOffset {
    Info,
    Liquid,
    Locked,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
pub enum ProofOffset {
    Info,
    Fungible,
    NonFungible,
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
pub enum AccountOffset {
    Account,
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
substate_key!(ResourceManagerOffset);
substate_key!(FungibleVaultOffset);
substate_key!(NonFungibleVaultOffset);
substate_key!(EpochManagerOffset);
substate_key!(ValidatorOffset);
substate_key!(ClockOffset);
substate_key!(AccountOffset);
substate_key!(AccessControllerOffset);
// Transient
substate_key!(WorktopOffset);
substate_key!(AuthZoneOffset);
substate_key!(BucketOffset);
substate_key!(ProofOffset);
