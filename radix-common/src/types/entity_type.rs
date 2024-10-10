use crate::prelude::*;
use sbor::Sbor;
use strum::FromRepr;
use strum::IntoStaticStr;

//=========================================================================
// Please update REP-60 after updating types/configs defined in this file!
// Please use and update REP-71 for choosing an entity type prefix
//=========================================================================

/// An enum which represents the different addressable entities.
#[repr(u8)]
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Ord, Hash, PartialOrd, FromRepr, Sbor, IntoStaticStr,
)]
#[sbor(use_repr_discriminators)]
pub enum EntityType {
    //=========================================================================
    // Package (start with char p for package)
    //=========================================================================
    /// A global package entity (13 in decimal). Gives Bech32 prefix: `p` followed by one of `5`, `4`, `k` or `h`.
    GlobalPackage = 0b00001101, //------------------- 00001 => p, 101xx => 54kh [pkg vanity prefix]

    //=========================================================================
    // System Components (start with char s for system)
    //=========================================================================
    /// The global consensus manager entity (134 in decimal). Gives Bech32 prefix: `s` followed by one of `c`, `e`, `6` or `m`.
    GlobalConsensusManager = 0b10000110, //-------------- 10000 => s, 110xx => ce6m [se vanity prefix]

    /// A global validator entity (131 in decimal). Gives Bech32 prefix: `s` followed by one of `v`, `d`, `w` or `0`.
    GlobalValidator = 0b10000011, //--------------------- 10000 => s, 011xx => vdw0

    /// A global transaction tracker (130 in decimal). Gives Bech32 prefix: `s` followed by one of `g`, `f`, `2` or `t`.
    GlobalTransactionTracker = 0b10000010, //-------- 10000 => s, 010xx => gf2t [st vanity prefix]

    //=========================================================================
    // Standard Global Components (start with char c for component)
    //=========================================================================
    /// A global generic (eg scrypto) component entity (192 in decimal). Gives Bech32 prefix: `c` followed by one of `q`, `p`, `z` or `r`.
    GlobalGenericComponent = 0b11000000, //---------- 11000 => c, 000xx => qpzr [cpt vanity prefix] (000 = generic component)

    /// A global allocated native account component entity (193 in decimal). Gives Bech32 prefix: `c` followed by one of `y`, `9`, `x` or `8`.
    GlobalAccount = 0b11000001, //------------------- 11000 => c, 001xx => y9x8 (001 = account)

    /// A global allocated native identity component entity (194 in decimal). Gives Bech32 prefix: `c` followed by one of `g`, `f`, `2` or `t`.
    GlobalIdentity = 0b11000010, //------------------ 11000 => c, 010xx => gf2t (010 = identity)

    /// A global native access controller entity (195 in decimal). Gives Bech32 prefix: `c` followed by one of `v`, `d`, `w` or `0`.
    GlobalAccessController = 0b11000011, //---------- 11000 => c, 011xx => vdw0 (011 = access controller)

    /// A global native pool entity (196 in decimal). Gives Bech32 prefix: `c` followed by one of `s`, `3`, `j` or `n`.
    GlobalOneResourcePool = 0b11000100, //----------- 11000 => c, 100xx => s3jn (100 = pool)

    /// A global native pool entity (197 in decimal). Gives Bech32 prefix: `c` followed by one of `5`, `4`, `k` or `h`.
    GlobalTwoResourcePool = 0b11000101, //----------- 11000 => c, 101xx => 54kh (101 = pool)

    /// A global native pool entity (198 in decimal). Gives Bech32 prefix: `c` followed by one of `c`, `e`, `6` or `m`.
    GlobalMultiResourcePool = 0b11000110, //--------- 11000 => c, 110xx => ce6m (101 = pool)

    //=========================================================================
    // Standard Global Components (start with char d since c is fully taken)
    //=========================================================================
    /// A global native locker component (104 in decimal). Gives Bech32 prefix: `d` followed by one of `q`, `p`, `z` or `r`.
    GlobalAccountLocker = 0b01101000, //--------- 01101 => d, 000xx => qpzr (000 = account locker)

