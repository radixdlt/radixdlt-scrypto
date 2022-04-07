use crate::math::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::collections::HashMap;
use crate::rust::string::String;

/// Not divisible.
pub const DIVISIBILITY_NONE: u8 = 0;
/// The maximum divisibility supported.
pub const DIVISIBILITY_MAXIMUM: u8 = 18;

/// Utility for setting up a new resource.
pub struct ResourceBuilder;

pub struct FungibleResourceBuilder {
    divisibility: u8,
    metadata: HashMap<String, String>,
    authorization: HashMap<ResourceMethod, MethodAuth>,
}

pub struct NonFungibleResourceBuilder {
    metadata: HashMap<String, String>,
    authorization: HashMap<ResourceMethod, MethodAuth>,
}

impl ResourceBuilder {
    /// Starts a new builder to create fungible resource.
    pub fn new_fungible() -> FungibleResourceBuilder {
        FungibleResourceBuilder::new()
    }

    /// Starts a new builder to create non-fungible resource.
    pub fn new_non_fungible() -> NonFungibleResourceBuilder {
        NonFungibleResourceBuilder::new()
    }
}

impl FungibleResourceBuilder {
    pub fn new() -> Self {
        Self {
            divisibility: DIVISIBILITY_MAXIMUM,
            metadata: HashMap::new(),
            authorization: HashMap::new(),
        }
    }

    /// Set the divisibility.
    ///
    /// `0` means the resource is not divisible; `18` is the max divisibility.
    pub fn divisibility(&mut self, divisibility: u8) -> &mut Self {
        assert!(divisibility <= 18);
        self.divisibility = divisibility;
        self
    }

    /// Adds a resource metadata.
    ///
    /// If a previous attribute with the same name has been set, it will be overwritten.
    pub fn metadata<K: AsRef<str>, V: AsRef<str>>(&mut self, name: K, value: V) -> &mut Self {
        self.metadata
            .insert(name.as_ref().to_owned(), value.as_ref().to_owned());
        self
    }

    pub fn auth(&mut self, method: ResourceMethod, method_auth: MethodAuth) -> &mut Self {
        self.authorization.insert(method, method_auth);
        self
    }

    /// Creates resource with the given initial supply.
    ///
    /// # Example
    /// ```ignore
    /// let bucket = ResourceBuilder::new_fungible()
    ///     .metadata("name", "TestToken")
    ///     .initial_supply(5);
    /// ```
    pub fn initial_supply<T: Into<Decimal>>(&self, amount: T) -> Bucket {
        self.build(Some(MintParams::fungible(amount))).1.unwrap()
    }

    /// Creates resource with no initial supply.
    pub fn no_initial_supply(&self) -> ResourceAddress {
        self.build(None).0
    }

    fn build(&self, mint_params: Option<MintParams>) -> (ResourceAddress, Option<Bucket>) {
        resource_system().new_resource(
            ResourceType::Fungible {
                divisibility: self.divisibility,
            },
            self.metadata.clone(),
            self.authorization.clone(),
            mint_params,
        )
    }
}

impl NonFungibleResourceBuilder {
    pub fn new() -> Self {
        Self {
            metadata: HashMap::new(),
            authorization: HashMap::new(),
        }
    }

    /// Adds a resource metadata.
    ///
    /// If a previous attribute with the same name has been set, it will be overwritten.
    pub fn metadata<K: AsRef<str>, V: AsRef<str>>(&mut self, name: K, value: V) -> &mut Self {
        self.metadata
            .insert(name.as_ref().to_owned(), value.as_ref().to_owned());
        self
    }

    pub fn auth(&mut self, method: ResourceMethod, method_auth: MethodAuth) -> &mut Self {
        self.authorization.insert(method, method_auth);
        self
    }

    /// Creates resource with the given initial supply.
    ///
    /// # Example
    /// ```ignore
    /// let bucket = ResourceBuilder::new_non_fungible()
    ///     .metadata("name", "TestNonFungible")
    ///     .initial_supply([
    ///         (NftKey::from(1u128), "immutable_part", "mutable_part"),
    ///         (NftKey::from(2u128), "another_immutable_part", "another_mutable_part"),
    ///     ]);
    /// ```
    pub fn initial_supply<T, V>(&self, entries: T) -> Bucket
    where
        T: IntoIterator<Item = (NonFungibleId, V)>,
        V: NonFungibleData,
    {
        self.build(Some(MintParams::non_fungible(entries)))
            .1
            .unwrap()
    }

    /// Creates resource with no initial supply.
    pub fn no_initial_supply(&self) -> ResourceAddress {
        self.build(None).0
    }

    fn build(&self, mint_params: Option<MintParams>) -> (ResourceAddress, Option<Bucket>) {
        resource_system().new_resource(
            ResourceType::NonFungible,
            self.metadata.clone(),
            self.authorization.clone(),
            mint_params,
        )
    }
}
