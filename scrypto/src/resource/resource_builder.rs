use crate::engine::scrypto_env::ScryptoEnv;
use crate::radix_engine_interface::api::api::SysNativeInvokable;
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

pub struct FungibleResourceBuilder {
    divisibility: u8,
    metadata: HashMap<String, String>,
    authorization: HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)>,
}

pub struct NonFungibleResourceBuilder {
    metadata: HashMap<String, String>,
    authorization: HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)>,
    id_type: NonFungibleIdType,
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

    pub fn mintable(&mut self, method_auth: AccessRule, mutability: Mutability) -> &mut Self {
        self.authorization.insert(Mint, (method_auth, mutability));
        self
    }

    pub fn burnable(&mut self, method_auth: AccessRule, mutability: Mutability) -> &mut Self {
        self.authorization.insert(Burn, (method_auth, mutability));
        self
    }

    pub fn recallable(&mut self, method_auth: AccessRule, mutability: Mutability) -> &mut Self {
        self.authorization
            .insert(Recall, (method_auth, mutability));
        self
    }

    pub fn restrict_withdraw(
        &mut self,
        method_auth: AccessRule,
        mutability: Mutability,
    ) -> &mut Self {
        self.authorization
            .insert(Withdraw, (method_auth, mutability));
        self
    }

    pub fn restrict_deposit(
        &mut self,
        method_auth: AccessRule,
        mutability: Mutability,
    ) -> &mut Self {
        self.authorization
            .insert(Deposit, (method_auth, mutability));
        self
    }

    pub fn updateable_metadata(
        &mut self,
        method_auth: AccessRule,
        mutability: Mutability,
    ) -> &mut Self {
        self.authorization
            .insert(UpdateMetadata, (method_auth, mutability));
        self
    }

    /// Creates resource with the given initial supply.
    ///
    /// # Example
    /// ```ignore
    /// let bucket = ResourceBuilder::new_fungible()
    ///     .metadata("name", "TestToken")
    ///     .initial_supply_no_owner(5);
    /// ```
    pub fn initial_supply_no_owner<T: Into<Decimal>>(&self, amount: T) -> Bucket {
        self.build_no_owner(Some(MintParams::fungible(amount)))
            .1
            .unwrap()
    }

    /// Creates resource with no initial supply.
    pub fn no_initial_supply_no_owner(&self) -> ResourceAddress {
        self.build_no_owner(None).0
    }

    fn build_no_owner(&self, mint_params: Option<MintParams>) -> (ResourceAddress, Option<Bucket>) {
        let mut authorization = self.authorization.clone();
        if !authorization.contains_key(&ResourceMethodAuthKey::Withdraw) {
            authorization.insert(ResourceMethodAuthKey::Withdraw, (rule!(allow_all), LOCKED));
        }

        ScryptoEnv
            .sys_invoke(ResourceManagerCreateNoOwnerInvocation {
                resource_type: ResourceType::Fungible {
                    divisibility: self.divisibility,
                },
                metadata: self.metadata.clone(),
                access_rules: authorization,
                mint_params,
            })
            .unwrap()
    }

    /*
    pub fn initial_supply_with_owner<T: Into<Decimal>>(&self, amount: T) -> (Bucket, Bucket) {
        let (_, bucket, owner_badge_bucket) =
            self.build_with_owner(Some(MintParams::fungible(amount)));
        (bucket.unwrap(), owner_badge_bucket)
    }

    pub fn no_initial_supply_with_owner(&self) -> (ResourceAddress, Bucket) {
        let (resource_address, _, owner_badge_bucket) = self.build_with_owner(None);
        (resource_address, owner_badge_bucket)
    }

    fn build_with_owner(
        &self,
        mint_params: Option<MintParams>,
    ) -> (ResourceAddress, Option<Bucket>, Bucket) {
        let mut authorization = self.authorization.clone();
        if !authorization.contains_key(&ResourceMethodAuthKey::Withdraw) {
            authorization.insert(ResourceMethodAuthKey::Withdraw, (rule!(allow_all), LOCKED));
        }

        ScryptoEnv
            .sys_invoke(ResourceManagerCreateWithManagerInvocation {
                resource_type: ResourceType::Fungible {
                    divisibility: self.divisibility,
                },
                metadata: self.metadata.clone(),
                access_rules: authorization,
                mint_params,
            })
            .unwrap()
    }
     */
}

