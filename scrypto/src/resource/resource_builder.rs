use crate::engine::scrypto_env::ScryptoEnv;
use crate::radix_engine_interface::api::api::Invokable;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::*;
use radix_engine_interface::rule;
use sbor::rust::borrow::ToOwned;
use sbor::rust::collections::HashMap;
use sbor::rust::string::String;

/// Not divisible.
pub const DIVISIBILITY_NONE: u8 = 0;
/// The maximum divisibility supported.
pub const DIVISIBILITY_MAXIMUM: u8 = 18;

/// Utility for setting up a new resource.
pub struct ResourceBuilder;

impl ResourceBuilder {
    /// Starts a new builder to create fungible resource.
    pub fn new_fungible() -> FungibleResourceBuilder {
        FungibleResourceBuilder::new()
    }

    /// Starts a new builder to create non-fungible resource.
    pub fn new_non_fungible(non_fungible_id_type: NonFungibleIdType) -> NonFungibleResourceBuilder {
        NonFungibleResourceBuilder::new(non_fungible_id_type)
    }
}

/// A resource builder which builds fungible resources that may or may not have an owner badge.
///
/// # Note
///
/// Once one of the methods that set behavior is called, a new [`FungibleResourceWithAuthBuilder`]
/// is created which commits the developer to building a resource that does not have an owner badge.
/// If none of these methods are called, then the developer has the choice to either building a
/// resource with an owner badge or without one.
pub struct FungibleResourceBuilder {
    divisibility: u8,
    metadata: HashMap<String, String>,
}

impl FungibleResourceBuilder {
    pub fn new() -> Self {
        Self {
            divisibility: DIVISIBILITY_MAXIMUM,
            metadata: HashMap::new(),
        }
    }

    /// Set the divisibility.
    ///
    /// `0` means the resource is not divisible; `18` is the max divisibility.
    pub fn divisibility(mut self, divisibility: u8) -> Self {
        assert!(divisibility <= 18);
        self.divisibility = divisibility;
        self
    }

    /// Adds a resource metadata.
    ///
    /// If a previous attribute with the same name has been set, it will be overwritten.
    pub fn metadata<K: AsRef<str>, V: AsRef<str>>(mut self, name: K, value: V) -> Self {
        self.metadata
            .insert(name.as_ref().to_owned(), value.as_ref().to_owned());
        self
    }

