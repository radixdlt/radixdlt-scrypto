use crate::*;
use radix_engine_common::types::{ModuleId, SubstateKey};
use sbor::rust::prelude::*;
use strum::{EnumIter, FromRepr};

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

impl Into<ModuleId> for SysModuleId {
    fn into(self) -> ModuleId {
        ModuleId(self as u8)
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AccessRulesOffset {
    AccessRules,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TypeInfoOffset {
    TypeInfo,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RoyaltyOffset {
    RoyaltyConfig,
    RoyaltyAccumulator,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ComponentOffset {
    State0,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PackageOffset {
    Info,
    CodeType,
    Code,
    Royalty,
    FunctionAccessRules,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FungibleResourceManagerOffset {
    Divisibility,
    TotalSupply,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NonFungibleResourceManagerOffset {
    IdType,
    DataSchema,
    TotalSupply,
    Data,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FungibleVaultOffset {
    LiquidFungible,
    LockedFungible,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NonFungibleVaultOffset {
    LiquidNonFungible,
    LockedNonFungible,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EpochManagerOffset {
    Config,
    EpochManager,
    CurrentValidatorSet,
    RegisteredValidators,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ValidatorOffset {
    Validator,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BucketOffset {
    Info,
    LiquidFungible,
    LockedFungible,
    LiquidNonFungible,
    LockedNonFungible,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProofOffset {
    Info,
    Fungible,
    NonFungible,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WorktopOffset {
    Worktop,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ClockOffset {
    CurrentTimeRoundedToMinutes,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AccountOffset {
    Account,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AccessControllerOffset {
    AccessController,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AuthZoneOffset {
    AuthZone,
}

macro_rules! into_substate_key {
    ($t:ty) => {
        impl Into<SubstateKey> for $t {
            fn into(self) -> SubstateKey {
                SubstateKey::Tuple(self as u8)
            }
        }

        impl Into<u8> for $t {
            fn into(self) -> u8 {
                self as u8
            }
        }
    };
}

into_substate_key!(AccessRulesOffset);
into_substate_key!(TypeInfoOffset);
into_substate_key!(RoyaltyOffset);
into_substate_key!(ComponentOffset);
into_substate_key!(PackageOffset);
into_substate_key!(FungibleResourceManagerOffset);
into_substate_key!(FungibleVaultOffset);
into_substate_key!(NonFungibleResourceManagerOffset);
into_substate_key!(NonFungibleVaultOffset);
into_substate_key!(EpochManagerOffset);
into_substate_key!(ValidatorOffset);
into_substate_key!(BucketOffset);
into_substate_key!(ProofOffset);
into_substate_key!(WorktopOffset);
into_substate_key!(ClockOffset);
into_substate_key!(AccountOffset);
into_substate_key!(AccessControllerOffset);
into_substate_key!(AuthZoneOffset);
