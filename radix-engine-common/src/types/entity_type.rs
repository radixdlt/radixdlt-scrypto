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

    InternalVault,
    InternalAccessController,
    InternalAccount,
    InternalKeyValueStore,
    InternalGenericComponent, // generic
}

impl EntityType {
    pub fn is_global(&self) -> bool {
        match self {
            EntityType::GlobalPackage
            | EntityType::GlobalFungibleResource
            | EntityType::GlobalNonFungibleResource
            | EntityType::GlobalEpochManager
            | EntityType::GlobalValidator
            | EntityType::GlobalClock
            | EntityType::GlobalAccessController
            | EntityType::GlobalAccount
            | EntityType::GlobalIdentity
            | EntityType::GlobalGenericComponent
            | EntityType::GlobalVirtualEcdsaAccount
            | EntityType::GlobalVirtualEddsaAccount
            | EntityType::GlobalVirtualEcdsaIdentity
            | EntityType::GlobalVirtualEddsaIdentity => true,
            EntityType::InternalVault
            | EntityType::InternalAccessController
            | EntityType::InternalAccount
            | EntityType::InternalGenericComponent
            | EntityType::InternalKeyValueStore => false,
        }
    }

    pub fn is_local(&self) -> bool {
        !self.is_global()
    }

    pub fn is_global_component(&self) -> bool {
        match self {
        EntityType::GlobalEpochManager |
        EntityType::GlobalValidator |
        EntityType::GlobalClock |
        EntityType::GlobalAccessController |
        EntityType::GlobalAccount |
        EntityType::GlobalIdentity |
        EntityType::GlobalGenericComponent |
        EntityType::GlobalVirtualEcdsaAccount |
        EntityType::GlobalVirtualEddsaAccount |
        EntityType::GlobalVirtualEcdsaIdentity |
        EntityType::GlobalVirtualEddsaIdentity => true,
        EntityType::GlobalPackage | /* PackageAddress */
        EntityType::GlobalFungibleResource | /* ResourceAddress */
        EntityType::GlobalNonFungibleResource | /* ResourceAddress */
        EntityType::InternalVault |
        EntityType::InternalAccessController |
        EntityType::InternalAccount |
        EntityType::InternalGenericComponent |
        EntityType::InternalKeyValueStore => false,
    }
    }

    pub fn is_global_package(&self) -> bool {
        matches!(self, EntityType::GlobalPackage)
    }

    pub fn is_global_resource(&self) -> bool {
        matches!(
            self,
            EntityType::GlobalFungibleResource | EntityType::GlobalNonFungibleResource
        )
    }

    pub fn is_global_virtual(&self) -> bool {
        match self {
            EntityType::GlobalVirtualEcdsaAccount
            | EntityType::GlobalVirtualEddsaAccount
            | EntityType::GlobalVirtualEcdsaIdentity
            | EntityType::GlobalVirtualEddsaIdentity => true,
            _ => false,
        }
    }

    pub fn is_internal_kv_store(&self) -> bool {
        matches!(self, EntityType::InternalKeyValueStore)
    }

    pub fn is_internal_vault(&self) -> bool {
        matches!(self, EntityType::InternalVault)
    }
}