    pub fn mintable<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> FungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(Mint, (method_auth, mutability.into()));
        FungibleResourceWithAuthBuilder {
            divisibility: self.divisibility,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn burnable<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> FungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(Burn, (method_auth, mutability.into()));
        FungibleResourceWithAuthBuilder {
            divisibility: self.divisibility,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn recallable<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> FungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(Recall, (method_auth, mutability.into()));
        FungibleResourceWithAuthBuilder {
            divisibility: self.divisibility,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn restrict_withdraw<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> FungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(Withdraw, (method_auth, mutability.into()));
        FungibleResourceWithAuthBuilder {
            divisibility: self.divisibility,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn restrict_deposit<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> FungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(Deposit, (method_auth, mutability.into()));
        FungibleResourceWithAuthBuilder {
            divisibility: self.divisibility,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn updateable_metadata<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> FungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(UpdateMetadata, (method_auth, mutability.into()));
        FungibleResourceWithAuthBuilder {
            divisibility: self.divisibility,
            metadata: self.metadata,
            authorization,
        }
    }

    /// Creates resource with the given initial supply.
    ///
    /// # Example
    /// ```ignore
    /// let bucket = ResourceBuilder::new_fungible()
    ///     .metadata("name", "TestToken")
    ///     .initial_supply(5);
    /// ```
    pub fn initial_supply<T: Into<Decimal>>(self, amount: T) -> Bucket {
        let mut authorization = HashMap::new();
        authorization.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));

        let (_resource_address, bucket) = ScryptoEnv
            .invoke(ResourceManagerCreateInvocation {
                resource_type: ResourceType::Fungible {
                    divisibility: self.divisibility,
                },
                metadata: self.metadata,
                access_rules: authorization,
                mint_params: Some(MintParams::fungible(amount)),
            })
            .unwrap();

        bucket.unwrap()
    }

    pub fn no_initial_supply(self) -> ResourceAddress {
        let (resource_address, _bucket) = ScryptoEnv
            .invoke(ResourceManagerCreateInvocation {
                resource_type: ResourceType::Fungible {
                    divisibility: self.divisibility,
                },
                metadata: self.metadata,
                access_rules: HashMap::new(),
                mint_params: None,
            })
            .unwrap();

        resource_address
    }

    pub fn initial_supply_with_owner<T: Into<Decimal>>(
        self,
        amount: T,
        owner_badge: NonFungibleAddress,
    ) -> Bucket {
        let (_resource_address, bucket) = ScryptoEnv
            .invoke(ResourceManagerCreateWithOwnerInvocation {
                resource_type: ResourceType::Fungible {
                    divisibility: self.divisibility,
                },
                metadata: self.metadata,
                owner_badge: owner_badge,
                mint_params: Some(MintParams::fungible(amount)),
            })
            .unwrap();

        bucket.unwrap()
    }

    pub fn no_initial_supply_with_owner(self, owner_badge: NonFungibleAddress) -> ResourceAddress {
        let (resource_address, _bucket) = ScryptoEnv
            .invoke(ResourceManagerCreateWithOwnerInvocation {
                resource_type: ResourceType::Fungible {
                    divisibility: self.divisibility,
                },
                metadata: self.metadata,
                owner_badge: owner_badge,
                mint_params: None,
            })
            .unwrap();

        resource_address
    }
}

/// A resource builder which builds fungible resources that do not have an owner-badge where
/// resource behavior is completely handled by the developer.
///  
/// Typically this resource builder is created from the [`FungibleResourceBuilder`] once the
/// developer has called one of the methods that set resource behavior. This is done as a static
/// way of committing the developer to this choice by transitioning into a builder which does not
/// offer the `initial_supply_with_owner` and `no_initial_supply_with_owner` methods.
pub struct FungibleResourceWithAuthBuilder {
    divisibility: u8,
    metadata: HashMap<String, String>,
    authorization: HashMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
}

impl FungibleResourceWithAuthBuilder {
    /// Adds a resource metadata.
    ///
    /// If a previous attribute with the same name has been set, it will be overwritten.
    pub fn metadata<K: AsRef<str>, V: AsRef<str>>(mut self, name: K, value: V) -> Self {
        self.metadata
            .insert(name.as_ref().to_owned(), value.as_ref().to_owned());
        self
    }

    pub fn mintable<R: Into<AccessRule>>(mut self, method_auth: AccessRule, mutability: R) -> Self {
        self.authorization
            .insert(Mint, (method_auth, mutability.into()));
        self
    }

    pub fn burnable<R: Into<AccessRule>>(mut self, method_auth: AccessRule, mutability: R) -> Self {
        self.authorization
            .insert(Burn, (method_auth, mutability.into()));
        self
    }

    pub fn recallable<R: Into<AccessRule>>(
        mut self,
        method_auth: AccessRule,
        mutability: R,
    ) -> Self {
        self.authorization
            .insert(Recall, (method_auth, mutability.into()));
        self
    }

    pub fn restrict_withdraw<R: Into<AccessRule>>(
        mut self,
        method_auth: AccessRule,
        mutability: R,
    ) -> Self {
        self.authorization
            .insert(Withdraw, (method_auth, mutability.into()));
        self
    }

    pub fn restrict_deposit<R: Into<AccessRule>>(
        mut self,
        method_auth: AccessRule,
        mutability: R,
    ) -> Self {
        self.authorization
            .insert(Deposit, (method_auth, mutability.into()));
        self
    }

    pub fn updateable_metadata<R: Into<AccessRule>>(
        mut self,
        method_auth: AccessRule,
        mutability: R,
    ) -> Self {
        self.authorization
            .insert(UpdateMetadata, (method_auth, mutability.into()));
        self
    }

    pub fn initial_supply<T: Into<Decimal>>(self, amount: T) -> Bucket {
        self.build(Some(MintParams::fungible(amount))).1.unwrap()
    }

    /// Creates resource with no initial supply.
    pub fn no_initial_supply(self) -> ResourceAddress {
        self.build(None).0
    }

    fn build(mut self, mint_params: Option<MintParams>) -> (ResourceAddress, Option<Bucket>) {
        if !self.authorization.contains_key(&Withdraw) {
            self.authorization
                .insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        }

        ScryptoEnv
            .invoke(ResourceManagerCreateInvocation {
                resource_type: ResourceType::Fungible {
                    divisibility: self.divisibility,
                },
                metadata: self.metadata,
                access_rules: self.authorization,
                mint_params,
            })
            .unwrap()
    }
}

/// A resource builder which builds non-fungible resources that may or may not have an owner badge.
///
/// # Note
///
/// Once one of the methods that set behavior is called, a new [`NonFungibleResourceWithAuthBuilder`]
/// is created which commits the developer to building a resource that does not have an owner badge.
/// If none of these methods are called, then the developer has the choice to either building a
/// resource with an owner badge or without one.
pub struct NonFungibleResourceBuilder {
    metadata: HashMap<String, String>,
    id_type: NonFungibleIdType,
}

impl NonFungibleResourceBuilder {
    pub fn new(id_type: NonFungibleIdType) -> Self {
        Self {
            metadata: HashMap::new(),
            id_type,
        }
    }

    /// Adds a resource metadata.
    ///
    /// If a previous attribute with the same name has been set, it will be overwritten.
    pub fn metadata<K: AsRef<str>, V: AsRef<str>>(mut self, name: K, value: V) -> Self {
        self.metadata
            .insert(name.as_ref().to_owned(), value.as_ref().to_owned());
        self
    }

    pub fn mintable<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> NonFungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(Mint, (method_auth, mutability.into()));
        NonFungibleResourceWithAuthBuilder {
            id_type: self.id_type,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn burnable<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> NonFungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(Burn, (method_auth, mutability.into()));
        NonFungibleResourceWithAuthBuilder {
            id_type: self.id_type,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn recallable<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> NonFungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(Recall, (method_auth, mutability.into()));
        NonFungibleResourceWithAuthBuilder {
            id_type: self.id_type,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn restrict_withdraw<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> NonFungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(Withdraw, (method_auth, mutability.into()));
        NonFungibleResourceWithAuthBuilder {
            id_type: self.id_type,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn restrict_deposit<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> NonFungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(Deposit, (method_auth, mutability.into()));
        NonFungibleResourceWithAuthBuilder {
            id_type: self.id_type,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn updateable_metadata<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> NonFungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(UpdateMetadata, (method_auth, mutability.into()));
        NonFungibleResourceWithAuthBuilder {
            id_type: self.id_type,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn updateable_non_fungible_data<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> NonFungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(UpdateNonFungibleData, (method_auth, mutability.into()));
        NonFungibleResourceWithAuthBuilder {
            id_type: self.id_type,
            metadata: self.metadata,
            authorization,
        }
    }

    /// Creates resource with the given initial supply.
    ///
    /// # Example
    /// ```ignore
    /// let bucket = ResourceBuilder::new_non_fungible(NonFungibleIdType::UUID)
    ///     .metadata("name", "TestNonFungible")
    ///     .initial_supply([
    ///         (NonFungibleId::from(1u128), "immutable_part", "mutable_part"),
    ///         (NonFungibleId::from(2u128), "another_immutable_part", "another_mutable_part"),
    ///     ]);
    /// ```
    pub fn initial_supply<T, V>(self, entries: T) -> Bucket
    where
        T: IntoIterator<Item = (NonFungibleId, V)>,
        V: NonFungibleData,
    {
        let mut encoded = HashMap::new();
        for (id, e) in entries {
            encoded.insert(id, (e.immutable_data().unwrap(), e.mutable_data().unwrap()));
        }
        self.build(Some(MintParams::NonFungible { entries: encoded }))
            .1
            .unwrap()
    }

    /// Creates resource with no initial supply.
    pub fn no_initial_supply(self) -> ResourceAddress {
        self.build(None).0
    }

    fn build(self, mint_params: Option<MintParams>) -> (ResourceAddress, Option<Bucket>) {
        let mut authorization = HashMap::new();
        authorization.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));

        ScryptoEnv
            .invoke(ResourceManagerCreateInvocation {
                resource_type: ResourceType::NonFungible {
                    id_type: self.id_type,
                },
                metadata: self.metadata,
                access_rules: authorization,
                mint_params,
            })
            .unwrap()
    }

    pub fn initial_supply_with_owner<T, V>(
        self,
        entries: T,
        owner_badge: NonFungibleAddress,
    ) -> Bucket
    where
        T: IntoIterator<Item = (NonFungibleId, V)>,
        V: NonFungibleData,
    {
        let mut encoded = HashMap::new();
        for (id, e) in entries {
            encoded.insert(id, (e.immutable_data().unwrap(), e.mutable_data().unwrap()));
        }

        let (_resource_address, bucket) = ScryptoEnv
            .invoke(ResourceManagerCreateWithOwnerInvocation {
                resource_type: ResourceType::NonFungible {
                    id_type: self.id_type,
                },
                metadata: self.metadata,
                owner_badge: owner_badge,
                mint_params: Some(MintParams::NonFungible { entries: encoded }),
            })
            .unwrap();

        bucket.unwrap()
    }

    pub fn no_initial_supply_with_owner(self, owner_badge: NonFungibleAddress) -> ResourceAddress {
        let (resource_address, _bucket) = ScryptoEnv
            .invoke(ResourceManagerCreateWithOwnerInvocation {
                resource_type: ResourceType::NonFungible {
                    id_type: self.id_type,
                },
                metadata: self.metadata,
                owner_badge: owner_badge,
                mint_params: None,
            })
            .unwrap();

        resource_address
    }
}

/// A resource builder which builds non-fungible resources that do not have an owner-badge where
/// resource behavior is completely handled by the developer.
///  
/// Typically this resource builder is created from the [`NonFungibleResourceBuilder`] once the
/// developer has called one of the methods that set resource behavior. This is done as a static
/// way of committing the developer to this choice by transitioning into a builder which does not
/// offer the `initial_supply_with_owner` and `no_initial_supply_with_owner` methods.
pub struct NonFungibleResourceWithAuthBuilder {
    id_type: NonFungibleIdType,
    metadata: HashMap<String, String>,
    authorization: HashMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
}

impl NonFungibleResourceWithAuthBuilder {
    /// Adds a resource metadata.
    ///
    /// If a previous attribute with the same name has been set, it will be overwritten.
    pub fn metadata<K: AsRef<str>, V: AsRef<str>>(mut self, name: K, value: V) -> Self {
        self.metadata
            .insert(name.as_ref().to_owned(), value.as_ref().to_owned());
        self
    }

    pub fn mintable<R: Into<AccessRule>>(mut self, method_auth: AccessRule, mutability: R) -> Self {
        self.authorization
            .insert(Mint, (method_auth, mutability.into()));
        self
    }

    pub fn burnable<R: Into<AccessRule>>(mut self, method_auth: AccessRule, mutability: R) -> Self {
        self.authorization
            .insert(Burn, (method_auth, mutability.into()));
        self
    }

    pub fn recallable<R: Into<AccessRule>>(
        mut self,
        method_auth: AccessRule,
        mutability: R,
    ) -> Self {
        self.authorization
            .insert(Recall, (method_auth, mutability.into()));
        self
    }

    pub fn restrict_withdraw<R: Into<AccessRule>>(
        mut self,
        method_auth: AccessRule,
        mutability: R,
    ) -> Self {
        self.authorization
            .insert(Withdraw, (method_auth, mutability.into()));
        self
    }

    pub fn restrict_deposit<R: Into<AccessRule>>(
        mut self,
        method_auth: AccessRule,
        mutability: R,
    ) -> Self {
        self.authorization
            .insert(Deposit, (method_auth, mutability.into()));
        self
    }

    pub fn updateable_metadata<R: Into<AccessRule>>(
        mut self,
        method_auth: AccessRule,
        mutability: R,
    ) -> Self {
        self.authorization
            .insert(UpdateMetadata, (method_auth, mutability.into()));
        self
    }

    pub fn updateable_non_fungible_data<R: Into<AccessRule>>(
        mut self,
        method_auth: AccessRule,
        mutability: R,
    ) -> Self {
        self.authorization
            .insert(UpdateNonFungibleData, (method_auth, mutability.into()));
        self
    }

    pub fn initial_supply<T, V>(self, entries: T) -> Bucket
    where
        T: IntoIterator<Item = (NonFungibleId, V)>,
        V: NonFungibleData,
    {
        let mut encoded = HashMap::new();
        for (id, e) in entries {
            encoded.insert(id, (e.immutable_data().unwrap(), e.mutable_data().unwrap()));
        }
        self.build(Some(MintParams::NonFungible { entries: encoded }))
            .1
            .unwrap()
    }

    /// Creates resource with no initial supply.
    pub fn no_initial_supply(self) -> ResourceAddress {
        self.build(None).0
    }

    fn build(mut self, mint_params: Option<MintParams>) -> (ResourceAddress, Option<Bucket>) {
        if !self.authorization.contains_key(&Withdraw) {
            self.authorization
                .insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        }

        ScryptoEnv
            .invoke(ResourceManagerCreateInvocation {
                resource_type: ResourceType::NonFungible {
                    id_type: self.id_type,
                },
                metadata: self.metadata,
                access_rules: self.authorization,
                mint_params,
            })
            .unwrap()
    }
}
