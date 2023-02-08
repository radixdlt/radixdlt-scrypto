use crate::engine::scrypto_env::ScryptoEnv;
use crate::radix_engine_interface::api::Invokable;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::constants::{RESOURCE_MANAGER_BLUEPRINT, RESOURCE_MANAGER_PACKAGE};
use radix_engine_interface::data::{scrypto_decode, scrypto_encode};
use radix_engine_interface::math::Decimal;
use sbor::rust::borrow::ToOwned;
use sbor::rust::collections::{BTreeMap, BTreeSet};
use sbor::rust::marker::PhantomData;
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
    pub fn new_non_fungible<Y: IsNonFungibleLocalId>() -> NonFungibleResourceBuilder<Y> {
        NonFungibleResourceBuilder::new()
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
    metadata: BTreeMap<String, String>,
}

impl FungibleResourceBuilder {
    pub fn new() -> Self {
        Self {
            divisibility: DIVISIBILITY_MAXIMUM,
            metadata: BTreeMap::new(),
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
        let mut authorization = BTreeMap::new();
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
        let mut authorization = BTreeMap::new();
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
        let mut authorization = BTreeMap::new();
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
        let mut authorization = BTreeMap::new();
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
        let mut authorization = BTreeMap::new();
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
        let mut authorization = BTreeMap::new();
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
        let rtn = ScryptoEnv.call_function(
            RESOURCE_MANAGER_PACKAGE,
            RESOURCE_MANAGER_BLUEPRINT,
            "create_fungible_with_initial_supply",
            scrypto_encode(&ResourceManagerCreateFungibleWithInitialSupplyInvocation {
                resource_address: None,
                divisibility: self.divisibility,
                metadata: self.metadata,
                access_rules: BTreeMap::new(),
                initial_supply: amount.into(),
            }).unwrap()
        ).unwrap();
        let (_resource_address, bucket): (ResourceAddress, Bucket) = scrypto_decode(&rtn).unwrap();
        bucket
    }

    pub fn no_initial_supply(self) -> ResourceAddress {
        let rtn = ScryptoEnv.call_function(
            RESOURCE_MANAGER_PACKAGE,
            RESOURCE_MANAGER_BLUEPRINT,
            "create_fungible",
            scrypto_encode(&ResourceManagerCreateFungibleInvocation {
                divisibility: self.divisibility,
                metadata: self.metadata,
                access_rules: BTreeMap::new(),
            }).unwrap()
        ).unwrap();

        scrypto_decode(&rtn).unwrap()
    }

    pub fn initial_supply_with_owner<T: Into<Decimal>>(
        self,
        amount: T,
        owner_badge: NonFungibleGlobalId,
    ) -> Bucket {
        let rtn = ScryptoEnv.call_function(
            RESOURCE_MANAGER_PACKAGE,
            RESOURCE_MANAGER_BLUEPRINT,
            "create_fungible_with_initial_supply",
            scrypto_encode(&ResourceManagerCreateFungibleWithInitialSupplyInvocation {
                resource_address: None,
                divisibility: self.divisibility,
                metadata: self.metadata,
                access_rules: resource_access_rules_from_owner_badge(&owner_badge),
                initial_supply: amount.into(),
            }).unwrap()
        ).unwrap();
        let (_resource_address, bucket): (ResourceAddress, Bucket) = scrypto_decode(&rtn).unwrap();
        bucket
    }

    pub fn no_initial_supply_with_owner(self, owner_badge: NonFungibleGlobalId) -> ResourceAddress {
        let rtn = ScryptoEnv.call_function(
            RESOURCE_MANAGER_PACKAGE,
            RESOURCE_MANAGER_BLUEPRINT,
            "create_fungible",
            scrypto_encode(&ResourceManagerCreateFungibleInvocation {
                divisibility: self.divisibility,
                metadata: self.metadata,
                access_rules: resource_access_rules_from_owner_badge(&owner_badge),
            }).unwrap()
        ).unwrap();

        scrypto_decode(&rtn).unwrap()
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
    metadata: BTreeMap<String, String>,
    authorization: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
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
        let rtn = ScryptoEnv.call_function(
            RESOURCE_MANAGER_PACKAGE,
            RESOURCE_MANAGER_BLUEPRINT,
            "create_fungible_with_initial_supply",
            scrypto_encode(&ResourceManagerCreateFungibleWithInitialSupplyInvocation {
                resource_address: None,
                divisibility: self.divisibility,
                metadata: self.metadata,
                access_rules: self.authorization,
                initial_supply: amount.into(),
            }).unwrap()
        ).unwrap();
        let (_resource_address, bucket): (ResourceAddress, Bucket) = scrypto_decode(&rtn).unwrap();
        bucket
    }

    /// Creates resource with no initial supply.
    pub fn no_initial_supply(self) -> ResourceAddress {
        let rtn = ScryptoEnv.call_function(
            RESOURCE_MANAGER_PACKAGE,
            RESOURCE_MANAGER_BLUEPRINT,
            "create_fungible",
            scrypto_encode(&ResourceManagerCreateFungibleInvocation {
                divisibility: self.divisibility,
                metadata: self.metadata,
                access_rules: self.authorization,
            }).unwrap()
        ).unwrap();

        scrypto_decode(&rtn).unwrap()
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
pub struct NonFungibleResourceBuilder<Y: IsNonFungibleLocalId> {
    metadata: BTreeMap<String, String>,
    id_type: PhantomData<Y>,
}

impl<Y: IsNonFungibleLocalId> NonFungibleResourceBuilder<Y> {
    pub fn new() -> Self {
        Self {
            metadata: BTreeMap::new(),
            id_type: PhantomData,
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
    ) -> NonFungibleResourceWithAuthBuilder<Y> {
        let mut authorization = BTreeMap::new();
        authorization.insert(Mint, (method_auth, mutability.into()));
        NonFungibleResourceWithAuthBuilder {
            id_type: PhantomData,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn burnable<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> NonFungibleResourceWithAuthBuilder<Y> {
        let mut authorization = BTreeMap::new();
        authorization.insert(Burn, (method_auth, mutability.into()));
        NonFungibleResourceWithAuthBuilder {
            id_type: PhantomData,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn recallable<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> NonFungibleResourceWithAuthBuilder<Y> {
        let mut authorization = BTreeMap::new();
        authorization.insert(Recall, (method_auth, mutability.into()));
        NonFungibleResourceWithAuthBuilder {
            id_type: PhantomData,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn restrict_withdraw<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> NonFungibleResourceWithAuthBuilder<Y> {
        let mut authorization = BTreeMap::new();
        authorization.insert(Withdraw, (method_auth, mutability.into()));
        NonFungibleResourceWithAuthBuilder {
            id_type: PhantomData,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn restrict_deposit<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> NonFungibleResourceWithAuthBuilder<Y> {
        let mut authorization = BTreeMap::new();
        authorization.insert(Deposit, (method_auth, mutability.into()));
        NonFungibleResourceWithAuthBuilder {
            id_type: PhantomData,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn updateable_metadata<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> NonFungibleResourceWithAuthBuilder<Y> {
        let mut authorization = BTreeMap::new();
        authorization.insert(UpdateMetadata, (method_auth, mutability.into()));
        NonFungibleResourceWithAuthBuilder {
            id_type: PhantomData,
            metadata: self.metadata,
            authorization,
        }
    }

    pub fn updateable_non_fungible_data<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> NonFungibleResourceWithAuthBuilder<Y> {
        let mut authorization = BTreeMap::new();
        authorization.insert(UpdateNonFungibleData, (method_auth, mutability.into()));
        NonFungibleResourceWithAuthBuilder {
            id_type: PhantomData,
            metadata: self.metadata,
            authorization,
        }
    }

    /// Creates resource with no initial supply.
    pub fn no_initial_supply(self) -> ResourceAddress {
        let rtn = ScryptoEnv.call_function(
            RESOURCE_MANAGER_PACKAGE,
            RESOURCE_MANAGER_BLUEPRINT,
            "create_non_fungible",
            scrypto_encode(&ResourceManagerCreateNonFungibleInvocation {
                resource_address: None,
                id_type: Y::id_type(),
                metadata: self.metadata,
                access_rules: BTreeMap::new(),
            }).unwrap()
        ).unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    pub fn no_initial_supply_with_owner(self, owner_badge: NonFungibleGlobalId) -> ResourceAddress {
        let rtn = ScryptoEnv.call_function(
            RESOURCE_MANAGER_PACKAGE,
            RESOURCE_MANAGER_BLUEPRINT,
            "create_non_fungible",
            scrypto_encode(&ResourceManagerCreateNonFungibleInvocation {
                resource_address: None,
                id_type: Y::id_type(),
                metadata: self.metadata,
                access_rules: resource_access_rules_from_owner_badge(&owner_badge),
            }).unwrap()
        ).unwrap();
        scrypto_decode(&rtn).unwrap()
    }
}

impl<Y: IsNonAutoGeneratedNonFungibleLocalId> NonFungibleResourceBuilder<Y> {
    /// Creates resource with the given initial supply.
    ///
    /// # Example
    /// ```ignore
    /// let bucket = ResourceBuilder::new_non_fungible::<u32>()
    ///     .metadata("name", "TestNonFungible")
    ///     .initial_supply([
    ///         (1u32, "immutable_part", "mutable_part"),
    ///         (2u32, "another_immutable_part", "another_mutable_part"),
    ///     ]);
    /// ```
    pub fn initial_supply<T, V>(self, entries: T) -> Bucket
    where
        T: IntoIterator<Item = (Y, V)>,
        V: NonFungibleData,
    {
        let mut encoded = BTreeMap::new();
        for (id, e) in entries {
            encoded.insert(
                id.into(),
                (e.immutable_data().unwrap(), e.mutable_data().unwrap()),
            );
        }

        let rtn = ScryptoEnv.call_function(
            RESOURCE_MANAGER_PACKAGE,
            RESOURCE_MANAGER_BLUEPRINT,
            "create_non_fungible_with_initial_supply",
            scrypto_encode(&ResourceManagerCreateNonFungibleWithInitialSupplyInvocation {
                id_type: Y::id_type(),
                metadata: self.metadata,
                access_rules: BTreeMap::new(),
                entries: encoded,
            }).unwrap()
        ).unwrap();
        let (_resource_address, bucket): (ResourceAddress, Bucket) = scrypto_decode(&rtn).unwrap();
        bucket
    }

    pub fn initial_supply_with_owner<T, V>(
        self,
        entries: T,
        owner_badge: NonFungibleGlobalId,
    ) -> Bucket
    where
        T: IntoIterator<Item = (Y, V)>,
        V: NonFungibleData,
    {
        let mut encoded = BTreeMap::new();
        for (id, e) in entries {
            encoded.insert(
                id.into(),
                (e.immutable_data().unwrap(), e.mutable_data().unwrap()),
            );
        }

        let rtn = ScryptoEnv.call_function(
            RESOURCE_MANAGER_PACKAGE,
            RESOURCE_MANAGER_BLUEPRINT,
            "create_non_fungible_with_initial_supply",
            scrypto_encode(&ResourceManagerCreateNonFungibleWithInitialSupplyInvocation {
                id_type: Y::id_type(),
                metadata: self.metadata,
                access_rules: resource_access_rules_from_owner_badge(&owner_badge),
                entries: encoded,
            }).unwrap()
        ).unwrap();
        let (_resource_address, bucket): (ResourceAddress, Bucket) = scrypto_decode(&rtn).unwrap();
        bucket
    }
}

impl NonFungibleResourceBuilder<u128> {
    pub fn initial_supply_uuid<T, V>(self, entries: T) -> Bucket
    where
        T: IntoIterator<Item = V>,
        V: NonFungibleData,
    {
        let mut encoded = BTreeSet::new();
        for e in entries {
            encoded.insert((e.immutable_data().unwrap(), e.mutable_data().unwrap()));
        }

        let rtn = ScryptoEnv.call_function(
            RESOURCE_MANAGER_PACKAGE,
            RESOURCE_MANAGER_BLUEPRINT,
            "create_uuid_non_fungible_with_initial_supply",
            scrypto_encode(&ResourceManagerCreateUuidNonFungibleWithInitialSupplyInvocation {
                metadata: self.metadata,
                access_rules: BTreeMap::new(),
                entries: encoded,
            }).unwrap()
        ).unwrap();

        let (_resource_address, bucket): (ResourceAddress, Bucket) = scrypto_decode(&rtn).unwrap();
        bucket
    }
}

/// A resource builder which builds non-fungible resources that do not have an owner-badge where
/// resource behavior is completely handled by the developer.
///  
/// Typically this resource builder is created from the [`NonFungibleResourceBuilder`] once the
/// developer has called one of the methods that set resource behavior. This is done as a static
/// way of committing the developer to this choice by transitioning into a builder which does not
/// offer the `initial_supply_with_owner` and `no_initial_supply_with_owner` methods.
pub struct NonFungibleResourceWithAuthBuilder<Y: IsNonFungibleLocalId> {
    metadata: BTreeMap<String, String>,
    authorization: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    id_type: PhantomData<Y>,
}

impl<Y: IsNonFungibleLocalId> NonFungibleResourceWithAuthBuilder<Y> {
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

    /// Creates resource with no initial supply.
    pub fn no_initial_supply(self) -> ResourceAddress {
        let rtn = ScryptoEnv.call_function(
            RESOURCE_MANAGER_PACKAGE,
            RESOURCE_MANAGER_BLUEPRINT,
            "create_non_fungible",
            scrypto_encode(&ResourceManagerCreateNonFungibleInvocation {
                resource_address: None,
                id_type: Y::id_type(),
                metadata: self.metadata,
                access_rules: self.authorization,
            }).unwrap()
        ).unwrap();

        scrypto_decode(&rtn).unwrap()
    }
}

impl<Y: IsNonAutoGeneratedNonFungibleLocalId> NonFungibleResourceWithAuthBuilder<Y> {
    pub fn initial_supply<T, V>(self, entries: T) -> Bucket
    where
        T: IntoIterator<Item = (Y, V)>,
        V: NonFungibleData,
    {
        let mut encoded = BTreeMap::new();
        for (id, e) in entries {
            encoded.insert(
                id.into(),
                (e.immutable_data().unwrap(), e.mutable_data().unwrap()),
            );
        }

        let rtn = ScryptoEnv.call_function(
            RESOURCE_MANAGER_PACKAGE,
            RESOURCE_MANAGER_BLUEPRINT,
            "create_non_fungible_with_initial_supply",
            scrypto_encode(&ResourceManagerCreateNonFungibleWithInitialSupplyInvocation {
                id_type: Y::id_type(),
                metadata: self.metadata,
                access_rules: self.authorization,
                entries: encoded,
            }).unwrap()
        ).unwrap();
        let (_resource_address, bucket): (ResourceAddress, Bucket) = scrypto_decode(&rtn).unwrap();
        bucket
    }
}

impl NonFungibleResourceWithAuthBuilder<u128> {
    pub fn initial_supply_uuid<T, V>(self, entries: T) -> Bucket
    where
        T: IntoIterator<Item = V>,
        V: NonFungibleData,
    {
        let mut encoded = BTreeSet::new();
        for e in entries {
            encoded.insert((e.immutable_data().unwrap(), e.mutable_data().unwrap()));
        }

        let rtn = ScryptoEnv.call_function(
            RESOURCE_MANAGER_PACKAGE,
            RESOURCE_MANAGER_BLUEPRINT,
            "create_uuid_non_fungible_with_initial_supply",
            scrypto_encode(&ResourceManagerCreateUuidNonFungibleWithInitialSupplyInvocation {
                metadata: self.metadata,
                access_rules: self.authorization,
                entries: encoded,
            }).unwrap()
        ).unwrap();

        let (_resource_address, bucket): (ResourceAddress, Bucket) = scrypto_decode(&rtn).unwrap();
        bucket
    }
}
