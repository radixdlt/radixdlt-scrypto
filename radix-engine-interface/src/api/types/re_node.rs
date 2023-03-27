use crate::*;
use sbor::rust::prelude::*;


//==========================================================
// Please update REP-60 after updating types defined here!
//==========================================================


//===============
// NodeId
//===============

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor)]
pub enum EntityType {
    GlobalPackage,
    GlobalFungibleResourceManager,
    GlobalNonFungibleResourceManager,
    GlobalEpochManager,
    GlobalValidator,
    GlobalClock,
    GlobalAccessController,
    GlobalAccount,
    GlobalIdentity,
    GlobalComponent,

    GlobalVirtualEcdsaAccount,
    GlobalVirtualEddsaAccount,
    GlobalVirtualEcdsaIdentity,
    GlobalVirtualEddsaIdentity,

    InternalVault,
    InternalAccessController,
    InternalAccount,
    InternalComponent,
    InternalKeyValueStore,
}

//===============
// ModuleId
//===============

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor, ManifestSbor)]
pub enum TypedModuleId {
    TypeInfo,
    ObjectState,
    KeyValueStore,
    Metadata,
    Royalty,
    AccessRules,
    AccessRules1, // TODO: remove
}

//===============
// SubstateKey
//===============

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