impl NonFungibleResourceBuilder {
    pub fn new() -> Self {
        Self {
            metadata: HashMap::new(),
            authorization: HashMap::new(),
            id_type: NonFungibleIdType::default(),
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

    pub fn mintable(&mut self, method_auth: AccessRule, mutability: Mutability) -> &mut Self {
        self.authorization.insert(Mint, (method_auth, mutability));
        self
    }

    pub fn burnable(&mut self, method_auth: AccessRule, mutability: Mutability) -> &mut Self {
        self.authorization.insert(Burn, (method_auth, mutability));
        self
    }

    pub fn recallable(&mut self, method_auth: AccessRule, mutability: Mutability) -> &mut Self {
        self.authorization.insert(Recall, (method_auth, mutability));
        self
    }

    pub fn restrict_withdraw(
        &mut self,
        method_auth: AccessRule,
        mutability: Mutability,
    ) -> &mut Self {
        self.authorization
            .insert(Withdraw, (method_auth, mutability));
        self
    }

    pub fn restrict_deposit(
        &mut self,
        method_auth: AccessRule,
        mutability: Mutability,
    ) -> &mut Self {
        self.authorization
            .insert(Deposit, (method_auth, mutability));
        self
    }

    pub fn updateable_non_fungible_data(
        &mut self,
        method_auth: AccessRule,
        mutability: Mutability,
    ) -> &mut Self {
        self.authorization
            .insert(UpdateNonFungibleData, (method_auth, mutability));
        self
    }

    pub fn updateable_metadata(
        &mut self,
        method_auth: AccessRule,
        mutability: Mutability,
    ) -> &mut Self {
        self.authorization
            .insert(UpdateMetadata, (method_auth, mutability));
        self
    }

    /// Set ID type to use for this non fungible resource
    pub fn set_id_type(&mut self, id_type: NonFungibleIdType) -> &mut Self {
        self.id_type = id_type;
        self
    }

    /// Creates resource with the given initial supply.
    ///
    /// # Example
    /// ```ignore
    /// let bucket = ResourceBuilder::new_non_fungible()
    ///     .metadata("name", "TestNonFungible")
    ///     .initial_supply_no_owner([
    ///         (NftKey::from(1u128), "immutable_part", "mutable_part"),
    ///         (NftKey::from(2u128), "another_immutable_part", "another_mutable_part"),
    ///     ]);
    /// ```
    pub fn initial_supply_no_owner<T, V>(&self, entries: T) -> Bucket
    where
        T: IntoIterator<Item = (NonFungibleId, V)>,
        V: NonFungibleData,
    {
        let mut encoded = HashMap::new();
        for (id, e) in entries {
            encoded.insert(id, (e.immutable_data().unwrap(), e.mutable_data().unwrap()));
        }
        self.build_no_owner(Some(MintParams::NonFungible { entries: encoded }))
            .1
            .unwrap()
    }

    /// Creates resource with no initial supply.
    pub fn no_initial_supply_no_owner(&self) -> ResourceAddress {
        self.build_no_owner(None).0
    }

    fn build_no_owner(&self, mint_params: Option<MintParams>) -> (ResourceAddress, Option<Bucket>) {
        let mut authorization = self.authorization.clone();
        if !authorization.contains_key(&ResourceMethodAuthKey::Withdraw) {
            authorization.insert(ResourceMethodAuthKey::Withdraw, (rule!(allow_all), LOCKED));
        }

        ScryptoEnv
            .sys_invoke(ResourceManagerCreateNoOwnerInvocation {
                resource_type: ResourceType::NonFungible {
                    id_type: self.id_type,
                },
                metadata: self.metadata.clone(),
                access_rules: authorization,
                mint_params,
            })
            .unwrap()
    }

    /*
    pub fn initial_supply_with_owner<T, V>(&self, entries: T) -> (Bucket, Bucket)
    where
        T: IntoIterator<Item = (NonFungibleId, V)>,
        V: NonFungibleData,
    {
        let mut encoded = HashMap::new();
        for (id, e) in entries {
            encoded.insert(id, (e.immutable_data().unwrap(), e.mutable_data().unwrap()));
        }
        let (_, bucket, owner_badge_bucket) =
            self.build_with_owner(Some(MintParams::NonFungible { entries: encoded }));
        (bucket.unwrap(), owner_badge_bucket)
    }

    /// Creates resource with no initial supply.
    pub fn no_initial_supply_with_owner(&self) -> ResourceAddress {
        self.build_with_owner(None).0
    }

    fn build_with_owner(
        &self,
        mint_params: Option<MintParams>,
    ) -> (ResourceAddress, Option<Bucket>, Bucket) {
        let mut authorization = self.authorization.clone();
        if !authorization.contains_key(&ResourceMethodAuthKey::Withdraw) {
            authorization.insert(ResourceMethodAuthKey::Withdraw, (rule!(allow_all), LOCKED));
        }

        ScryptoEnv
            .sys_invoke(ResourceManagerCreateWithManagerInvocation {
                resource_type: ResourceType::NonFungible {
                    id_type: self.id_type,
                },
                metadata: self.metadata.clone(),
                access_rules: authorization,
                mint_params,
            })
            .unwrap()
    }
     */
}