    //=========================================================================
    // Secp256k1 Preallocated Global Components (start with char 6 for Secp256k1)
    //=========================================================================
    /// A global preallocated Secp256k1 account component entity (209 in decimal). Gives Bech32 prefix: `6` followed by one of `y`, `9`, `x` or `8`.
    GlobalPreallocatedSecp256k1Account = 0b11010001, //--- 11010 => 6, 001xx => y9x8 (001 = account)

    /// A global preallocated Secp256k1 identity component entity (210 in decimal). Gives Bech32 prefix: `6` followed by one of `g`, `f`, `2` or `t`.
    GlobalPreallocatedSecp256k1Identity = 0b11010010, //-- 11010 => 6, 010xx => gf2t (010 = identity)

    //=========================================================================
    // Ed25519 Preallocated Global Components (start with char 2 for Ed25519)
    //=========================================================================
    /// A global preallocated Ed25519 account component entity (81 in decimal). Gives Bech32 prefix: `2` followed by one of `y`, `9`, `x` or `8`.
    GlobalPreallocatedEd25519Account = 0b01010001, //----- 01010 => 2, 001xx => y9x8 (001 = account)

    /// A global preallocated Ed25519 identity component entity (82 in decimal). Gives Bech32 prefix: `2` followed by one of `g`, `f`, `2` or `t`.
    GlobalPreallocatedEd25519Identity = 0b01010010, //---- 01010 => 2, 010xx => gf2t (010 = identity)

    //=========================================================================
    // Fungible-related (start with letter t for token)
    //=========================================================================
    /// A global fungible resource entity (93 in decimal). Gives Bech32 prefix: `t` followed by one of `5`, `4`, `k` or `h`.
    GlobalFungibleResourceManager = 0b01011101, //---------- 01011 => t, 101xx => 54kh [tkn vanity prefix]
    /// An internal fungible vault entity (88 in decimal). Gives Bech32 prefix: `t` followed by one of `q`, `p`, `z` or `r`.
    InternalFungibleVault = 0b01011000, //------------------ 01011 => t, 000xx => qpzr (000 = vault under t/f prefix)

    //=========================================================================
    // Non-fungible-related (start with letter n for non-fungible)
    //=========================================================================
    /// A global non-fungible resource entity (154 in decimal). Gives Bech32 prefix: `n` followed by one of `g`, `f`, `2` or `t`.
    GlobalNonFungibleResourceManager = 0b10011010, //------- 10011 => n, 010xx => gf2t [nf  vanity prefix]

    /// An internal non-fungible vault entity (152 in decimal). Gives Bech32 prefix: `n` followed by one of `q`, `p`, `z` or `r`.
    InternalNonFungibleVault = 0b10011000, //-------- 10011 => n, 000xx => qpzr (000 = vault under t/f prefix)

    //=========================================================================
    // Internal misc components (start with letter l for ..? local)
    //=========================================================================
    /// An internal generic (eg scrypto) component entity (248 in decimal). Gives Bech32 prefix: `l` followed by one of `q`, `p`, `z` or `r`.
    InternalGenericComponent = 0b11111000, //-------- 11111 => l, 000xx => qpzr (000 = generic component)

    //=========================================================================
    // Internal key-value-store-like entities (start with k for key-value)
    //=========================================================================
    /// An internal key-value store entity (176 in decimal). Gives Bech32 prefix: `k` followed by one of `q`, `p`, `z` or `r`.
    ///
    /// A key value store allows access to substates, but not on-ledger iteration.
    /// The substates are considered independent for contention/locking/versioning.
    InternalKeyValueStore = 0b10110000, //----------- 10110 => k, 000xx => qpzr
}

