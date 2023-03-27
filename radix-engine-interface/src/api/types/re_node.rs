use crate::*;
use sbor::rust::prelude::*;

//=========================================================================
// Please update REP-60 after updating types/configs defined in this file!
//=========================================================================

/// The unique identifier of a (stored) node.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor)]
#[sbor(transparent)]
pub struct NodeId([u8; Self::LENGTH]);

impl NodeId {
    pub const LENGTH: usize = 27;

    pub fn new(entity_byte: u8, random_bytes: &[u8; Self::LENGTH - 1]) -> Self {
        let mut buf = [0u8; Self::LENGTH];
        buf[0] = entity_byte;
        buf[1..random_bytes.len() + 1].copy_from_slice(random_bytes);
        Self(buf)
    }
}

impl AsRef<[u8]> for NodeId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Into<[u8; NodeId::LENGTH]> for NodeId {
    fn into(self) -> [u8; NodeId::LENGTH] {
        self.0
    }
}

/// The unique identifier of a node module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor)]
#[sbor(transparent)]
pub struct ModuleId(pub u8);

/// The unique identifier of a substate within a node module.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor)]
#[sbor(transparent)]
pub struct SubstateKey(Vec<u8>);

impl SubstateKey {
    pub const MIN_LENGTH: usize = 1;
    pub const MAX_LENGTH: usize = 128;
    pub const MIN: Self = Self(vec![u8::MIN; Self::MIN_LENGTH]);
    pub const MAX: Self = Self(vec![u8::MAX; Self::MAX_LENGTH]);

    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        Self::from_vec(slice.to_vec())
    }

    pub fn from_vec(bytes: Vec<u8>) -> Option<Self> {
        if bytes.len() < Self::MIN_LENGTH || bytes.len() > Self::MAX_LENGTH {
            None
        } else {
            Some(Self(bytes))
        }
    }
}

impl AsRef<[u8]> for SubstateKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Into<Vec<u8>> for SubstateKey {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

//=================
// ModuleId Config
//=================

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

//====================
// SubstateKey Config
//====================

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
