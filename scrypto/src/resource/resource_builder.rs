use crate::math::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::collections::HashMap;
use crate::rust::string::String;

/// Not divisible.
pub const DIVISIBILITY_NONE: u8 = 0;
/// The maximum divisibility supported.
pub const DIVISIBILITY_MAXIMUM: u8 = 18;

/// Utility for creating resources.
pub struct ResourceBuilder {
    resource_type: ResourceType,
    metadata: HashMap<String, String>,
    flags: u64,
    mutable_flags: u64,
    authorities: HashMap<ResourceDefId, u64>,
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

    /// Starts a new builder to create fungible resource.
    ///
    /// # Arguments
    /// * `divisibility` - The divisibility of the resource; `0` means not divisible, and `18` is the allowed max divisibility.
    pub fn new_fungible(divisibility: u8) -> Self {
        Self::new(ResourceType::Fungible { divisibility })
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
    pub fn flags(&mut self, flags: u64) -> &mut Self {
        self.flags = flags;
        self
    }

    /// Sets the features flags that can be updated in future.
    pub fn mutable_flags(&mut self, mutable_flags: u64) -> &mut Self {
        self.mutable_flags = mutable_flags;
        self
    }

    /// Adds a badge for authorization.
    pub fn badge(&mut self, badge: ResourceDefId, permissions: u64) -> &mut Self {
        self.authorities.insert(badge, permissions);
        self
    }

    /// Creates resource with the given initial supply.
    pub fn initial_supply(&self, supply: Supply) -> Bucket {
        self.build(Some(supply)).1.unwrap()
    }

    /// Creates resource with the given initial fungible supply.
    ///
    /// # Example
    /// ```ignore
    /// let bucket = ResourceBuilder::new_fungible()
    ///     .metadata("name", "TestToken")
    ///     .initial_supply_fungible(5);
    /// ```
    pub fn initial_supply_fungible<T: Into<Decimal>>(&self, amount: T) -> Bucket {
        self.build(Some(Supply::fungible(amount))).1.unwrap()
    }

    /// Creates resource with the given initial non-fungible supply.
    ///
    /// # Example
    /// ```ignore
    /// let bucket = ResourceBuilder::new_non_fungible()
    ///     .metadata("name", "TestNonFungible")
    ///     .initial_supply_non_fungible([
    ///         (NftKey::from(1u128), "immutable_part", "mutable_part"),
    ///         (NftKey::from(2u128), "another_immutable_part", "another_mutable_part"),
    ///     ]);
    /// ```
    pub fn initial_supply_non_fungible<T, V>(&self, entries: T) -> Bucket
    where
        T: IntoIterator<Item = (NonFungibleId, V)>,
        V: NonFungibleData,
    {
        self.build(Some(Supply::non_fungible(entries))).1.unwrap()
    }

    /// Creates resource with no initial supply.
    pub fn no_initial_supply(&self) -> ResourceDefId {
        self.build(None).0
    }

    fn build(&self, supply: Option<Supply>) -> (ResourceDefId, Option<Bucket>) {
        resource_system().instantiate_resource_definition(
            self.resource_type,
            self.metadata.clone(),
            self.flags,
            self.mutable_flags,
            self.authorities.clone(),
            supply,
        )
    }
}
