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
            flags: 0,
            mutable_flags: 0,
            authorities: HashMap::new(),
        }
    }

    /// Starts a new builder to create fungible resource, e.g., tokens.
    ///
    /// Fungible resource can have different granularity
    /// * If granularity is `0`, the smallest unit is `10^-18`;
    /// * If granularity is `1`, the smallest unit is `10^-17`;
    /// * So on and so forth.
    pub fn new_fungible(granularity: u8) -> Self {
        Self::new(ResourceType::Fungible { granularity })
    }

    /// Starts a new builder to create non-fungible resource, e.g. NFT.
    pub fn new_non_fungible() -> Self {
        Self::new(ResourceType::NonFungible)
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

    /// Adds a badge for authorization.
    pub fn badge<A: Into<ResourceDef>>(&mut self, badge_address: A, permissions: u16) -> &mut Self {
        self.authorities
            .insert(badge_address.into().address(), permissions);
        self
    }

    /// Creates resource with the given initial supply.
    pub fn initial_supply(&self, supply: NewSupply) -> Bucket {
        self.build(Some(supply)).1.unwrap()
    }

    /// Creates resource with the given initial fungible supply.
    pub fn initial_supply_fungible<T: Into<Decimal>>(&self, amount: T) -> Bucket {
        self.build(Some(NewSupply::fungible(amount))).1.unwrap()
    }

    /// Creates resource with the given initial non-fungible supply.
    ///
    /// # Example
    /// ```ignore
    /// let bucket = ResourceBuilder::new_non_fungible()
    ///     .metadata("name", "TestNft")
    ///     .initial_supply_non_fungible([
    ///         (1, "immutable_part", "mutable_part"),
    ///         (2, "another_immutable_part", "another_mutable_part"),
    ///     ]);
    /// ```
    pub fn initial_supply_non_fungible<T, V>(&self, entries: T) -> Bucket
    where
        T: IntoIterator<Item = (u128, V)>,
        V: NftData
    {
        self.build(Some(NewSupply::non_fungible(entries)))
            .1
            .unwrap()
    }

    /// Creates resource with no initial supply.
    pub fn no_initial_supply(&self) -> ResourceDef {
        self.build(None).0
    }

    fn build(&self, supply: Option<NewSupply>) -> (ResourceDef, Option<Bucket>) {
        ResourceDef::new(
            self.resource_type,
            self.metadata.clone(),
            self.flags,
            self.mutable_flags,
            self.authorities.clone(),
            supply,
        )
    }
}
