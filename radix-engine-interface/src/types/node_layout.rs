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
pub enum TypedModuleId {
    TypeInfo,
    ObjectState,
    Metadata,
    Royalty,
    AccessRules,
}

impl Into<ModuleId> for TypedModuleId {
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
pub enum ResourceManagerOffset {
    ResourceManager,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VaultOffset {
    Info,
    LiquidFungible,
    LockedFungible,
    LiquidNonFungible,
    LockedNonFungible,
}

#[repr(u8)]
#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EpochManagerOffset {
    EpochManager,
    CurrentValidatorSet,
    PreparingValidatorSet,
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
                SubstateKey::from_vec(vec![self as u8])
                    .expect("Failed to convert offset into substate key")
            }
        }
    };
}

into_substate_key!(AccessRulesOffset);
into_substate_key!(TypeInfoOffset);
into_substate_key!(RoyaltyOffset);
into_substate_key!(ComponentOffset);
into_substate_key!(PackageOffset);
into_substate_key!(ResourceManagerOffset);
into_substate_key!(VaultOffset);
into_substate_key!(EpochManagerOffset);
into_substate_key!(ValidatorOffset);
into_substate_key!(BucketOffset);
into_substate_key!(ProofOffset);
into_substate_key!(WorktopOffset);
into_substate_key!(ClockOffset);
into_substate_key!(AccountOffset);
into_substate_key!(AccessControllerOffset);
into_substate_key!(AuthZoneOffset);
