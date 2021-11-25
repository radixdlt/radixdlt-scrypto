use sbor::{Decode, Describe, Encode, TypeId};

use crate::rust::collections::HashMap;
use crate::rust::collections::HashSet;
use crate::rust::vec::Vec;
use crate::types::*;

/// Represents the level of a log message.
#[derive(Debug, Clone, Copy, TypeId, Encode, Decode, Describe, Eq, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Represents the type of a resource.
#[derive(Debug, Clone, Copy, TypeId, Encode, Decode, Describe, Eq, PartialEq)]
pub enum ResourceType {
    /// Represents a fungible resource
    Fungible { granularity: u8 },

    /// Represents a non-fungible resource
    NonFungible,
}

/// Represents som supply of resource.
#[derive(Debug, Clone, TypeId, Encode, Decode, Describe)]
pub enum ResourceSupply {
    /// A supply of fungible resource represented by amount.
    Fungible { amount: Decimal },

    /// A supply of non-fungible resource represented by a collection of NFTs keyed by ID.
    NonFungible { entries: HashMap<u128, Vec<u8>> },
}

/// Represents a resource feature.
#[derive(Debug, Clone, Copy, TypeId, Encode, Decode, Describe, Eq, PartialEq, Hash)]
pub enum ResourceFeature {
    /// Can be transferred.
    Transferable,
    /// Can be transferred freely, without any authority.
    FreelyTransferable,
    /// More supply can be minted.
    Mintable,
    /// Tokens can be burned.
    Burnable,
    /// May be burned by the holder, without any authority.
    FreelyBurnable,
    /// Top-level metadata can be changed.
    SharedMetadataMutable,
    /// The mutable data part of an individual NFT can be modified.
    IndividualMetadataMutable,
    /// Can be seized from any vault if the proper authority is presented.
    Clawbackable,
}

/// Represents the permission to apply some operation on a resource.
#[derive(Debug, Clone, Copy, TypeId, Encode, Decode, Describe, Eq, PartialEq, Hash)]
pub enum ResourcePermission {
    /// To transfer, useful when `FREELY_TRANSFERABLE` is off.
    Transfer,
    /// To create new supply.
    Mint,
    /// To burn.
    Burn,
    /// To update/lock resource features.
    ManageResourceFeatures,
    /// To change the shared metadata.
    ChangeSharedMetadata,
    /// To alter the contents of the mutable part of an individual NFT’s data.
    ChangeIndividualMetadata,
    /// To seize from any vault.
    Clawback,
}

/// Represents the configuration of a resource:
/// * The enabled feature set
/// * The locked feature set
/// * The badge list for each permission
///
/// TODO: implement set using bitmask for storage efficiency
#[derive(Debug, Clone, TypeId, Encode, Decode, Describe, Eq, PartialEq)]
pub struct ResourceConfigs {
    pub enabled_features: HashSet<ResourceFeature>,
    pub locked_features: HashSet<ResourceFeature>,
    pub permissions: HashMap<ResourcePermission, Vec<Address>>,
}

#[derive(Debug, Clone)]
pub enum ResourceConfigsError {
    FeatureAlreadyEnabled,
    FeatureAlreadyDisabled,
    FeatureAlreadyLocked,
    FeatureLocked,
}

impl ResourceConfigs {
    pub fn new(
        enabled_features: HashSet<ResourceFeature>,
        locked_features: HashSet<ResourceFeature>,
        permissions: HashMap<ResourcePermission, Vec<Address>>,
    ) -> Self {
        Self {
            enabled_features,
            locked_features,
            permissions,
        }
    }

    pub fn is_feature_enabled(&self, feature: ResourceFeature) -> bool {
        self.enabled_features.contains(&feature)
    }

    pub fn is_feature_locked(&self, feature: ResourceFeature) -> bool {
        self.locked_features.contains(&feature)
    }

    pub fn enable_feature(&mut self, feature: ResourceFeature) -> Result<(), ResourceConfigsError> {
        if self.is_feature_locked(feature) {
            Err(ResourceConfigsError::FeatureLocked)
        } else if self.is_feature_enabled(feature) {
            Err(ResourceConfigsError::FeatureAlreadyEnabled)
        } else {
            self.enabled_features.insert(feature);
            Ok(())
        }
    }

    pub fn disable_feature(
        &mut self,
        feature: ResourceFeature,
    ) -> Result<(), ResourceConfigsError> {
        if self.is_feature_locked(feature) {
            Err(ResourceConfigsError::FeatureLocked)
        } else if !self.is_feature_enabled(feature) {
            Err(ResourceConfigsError::FeatureAlreadyDisabled)
        } else {
            self.enabled_features.remove(&feature);
            Ok(())
        }
    }

    pub fn lock_feature(&mut self, feature: ResourceFeature) -> Result<(), ResourceConfigsError> {
        if self.is_feature_locked(feature) {
            Err(ResourceConfigsError::FeatureAlreadyLocked)
        } else {
            self.locked_features.insert(feature);
            Ok(())
        }
    }

    pub fn set_authority(&mut self, permission: ResourcePermission, authority: Vec<Address>) {
        self.permissions.insert(permission, authority);
    }
}
