use crate::kernel::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::collections::HashMap;
use crate::rust::collections::HashSet;
use crate::rust::string::String;

/// Utility for creating resources.
pub struct ResourceBuilder {
    resource_type: ResourceType,
    metadata: HashMap<String, String>,
    initial_supply: Option<ResourceSupply>,
    configs: ResourceConfigs,
}

impl ResourceBuilder {
    /// Starts a new builder to create fungible resource, e.g., tokens and badges.
    ///
    /// Granularity decides the divisibility of the resource:
    /// * If the granularity is `1`, the smallest unit is `10^-18`;
    /// * If the granularity is `2`, the smallest unit is `10^-17`;
    /// * So on and so forth.
    pub fn new_fungible(granularity: u8) -> Self {
        Self {
            resource_type: ResourceType::Fungible { granularity },
            metadata: HashMap::new(),
            initial_supply: None,
            configs: ResourceConfigs::new(HashSet::new(), HashSet::new(), HashMap::new()),
        }
    }

    /// Starts a new builder to create non_fungible resource, e.g. NFT.
    pub fn new_non_fungible() -> Self {
        Self {
            resource_type: ResourceType::NonFungible,
            metadata: HashMap::new(),
            initial_supply: None,
            configs: ResourceConfigs::new(HashSet::new(), HashSet::new(), HashMap::new()),
        }
    }

    /// Adds a shared metadata to the resource to be created.
    pub fn with_metadata<K: AsRef<str>, V: AsRef<str>>(&mut self, name: K, value: V) -> &mut Self {
        self.metadata
            .insert(name.as_ref().to_owned(), value.as_ref().to_owned());
        self
    }

    /// Sets the initial supply.
    pub fn with_initial_supply(&mut self, supply: ResourceSupply) -> &mut Self {
        self.initial_supply = Some(supply);
        self
    }

    /// Enables a feature on the resource.
    pub fn with_feature(&mut self, feature: ResourceFeature) -> &mut Self {
        self.configs.enable_feature(feature).unwrap();
        self
    }

    /// Adds a permission configuration to the resource.
    pub fn with_permission<T: Into<ResourceDef>>(
        &mut self,
        permission: ResourcePermission,
        authorities: Vec<T>,
    ) -> &mut Self {
        let mut addresses = Vec::new();
        for a in authorities {
            addresses.push(a.into().address());
        }
        self.configs.set_authority(permission, addresses);
        self
    }

    /// If enabled, resource can be transferred by the holders, with the specified authority badge.
    pub fn transferable<T: Into<ResourceDef>>(&mut self, authorities: Vec<T>) -> &mut Self {
        self.with_feature(ResourceFeature::Transferable)
            .with_permission(ResourcePermission::Transfer, authorities)
    }

    /// If enabled, resource can be freely transferred by the holders.
    pub fn freely_transferable(&mut self) -> &mut Self {
        self.with_feature(ResourceFeature::Transferable)
    }

    /// If enabled, resource can be minted, with the specified authority badge.
    pub fn mintable<T: Into<ResourceDef>>(&mut self, authorities: Vec<T>) -> &mut Self {
        self.with_feature(ResourceFeature::Mintable)
            .with_permission(ResourcePermission::Mint, authorities)
    }

    /// If enabled, resource can be burned by the holder, with the specified authority badge.
    pub fn burnable<T: Into<ResourceDef>>(&mut self, authorities: Vec<T>) -> &mut Self {
        self.with_feature(ResourceFeature::Burnable)
            .with_permission(ResourcePermission::Burn, authorities)
    }

    /// If enabled, resource can be freely burned by the holders.
    pub fn freely_burnable(&mut self) -> &mut Self {
        self.with_feature(ResourceFeature::FreelyBurnable)
    }

    /// If enabled, shared resource metadata can be updated, with the specified authority badge.
    pub fn shared_metadata_mutable<T: Into<ResourceDef>>(
        &mut self,
        authorities: Vec<T>,
    ) -> &mut Self {
        self.with_feature(ResourceFeature::SharedMetadataMutable)
            .with_permission(ResourcePermission::ChangeSharedMetadata, authorities)
    }

    /// If enabled, individual resource metadata can be updated, with the specified authority badge.
    pub fn individual_metadata_mutable<T: Into<ResourceDef>>(
        &mut self,
        authorities: Vec<T>,
    ) -> &mut Self {
        self.with_feature(ResourceFeature::IndividualMetadataMutable)
            .with_permission(ResourcePermission::ChangeIndividualMetadata, authorities)
    }

    /// If enabled, resource can be seized, with the specified authority badge.
    pub fn clawbackable<T: Into<ResourceDef>>(&mut self, authorities: Vec<T>) -> &mut Self {
        self.with_feature(ResourceFeature::Clawbackable)
            .with_permission(ResourcePermission::Clawback, authorities)
    }

    /// If enabled, resource feature can be toggled on/off, with the specified authority badge.
    pub fn managed_by<T: Into<ResourceDef>>(&mut self, authorities: Vec<T>) -> &mut Self {
        self.with_permission(ResourcePermission::ManageResourceFeatures, authorities)
    }

    /// Creates a resource based on previous specifications.
    ///
    /// A bucket will be returned, but it may be empty if no initial supply is provided.
    pub fn build(self) -> Bucket {
        ResourceDef::new(
            self.resource_type,
            self.metadata,
            self.initial_supply.unwrap_or(match self.resource_type {
                ResourceType::Fungible { .. } => ResourceSupply::Fungible { amount: 0.into() },
                ResourceType::NonFungible { .. } => ResourceSupply::NonFungible {
                    entries: HashMap::new(),
                },
            }),
            self.configs,
        )
    }
}
