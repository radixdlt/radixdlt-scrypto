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
    pub fn new_non_fungible() -> NonFungibleResourceBuilder {
        NonFungibleResourceBuilder::new()
    }
}

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

    pub fn mintable(
        self,
        method_auth: AccessRule,
        mutability: Mutability,
    ) -> FungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(Mint, (method_auth, AccessRule::from(mutability)));
        FungibleResourceWithAuthBuilder {
            divisibility: self.divisibility,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn burnable(
        self,
        method_auth: AccessRule,
        mutability: Mutability,
    ) -> FungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(Burn, (method_auth, AccessRule::from(mutability)));
        FungibleResourceWithAuthBuilder {
            divisibility: self.divisibility,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn recallable(
        self,
        method_auth: AccessRule,
        mutability: Mutability,
    ) -> FungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(Recall, (method_auth, AccessRule::from(mutability)));
        FungibleResourceWithAuthBuilder {
            divisibility: self.divisibility,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn restrict_withdraw(
        self,
        method_auth: AccessRule,
        mutability: Mutability,
    ) -> FungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(Withdraw, (method_auth, AccessRule::from(mutability)));
        FungibleResourceWithAuthBuilder {
            divisibility: self.divisibility,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn restrict_deposit(
        self,
        method_auth: AccessRule,
        mutability: Mutability,
    ) -> FungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(Deposit, (method_auth, AccessRule::from(mutability)));
        FungibleResourceWithAuthBuilder {
            divisibility: self.divisibility,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn updateable_metadata(
        self,
        method_auth: AccessRule,
        mutability: Mutability,
    ) -> FungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(UpdateMetadata, (method_auth, AccessRule::from(mutability)));
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

    pub fn mintable(mut self, method_auth: AccessRule, mutability: Mutability) -> Self {
        self.authorization
            .insert(Mint, (method_auth, AccessRule::from(mutability)));
        self
    }

    pub fn burnable(mut self, method_auth: AccessRule, mutability: Mutability) -> Self {
        self.authorization
            .insert(Burn, (method_auth, AccessRule::from(mutability)));
        self
    }

    pub fn recallable(mut self, method_auth: AccessRule, mutability: Mutability) -> Self {
        self.authorization
            .insert(Recall, (method_auth, AccessRule::from(mutability)));
        self
    }

    pub fn restrict_withdraw(mut self, method_auth: AccessRule, mutability: Mutability) -> Self {
        self.authorization
            .insert(Withdraw, (method_auth, AccessRule::from(mutability)));
        self
    }

    pub fn restrict_deposit(mut self, method_auth: AccessRule, mutability: Mutability) -> Self {
        self.authorization
            .insert(Deposit, (method_auth, AccessRule::from(mutability)));
        self
    }

    pub fn updateable_metadata(mut self, method_auth: AccessRule, mutability: Mutability) -> Self {
        self.authorization
            .insert(UpdateMetadata, (method_auth, AccessRule::from(mutability)));
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

pub struct NonFungibleResourceBuilder {
    metadata: HashMap<String, String>,
    id_type: NonFungibleIdType,
}

impl NonFungibleResourceBuilder {
    pub fn new() -> Self {
        Self {
            metadata: HashMap::new(),
            id_type: NonFungibleIdType::default(),
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

    pub fn mintable(
        self,
        method_auth: AccessRule,
        mutability: Mutability,
    ) -> NonFungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(Mint, (method_auth, AccessRule::from(mutability)));
        NonFungibleResourceWithAuthBuilder {
            id_type: self.id_type,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn burnable(
        self,
        method_auth: AccessRule,
        mutability: Mutability,
    ) -> NonFungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(Burn, (method_auth, AccessRule::from(mutability)));
        NonFungibleResourceWithAuthBuilder {
            id_type: self.id_type,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn recallable(
        self,
        method_auth: AccessRule,
        mutability: Mutability,
    ) -> NonFungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(Recall, (method_auth, AccessRule::from(mutability)));
        NonFungibleResourceWithAuthBuilder {
            id_type: self.id_type,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn restrict_withdraw(
        self,
        method_auth: AccessRule,
        mutability: Mutability,
    ) -> NonFungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(Withdraw, (method_auth, AccessRule::from(mutability)));
        NonFungibleResourceWithAuthBuilder {
            id_type: self.id_type,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn restrict_deposit(
        self,
        method_auth: AccessRule,
        mutability: Mutability,
    ) -> NonFungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(Deposit, (method_auth, AccessRule::from(mutability)));
        NonFungibleResourceWithAuthBuilder {
            id_type: self.id_type,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn updateable_metadata(
        self,
        method_auth: AccessRule,
        mutability: Mutability,
    ) -> NonFungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(UpdateMetadata, (method_auth, AccessRule::from(mutability)));
        NonFungibleResourceWithAuthBuilder {
            id_type: self.id_type,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn updateable_non_fungible_data(
        self,
        method_auth: AccessRule,
        mutability: Mutability,
    ) -> NonFungibleResourceWithAuthBuilder {
        let mut authorization = HashMap::new();
        authorization.insert(
            UpdateNonFungibleData,
            (method_auth, AccessRule::from(mutability)),
        );
        NonFungibleResourceWithAuthBuilder {
            id_type: self.id_type,
            metadata: self.metadata,
            authorization,
        }
    }

    /// Set ID type to use for this non fungible resource
    pub fn id_type(mut self, id_type: NonFungibleIdType) -> Self {
        self.id_type = id_type;
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

    pub fn mintable(mut self, method_auth: AccessRule, mutability: Mutability) -> Self {
        self.authorization
            .insert(Mint, (method_auth, AccessRule::from(mutability)));
        self
    }

    pub fn burnable(mut self, method_auth: AccessRule, mutability: Mutability) -> Self {
        self.authorization
            .insert(Burn, (method_auth, AccessRule::from(mutability)));
        self
    }

    pub fn recallable(mut self, method_auth: AccessRule, mutability: Mutability) -> Self {
        self.authorization
            .insert(Recall, (method_auth, AccessRule::from(mutability)));
        self
    }

    pub fn restrict_withdraw(mut self, method_auth: AccessRule, mutability: Mutability) -> Self {
        self.authorization
            .insert(Withdraw, (method_auth, AccessRule::from(mutability)));
        self
    }

    pub fn restrict_deposit(mut self, method_auth: AccessRule, mutability: Mutability) -> Self {
        self.authorization
            .insert(Deposit, (method_auth, AccessRule::from(mutability)));
        self
    }

    pub fn updateable_metadata(mut self, method_auth: AccessRule, mutability: Mutability) -> Self {
        self.authorization
            .insert(UpdateMetadata, (method_auth, AccessRule::from(mutability)));
        self
    }

    pub fn updateable_non_fungible_data(
        mut self,
        method_auth: AccessRule,
        mutability: Mutability,
    ) -> Self {
        self.authorization.insert(
            UpdateNonFungibleData,
            (method_auth, AccessRule::from(mutability)),
        );
        self
    }

    pub fn id_type(mut self, id_type: NonFungibleIdType) -> Self {
        self.id_type = id_type;
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