impl EntityType {
    pub const fn is_global(&self) -> bool {
        match self {
            EntityType::GlobalPackage
            | EntityType::GlobalFungibleResourceManager
            | EntityType::GlobalNonFungibleResourceManager
            | EntityType::GlobalConsensusManager
            | EntityType::GlobalValidator
            | EntityType::GlobalAccessController
            | EntityType::GlobalAccount
            | EntityType::GlobalIdentity
            | EntityType::GlobalGenericComponent
            | EntityType::GlobalPreallocatedSecp256k1Account
            | EntityType::GlobalPreallocatedEd25519Account
            | EntityType::GlobalPreallocatedSecp256k1Identity
            | EntityType::GlobalPreallocatedEd25519Identity
            | EntityType::GlobalOneResourcePool
            | EntityType::GlobalTwoResourcePool
            | EntityType::GlobalMultiResourcePool
            | EntityType::GlobalTransactionTracker
            | EntityType::GlobalAccountLocker => true,
            EntityType::InternalFungibleVault
            | EntityType::InternalNonFungibleVault
            | EntityType::InternalGenericComponent
            | EntityType::InternalKeyValueStore => false,
        }
    }

    pub const fn is_internal(&self) -> bool {
        !self.is_global()
    }

    pub const fn is_global_component(&self) -> bool {
        match self {
            EntityType::GlobalConsensusManager
            | EntityType::GlobalValidator
            | EntityType::GlobalAccessController
            | EntityType::GlobalAccount
            | EntityType::GlobalIdentity
            | EntityType::GlobalGenericComponent
            | EntityType::GlobalPreallocatedSecp256k1Account
            | EntityType::GlobalPreallocatedEd25519Account
            | EntityType::GlobalPreallocatedSecp256k1Identity
            | EntityType::GlobalPreallocatedEd25519Identity
            | EntityType::GlobalOneResourcePool
            | EntityType::GlobalTwoResourcePool
            | EntityType::GlobalMultiResourcePool
            | EntityType::GlobalTransactionTracker
            | EntityType::GlobalAccountLocker => true,
            EntityType::GlobalPackage
            | EntityType::GlobalFungibleResourceManager
            | EntityType::GlobalNonFungibleResourceManager
            | EntityType::InternalFungibleVault
            | EntityType::InternalNonFungibleVault
            | EntityType::InternalGenericComponent
            | EntityType::InternalKeyValueStore => false,
        }
    }

    pub const fn is_global_package(&self) -> bool {
        matches!(self, EntityType::GlobalPackage)
    }

    pub const fn is_global_account(&self) -> bool {
        matches!(
            self,
            EntityType::GlobalAccount
                | EntityType::GlobalPreallocatedSecp256k1Account
                | EntityType::GlobalPreallocatedEd25519Account
        )
    }

    pub const fn is_global_consensus_manager(&self) -> bool {
        matches!(self, EntityType::GlobalConsensusManager)
    }

    pub const fn is_global_validator(&self) -> bool {
        matches!(self, EntityType::GlobalValidator)
    }

    pub const fn is_global_resource_manager(&self) -> bool {
        matches!(
            self,
            EntityType::GlobalFungibleResourceManager
                | EntityType::GlobalNonFungibleResourceManager
        )
    }

    pub const fn is_global_fungible_resource_manager(&self) -> bool {
        matches!(self, EntityType::GlobalFungibleResourceManager)
    }

    pub const fn is_global_non_fungible_resource_manager(&self) -> bool {
        matches!(self, EntityType::GlobalNonFungibleResourceManager)
    }

    pub const fn is_global_preallocated(&self) -> bool {
        match self {
            EntityType::GlobalPreallocatedSecp256k1Account
            | EntityType::GlobalPreallocatedEd25519Account
            | EntityType::GlobalPreallocatedSecp256k1Identity
            | EntityType::GlobalPreallocatedEd25519Identity => true,
            _ => false,
        }
    }

    pub const fn is_internal_kv_store(&self) -> bool {
        matches!(self, EntityType::InternalKeyValueStore)
    }

    pub const fn is_internal_fungible_vault(&self) -> bool {
        matches!(self, EntityType::InternalFungibleVault)
    }

    pub const fn is_internal_non_fungible_vault(&self) -> bool {
        matches!(self, EntityType::InternalNonFungibleVault)
    }

    pub const fn is_internal_vault(&self) -> bool {
        matches!(
            self,
            EntityType::InternalFungibleVault | EntityType::InternalNonFungibleVault
        )
    }
}
