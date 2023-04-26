use strum::FromRepr;

//=========================================================================
// Please update REP-60 after updating types/configs defined in this file!
//=========================================================================

/// An enum which represents the different addressable entities.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, FromRepr)]
pub enum EntityType {
    GlobalPackage,
    GlobalFungibleResource,
    GlobalNonFungibleResource,
    GlobalEpochManager,
    GlobalValidator,
    GlobalClock,
    GlobalAccessController,
    GlobalAccount,
    GlobalIdentity,
    GlobalGenericComponent, // generic

    GlobalVirtualEcdsaAccount,
    GlobalVirtualEddsaAccount,
    GlobalVirtualEcdsaIdentity,
    GlobalVirtualEddsaIdentity,

    InternalFungibleVault,
    InternalNonFungibleVault,
    InternalAccount,
    InternalKeyValueStore,
    InternalIndex,
    InternalSortedIndex,
    InternalGenericComponent, // generic
}
