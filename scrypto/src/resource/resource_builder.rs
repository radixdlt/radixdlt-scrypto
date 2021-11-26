use crate::kernel::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::collections::HashMap;
use crate::rust::string::String;
use crate::types::*;

/// Utility for creating resources.
pub struct ResourceBuilder {
    resource_type: ResourceType,
    metadata: HashMap<String, String>,
    granularity: u8,
    flags: u16,
    mutable_flags: u16,
    authorities: HashMap<Address, u16>,
}

impl ResourceBuilder {
    /// Starts a new builder.
    pub fn new(resource_type: ResourceType) -> Self {
        Self {
            resource_type,
            metadata: HashMap::new(),
            granularity: 1,
            flags: 0,
            mutable_flags: 0,
            authorities: HashMap::new(),
        }
    }

    /// Starts a new builder to create fungible resource, e.g., tokens.
    pub fn new_fungible() -> Self {
        Self::new(ResourceType::Fungible)
    }

    /// Starts a new builder to create non-fungible resource, e.g. NFT.
    pub fn new_non_fungible() -> Self {
        Self::new(ResourceType::NonFungible)
    }

    /// Sets the resource granularity.
    ///
    /// * If the granularity is `1`, the smallest unit is `10^-18`;
    /// * If the granularity is `2`, the smallest unit is `10^-17`;
    /// * So on and so forth.
    pub fn granularity(&mut self, granularity: u8) -> &mut Self {
        self.granularity = granularity;
        self
    }

    /// Adds a shared metadata.
    ///
    /// If a previous attribute with the same name has been set, it will be overwritten.
    pub fn metadata<K: AsRef<str>, V: AsRef<str>>(&mut self, name: K, value: V) -> &mut Self {
        self.metadata
            .insert(name.as_ref().to_owned(), value.as_ref().to_owned());
        self
    }

    /// Sets the feature flags.
    pub fn flags(&mut self, flags: u16) -> &mut Self {
        self.flags = flags;
        self
    }

    /// Sets the features flags that can be updated in future.
    pub fn mutable_flags(&mut self, mutable_flags: u16) -> &mut Self {
        self.mutable_flags = mutable_flags;
        self
    }

    /// Adds a permission configuration.
    pub fn permission(&mut self, badge_address: Address, authorities: u16) -> &mut Self {
        self.authorities.insert(badge_address, authorities);
        self
    }

    /// Creates resource with the given initial supply.
    pub fn initial_supply(&self, supply: NewSupply) -> Bucket {
        self.build(Some(supply)).1.unwrap()
    }

    /// Creates resource with no initial supply.
    pub fn no_initial_supply(&self) -> ResourceDef {
        self.build(None).0
    }

    fn build(&self, supply: Option<NewSupply>) -> (ResourceDef, Option<Bucket>) {
        ResourceDef::new(
            self.resource_type,
            self.metadata.clone(),
            self.granularity,
            self.flags,
            self.mutable_flags,
            self.authorities.clone(),
            supply,
        )
    }
}
