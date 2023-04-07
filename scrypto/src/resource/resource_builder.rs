use crate::engine::scrypto_env::ScryptoEnv;
use crate::radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::constants::RESOURCE_MANAGER_PACKAGE;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::NonFungibleData;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::collections::*;
use sbor::rust::marker::PhantomData;
use sbor::rust::string::String;
use scrypto::prelude::ScryptoValue;

/// Not divisible.
pub const DIVISIBILITY_NONE: u8 = 0;
/// The maximum divisibility supported.
pub const DIVISIBILITY_MAXIMUM: u8 = 18;

/// Utility for setting up a new resource.
///
/// * You start the building process with one of the methods starting with `new_`.
/// * The allowed methods change depending on which methods have already been called.
///   For example, you can either use `owner_non_fungible_badge` or set access rules individually, but not both.
/// * You can complete the building process using either `create_with_no_initial_supply()` or `mint_initial_supply(..)`.
///
/// ### Example
/// ```no_run
/// use scrypto::prelude::*;
///
/// let bucket = ResourceBuilder::new_fungible()
///     .metadata("name", "TestToken")
///     .mint_initial_supply(5);
/// ```
pub struct ResourceBuilder;

impl ResourceBuilder {
    /// Starts a new builder to create a fungible resource.
    pub fn new_fungible() -> InProgressResourceBuilder<FungibleResourceType, NoAuth> {
        InProgressResourceBuilder::default()
    }

    /// Starts a new builder to create a non-fungible resource with a `NonFungibleIdType::String`
    pub fn new_string_non_fungible<D: NonFungibleData>(
    ) -> InProgressResourceBuilder<NonFungibleResourceType<StringNonFungibleLocalId, D>, NoAuth>
    {
        InProgressResourceBuilder::default()
    }

    /// Starts a new builder to create a non-fungible resource with a `NonFungibleIdType::Integer`
    pub fn new_integer_non_fungible<D: NonFungibleData>(
    ) -> InProgressResourceBuilder<NonFungibleResourceType<IntegerNonFungibleLocalId, D>, NoAuth>
    {
        InProgressResourceBuilder::default()
    }

    /// Starts a new builder to create a non-fungible resource with a `NonFungibleIdType::Bytes`
    pub fn new_bytes_non_fungible<D: NonFungibleData>(
    ) -> InProgressResourceBuilder<NonFungibleResourceType<BytesNonFungibleLocalId, D>, NoAuth>
    {
        InProgressResourceBuilder::default()
    }

    /// Starts a new builder to create a non-fungible resource with a `NonFungibleIdType::UUID`
    pub fn new_uuid_non_fungible<D: NonFungibleData>(
    ) -> InProgressResourceBuilder<NonFungibleResourceType<UUIDNonFungibleLocalId, D>, NoAuth> {
        InProgressResourceBuilder::default()
    }
}

/// Utility for setting up a new resource, which has building in progress.
///
/// * You start the building process with one of the methods starting with `ResourceBuilder::new_`.
/// * The allowed methods change depending on which methods have already been called.
///   For example, you can either use `owner_non_fungible_badge` or set access rules individually, but not both.
/// * You can complete the building process using either `create_with_no_initial_supply()` or `mint_initial_supply(..)`.
///
/// ### Example
/// ```no_run
/// use scrypto::prelude::*;
///
/// let bucket = ResourceBuilder::new_fungible()
///     .metadata("name", "TestToken")
///     .mint_initial_supply(5);
/// ```
#[must_use]
pub struct InProgressResourceBuilder<T: AnyResourceType, A: ConfiguredAuth> {
    resource_type: T,
    metadata: BTreeMap<String, String>,
    auth: A,
}

impl<T: AnyResourceType> Default for InProgressResourceBuilder<T, NoAuth> {
    fn default() -> Self {
        Self {
            resource_type: T::default(),
            metadata: BTreeMap::new(),
            auth: NoAuth,
        }
    }
}

pub trait ConfiguredAuth {
    fn into_access_rules(self) -> BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>;
}

pub struct NoAuth;
impl ConfiguredAuth for NoAuth {
    fn into_access_rules(self) -> BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)> {
        BTreeMap::new()
    }
}

pub struct AccessRuleAuth(BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>);

impl ConfiguredAuth for AccessRuleAuth {
    fn into_access_rules(self) -> BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)> {
        self.0
    }
}

pub struct OwnerBadgeAuth(NonFungibleGlobalId);
impl ConfiguredAuth for OwnerBadgeAuth {
    fn into_access_rules(self) -> BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)> {
        resource_access_rules_from_owner_badge(&self.0)
    }
}

// Various types for ResourceType
pub trait AnyResourceType: Default {}

pub struct FungibleResourceType {
    divisibility: u8,
}
impl AnyResourceType for FungibleResourceType {}
impl Default for FungibleResourceType {
    fn default() -> Self {
        Self {
            divisibility: DIVISIBILITY_MAXIMUM,
        }
    }
}

pub struct NonFungibleResourceType<T: IsNonFungibleLocalId, D: NonFungibleData>(
    PhantomData<T>,
    PhantomData<D>,
);
impl<T: IsNonFungibleLocalId, D: NonFungibleData> AnyResourceType
    for NonFungibleResourceType<T, D>
{
}
impl<T: IsNonFungibleLocalId, D: NonFungibleData> Default for NonFungibleResourceType<T, D> {
    fn default() -> Self {
        Self(PhantomData, PhantomData)
    }
}

// Builder types
pub trait IsFungibleBuilder {}
impl<A: ConfiguredAuth> IsFungibleBuilder for InProgressResourceBuilder<FungibleResourceType, A> {}

pub trait IsNonFungibleBuilder {}
impl<A: ConfiguredAuth, Y: IsNonFungibleLocalId, D: NonFungibleData> IsNonFungibleBuilder
    for InProgressResourceBuilder<NonFungibleResourceType<Y, D>, A>
{
}

////////////////////////////////////////////////////////////
/// PUBLIC TRAITS AND METHODS
/// All public methods first - these all need good rust docs
////////////////////////////////////////////////////////////

pub trait UpdateMetadataBuilder: private::CanAddMetadata {
    /// Adds a resource metadata.
    ///
    /// If a previous attribute with the same name has been set, it will be overwritten.
    fn metadata<K: Into<String>, V: Into<String>>(self, name: K, value: V) -> Self::OutputBuilder {
        self.add_metadata(name.into(), value.into())
    }
}
impl<B: private::CanAddMetadata> UpdateMetadataBuilder for B {}

pub trait UpdateAuthBuilder: private::CanAddAuth {
    /// Sets the resource to be mintable.
    ///
    /// * The first parameter is the access rule which allows minting of the resource.
    /// * The second parameter is the mutability / access rule which controls if and how the access rule can be updated.
    ///
    /// ### Examples
    ///
    /// ```no_run
    /// use scrypto::prelude::*;
    ///
    /// # let resource_address = RADIX_TOKEN;
    /// // Sets the resource to be mintable with a proof of a specific resource, and this is locked forever.
    /// ResourceBuilder::new_fungible()
    ///    .mintable(rule!(require(resource_address)), LOCKED);
    ///
    /// # let resource_address = RADIX_TOKEN;
    /// // Sets the resource to not be mintable, but this is can be changed in future by the second rule
    /// ResourceBuilder::new_fungible()
    ///    .mintable(rule!(deny_all), MUTABLE(rule!(require(resource_address))));
    /// ```
    fn mintable<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> Self::OutputBuilder {
        self.add_auth(Mint, method_auth, mutability.into())
    }

    /// Sets the resource to be burnable.
    ///
    /// * The first parameter is the access rule which allows minting of the resource.
    /// * The second parameter is the mutability / access rule which controls if and how the access rule can be updated.
    ///
    /// ### Examples
    ///
    /// ```no_run
    /// use scrypto::prelude::*;
    ///
    /// # let resource_address = RADIX_TOKEN;
    /// // Sets the resource to be burnable with a proof of a specific resource, and this is locked forever.
    /// ResourceBuilder::new_fungible()
    ///    .burnable(rule!(require(resource_address)), LOCKED);
    ///
    /// # let resource_address = RADIX_TOKEN;
    /// // Sets the resource to be freely burnable, but this is can be changed in future by the second rule.
    /// ResourceBuilder::new_fungible()
    ///    .burnable(rule!(allow_all), MUTABLE(rule!(require(resource_address))));
    /// ```
    fn burnable<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> Self::OutputBuilder {
        self.add_auth(Burn, method_auth, mutability.into())
    }

    /// Sets the resource to be recallable from vaults.
    ///
    /// * The first parameter is the access rule which allows recalling of the resource.
    /// * The second parameter is the mutability / access rule which controls if and how the access rule can be updated.
    ///
    /// ### Examples
    ///
    /// ```no_run
    /// use scrypto::prelude::*;
    ///
    /// # let resource_address = RADIX_TOKEN;
    /// // Sets the resource to be recallable with a proof of a specific resource, and this is locked forever.
    /// ResourceBuilder::new_fungible()
    ///    .recallable(rule!(require(resource_address)), LOCKED);
    ///
    /// # let resource_address = RADIX_TOKEN;
    /// // Sets the resource to not be recallable, but this is can be changed in future by the second rule
    /// ResourceBuilder::new_fungible()
    ///    .recallable(rule!(deny_all), MUTABLE(rule!(require(resource_address))));
    /// ```
    fn recallable<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> Self::OutputBuilder {
        self.add_auth(Recall, method_auth, mutability.into())
    }

    /// Sets the resource to not be freely withdrawable from a vault.
    ///
    /// * The first parameter is the access rule which allows withdrawing from a vault.
    /// * The second parameter is the mutability / access rule which controls if and how the access rule can be updated.
    ///
    /// ### Examples
    ///
    /// ```no_run
    /// use scrypto::prelude::*;
    ///
    /// # let resource_address = RADIX_TOKEN;
    /// // Sets the resource to be withdrawable with a proof of a specific resource, and this is locked forever.
    /// ResourceBuilder::new_fungible()
    ///    .restrict_withdraw(rule!(require(resource_address)), LOCKED);
    ///
    /// # let resource_address = RADIX_TOKEN;
    /// // Sets the resource to not be withdrawable, but this is can be changed in future by the second rule
    /// ResourceBuilder::new_fungible()
    ///    .restrict_withdraw(rule!(deny_all), MUTABLE(rule!(require(resource_address))));
    /// ```
    fn restrict_withdraw<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> Self::OutputBuilder {
        self.add_auth(Withdraw, method_auth, mutability.into())
    }

    /// Sets the resource to not be freely depositable into a vault.
    ///
    /// * The first parameter is the access rule which allows depositing into a vault.
    /// * The second parameter is the mutability / access rule which controls if and how the access rule can be updated.
    ///
    /// ### Examples
    ///
    /// ```no_run
    /// use scrypto::prelude::*;
    ///
    /// # let resource_address = RADIX_TOKEN;
    /// // Sets the resource to be depositable with a proof of a specific resource, and this is locked forever.
    /// ResourceBuilder::new_fungible()
    ///    .restrict_deposit(rule!(require(resource_address)), LOCKED);
    ///
    /// # let resource_address = RADIX_TOKEN;
    /// // Sets the resource to not be depositable, but this is can be changed in future by the second rule
    /// ResourceBuilder::new_fungible()
    ///    .restrict_deposit(rule!(deny_all), MUTABLE(rule!(require(resource_address))));
    /// ```
    fn restrict_deposit<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> Self::OutputBuilder {
        self.add_auth(Deposit, method_auth, mutability.into())
    }

    /// Sets how the resource's metadata can be updated.
    ///
    /// * The first parameter is the access rule which allows updating the metadata of the resource.
    /// * The second parameter is the mutability / access rule which controls if and how the access rule can be updated.
    ///
    /// ### Examples
    ///
    /// ```no_run
    /// use scrypto::prelude::*;
    ///
    /// # let resource_address = RADIX_TOKEN;
    /// // Sets the resource to allow its metadata to be updated with a proof of a specific resource, and this is locked forever.
    /// ResourceBuilder::new_fungible()
    ///    .updateable_metadata(rule!(require(resource_address)), LOCKED);
    ///
    /// # let resource_address = RADIX_TOKEN;
    /// // Sets the resource to not allow its metadata to be updated, but this is can be changed in future by the second rule.
    /// ResourceBuilder::new_fungible()
    ///    .updateable_metadata(rule!(deny_all), MUTABLE(rule!(require(resource_address))));
    /// ```
    fn updateable_metadata<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> Self::OutputBuilder {
        self.add_auth(UpdateMetadata, method_auth, mutability.into())
    }
}
impl<B: private::CanAddAuth> UpdateAuthBuilder for B {}

pub trait UpdateNonFungibleAuthBuilder: IsNonFungibleBuilder + private::CanAddAuth {
    /// Sets how each non-fungible's mutable data can be updated.
    ///
    /// * The first parameter is the access rule which allows updating the mutable data of each non-fungible.
    /// * The second parameter is the mutability / access rule which controls if and how the access rule can be updated.
    ///
    /// ### Examples
    ///
    /// ```no_run
    /// use scrypto::prelude::*;
    ///
    /// # let resource_address = RADIX_TOKEN;
    ///
    /// #[derive(ScryptoSbor, NonFungibleData)]
    /// struct NFData {
    ///     pub name: String,
    ///     #[mutable]
    ///     pub flag: bool,
    /// }
    /// // Permits the updating of non-fungible mutable data with a proof of a specific resource, and this is locked forever.
    /// ResourceBuilder::new_uuid_non_fungible::<NFData>()
    ///    .updateable_non_fungible_data(rule!(require(resource_address)), LOCKED);
    ///
    /// # let resource_address = RADIX_TOKEN;
    /// // Does not currently permit the updating of non-fungible mutable data, but this is can be changed in future by the second rule.
    /// ResourceBuilder::new_uuid_non_fungible::<NFData>()
    ///    .updateable_non_fungible_data(rule!(deny_all), MUTABLE(rule!(require(resource_address))));
    /// ```
    fn updateable_non_fungible_data<R: Into<AccessRule>>(
        self,
        method_auth: AccessRule,
        mutability: R,
    ) -> Self::OutputBuilder {
        self.add_auth(UpdateNonFungibleData, method_auth, mutability.into())
    }
}
impl<B: IsNonFungibleBuilder + private::CanAddAuth> UpdateNonFungibleAuthBuilder for B {}

pub trait SetOwnerBuilder: private::CanAddOwner {
    /// Sets the owner badge to be the given non-fungible.
    ///
    /// The owner badge is given starting permissions to update the metadata/data associated with the resource,
    /// and to change any of the access rules after creation.
    fn owner_non_fungible_badge(self, owner_badge: NonFungibleGlobalId) -> Self::OutputBuilder {
        self.set_owner(owner_badge)
    }
}
impl<B: private::CanAddOwner> SetOwnerBuilder for B {}

pub trait CreateWithNoSupplyBuilder: private::CanCreateWithNoSupply {
    /// Creates the resource with no initial supply.
    ///
    /// The resource's address is returned.
    fn create_with_no_initial_supply(self) -> ResourceAddress {
        match self.into_create_with_no_supply_invocation() {
            private::CreateWithNoSupply::Fungible {
                divisibility,
                metadata,
                access_rules,
            } => ScryptoEnv
                .call_function(
                    RESOURCE_MANAGER_PACKAGE,
                    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
                    scrypto_encode(&FungibleResourceManagerCreateInput {
                        divisibility,
                        metadata,
                        access_rules,
                    })
                    .unwrap(),
                )
                .map(|bytes| scrypto_decode(&bytes).unwrap())
                .unwrap(),
            private::CreateWithNoSupply::NonFungible {
                id_type,
                non_fungible_schema,
                metadata,
                access_rules,
            } => ScryptoEnv
                .call_function(
                    RESOURCE_MANAGER_PACKAGE,
                    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
                    scrypto_encode(&NonFungibleResourceManagerCreateInput {
                        id_type,
                        non_fungible_schema,
                        metadata,
                        access_rules,
                    })
                    .unwrap(),
                )
                .map(|bytes| scrypto_decode(&bytes).unwrap())
                .unwrap(),
        }
    }
}
impl<B: private::CanCreateWithNoSupply> CreateWithNoSupplyBuilder for B {}

impl<A: ConfiguredAuth> InProgressResourceBuilder<FungibleResourceType, A> {
    /// Set the resource's divisibility: the number of digits of precision after the decimal point in its balances.
    ///
    /// * `0` means the resource is not divisible (balances are always whole numbers)
    /// * `18` is the maximum divisibility, and the default.
    ///
    /// ### Examples
    ///
    /// ```no_run
    /// use scrypto::prelude::*;
    ///
    /// // Only permits whole-number balances.
    /// ResourceBuilder::new_fungible()
    ///    .divisibility(0);
    ///
    /// // Only permits balances to 3 decimal places.
    /// ResourceBuilder::new_fungible()
    ///    .divisibility(3);
    /// ```
    pub fn divisibility(mut self, divisibility: u8) -> Self {
        assert!(divisibility <= 18);
        self.resource_type = FungibleResourceType { divisibility };
        self
    }
}

impl<A: ConfiguredAuth> InProgressResourceBuilder<FungibleResourceType, A> {
    /// Creates resource with the given initial supply.
    ///
    /// # Example
    /// ```no_run
    /// use scrypto::prelude::*;
    ///
    /// let bucket = ResourceBuilder::new_fungible()
    ///     .mint_initial_supply(5);
    /// ```
    pub fn mint_initial_supply<T: Into<Decimal>>(self, amount: T) -> Bucket {
        ScryptoEnv
            .call_function(
                RESOURCE_MANAGER_PACKAGE,
                FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
                scrypto_encode(&FungibleResourceManagerCreateWithInitialSupplyInput {
                    divisibility: self.resource_type.divisibility,
                    metadata: self.metadata,
                    access_rules: self.auth.into_access_rules(),
                    initial_supply: amount.into(),
                })
                .unwrap(),
            )
            .map(|bytes| {
                scrypto_decode::<(ResourceAddress, Bucket)>(&bytes)
                    .unwrap()
                    .1
            })
            .unwrap()
    }
}

impl<A: ConfiguredAuth, D: NonFungibleData>
    InProgressResourceBuilder<NonFungibleResourceType<StringNonFungibleLocalId, D>, A>
{
    /// Creates the non-fungible resource, and mints an individual non-fungible for each key/data pair provided.
    ///
    /// ### Example
    /// ```no_run
    /// use scrypto::prelude::*;
    ///
    /// #[derive(ScryptoSbor, NonFungibleData)]
    /// struct NFData {
    ///     pub name: String,
    ///     #[mutable]
    ///     pub flag: bool,
    /// }
    ///
    /// let bucket = ResourceBuilder::new_string_non_fungible::<NFData>()
    ///     .mint_initial_supply([
    ///         ("One".try_into().unwrap(), NFData { name: "NF One".to_owned(), flag: true }),
    ///         ("Two".try_into().unwrap(), NFData { name: "NF Two".to_owned(), flag: true }),
    ///     ]);
    /// ```
    pub fn mint_initial_supply<T>(self, entries: T) -> Bucket
    where
        T: IntoIterator<Item = (StringNonFungibleLocalId, D)>,
    {
        ScryptoEnv
            .call_function(
                RESOURCE_MANAGER_PACKAGE,
                NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
                scrypto_encode(&NonFungibleResourceManagerCreateWithInitialSupplyInput {
                    id_type: StringNonFungibleLocalId::id_type(),
                    non_fungible_schema: NonFungibleDataSchema::new_schema::<D>(),
                    metadata: self.metadata,
                    access_rules: self.auth.into_access_rules(),
                    entries: map_entries(entries),
                })
                .unwrap(),
            )
            .map(|bytes| {
                scrypto_decode::<(ResourceAddress, Bucket)>(&bytes)
                    .unwrap()
                    .1
            })
            .unwrap()
    }
}

impl<A: ConfiguredAuth, D: NonFungibleData>
    InProgressResourceBuilder<NonFungibleResourceType<IntegerNonFungibleLocalId, D>, A>
{
    /// Creates the non-fungible resource, and mints an individual non-fungible for each key/data pair provided.
    ///
    /// ### Example
    /// ```no_run
    /// use scrypto::prelude::*;
    ///
    /// #[derive(ScryptoSbor, NonFungibleData)]
    /// struct NFData {
    ///     pub name: String,
    ///     #[mutable]
    ///     pub flag: bool,
    /// }
    ///
    /// let bucket = ResourceBuilder::new_integer_non_fungible()
    ///     .mint_initial_supply([
    ///         (1u64.into(), NFData { name: "NF One".to_owned(), flag: true }),
    ///         (2u64.into(), NFData { name: "NF Two".to_owned(), flag: true }),
    ///     ]);
    /// ```
    pub fn mint_initial_supply<T>(self, entries: T) -> Bucket
    where
        T: IntoIterator<Item = (IntegerNonFungibleLocalId, D)>,
    {
        ScryptoEnv
            .call_function(
                RESOURCE_MANAGER_PACKAGE,
                NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
                scrypto_encode(&NonFungibleResourceManagerCreateWithInitialSupplyInput {
                    id_type: IntegerNonFungibleLocalId::id_type(),
                    non_fungible_schema: NonFungibleDataSchema::new_schema::<D>(),
                    metadata: self.metadata,
                    access_rules: self.auth.into_access_rules(),
                    entries: map_entries(entries),
                })
                .unwrap(),
            )
            .map(|bytes| {
                scrypto_decode::<(ResourceAddress, Bucket)>(&bytes)
                    .unwrap()
                    .1
            })
            .unwrap()
    }
}

impl<A: ConfiguredAuth, D: NonFungibleData>
    InProgressResourceBuilder<NonFungibleResourceType<BytesNonFungibleLocalId, D>, A>
{
    /// Creates the non-fungible resource, and mints an individual non-fungible for each key/data pair provided.
    ///
    /// ### Example
    /// ```no_run
    /// use scrypto::prelude::*;
    ///
    /// #[derive(ScryptoSbor, NonFungibleData)]
    /// struct NFData {
    ///     pub name: String,
    ///     #[mutable]
    ///     pub flag: bool,
    /// }
    ///
    /// let bucket = ResourceBuilder::new_bytes_non_fungible::<NFData>()
    ///     .mint_initial_supply([
    ///         (vec![1u8].try_into().unwrap(), NFData { name: "NF One".to_owned(), flag: true }),
    ///         (vec![2u8].try_into().unwrap(), NFData { name: "NF Two".to_owned(), flag: true }),
    ///     ]);
    /// ```
    pub fn mint_initial_supply<T>(self, entries: T) -> Bucket
    where
        T: IntoIterator<Item = (BytesNonFungibleLocalId, D)>,
    {
        ScryptoEnv
            .call_function(
                RESOURCE_MANAGER_PACKAGE,
                NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
                scrypto_encode(&NonFungibleResourceManagerCreateWithInitialSupplyInput {
                    id_type: BytesNonFungibleLocalId::id_type(),
                    non_fungible_schema: NonFungibleDataSchema::new_schema::<D>(),
                    metadata: self.metadata,
                    access_rules: self.auth.into_access_rules(),
                    entries: map_entries(entries),
                })
                .unwrap(),
            )
            .map(|bytes| {
                scrypto_decode::<(ResourceAddress, Bucket)>(&bytes)
                    .unwrap()
                    .1
            })
            .unwrap()
    }
}

impl<A: ConfiguredAuth, D: NonFungibleData>
    InProgressResourceBuilder<NonFungibleResourceType<UUIDNonFungibleLocalId, D>, A>
{
    /// Creates the UUID non-fungible resource, and mints an individual non-fungible for each piece of data provided.
    ///
    /// The system automatically generates a new UUID `NonFungibleLocalId` for each non-fungible,
    /// and assigns the given data to each.
    ///
    /// ### Example
    /// ```no_run
    /// use scrypto::prelude::*;
    ///
    /// #[derive(ScryptoSbor, NonFungibleData)]
    /// struct NFData {
    ///     pub name: String,
    ///     #[mutable]
    ///     pub flag: bool,
    /// }
    ///
    /// let bucket = ResourceBuilder::new_uuid_non_fungible::<NFData>()
    ///     .mint_initial_supply([
    ///         (NFData { name: "NF One".to_owned(), flag: true }),
    ///         (NFData { name: "NF Two".to_owned(), flag: true }),
    ///     ]);
    /// ```
    pub fn mint_initial_supply<T>(self, entries: T) -> Bucket
    where
        T: IntoIterator<Item = D>,
    {
        ScryptoEnv
            .call_function(
                RESOURCE_MANAGER_PACKAGE,
                NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_UUID_WITH_INITIAL_SUPPLY_IDENT,
                scrypto_encode(
                    &NonFungibleResourceManagerCreateUuidWithInitialSupplyInput {
                        non_fungible_schema: NonFungibleDataSchema::new_schema::<D>(),
                        metadata: self.metadata,
                        access_rules: self.auth.into_access_rules(),
                        entries: entries
                            .into_iter()
                            .map(|data| {
                                let value: ScryptoValue =
                                    scrypto_decode(&scrypto_encode(&data).unwrap()).unwrap();
                                (value,)
                            })
                            .collect(),
                    },
                )
                .unwrap(),
            )
            .map(|bytes| {
                scrypto_decode::<(ResourceAddress, Bucket)>(&bytes)
                    .unwrap()
                    .1
            })
            .unwrap()
    }
}

///////////////////////////////////
/// PRIVATE TRAIT IMPLEMENTATIONS
/// These don't need good rust docs
///////////////////////////////////

fn map_entries<T: IntoIterator<Item = (Y, V)>, V: NonFungibleData, Y: IsNonFungibleLocalId>(
    entries: T,
) -> BTreeMap<NonFungibleLocalId, (ScryptoValue,)> {
    entries
        .into_iter()
        .map(|(id, data)| {
            let value: ScryptoValue = scrypto_decode(&scrypto_encode(&data).unwrap()).unwrap();
            (id.into(), (value,))
        })
        .collect()
}

impl<T: AnyResourceType, A: ConfiguredAuth> private::CanAddMetadata
    for InProgressResourceBuilder<T, A>
{
    type OutputBuilder = Self;

    fn add_metadata(mut self, key: String, value: String) -> Self::OutputBuilder {
        self.metadata.insert(key, value);
        self
    }
}

impl<T: AnyResourceType> private::CanAddAuth for InProgressResourceBuilder<T, NoAuth> {
    type OutputBuilder = InProgressResourceBuilder<T, AccessRuleAuth>;

    fn add_auth(
        self,
        method: ResourceMethodAuthKey,
        method_auth: AccessRule,
        mutability: AccessRule,
    ) -> Self::OutputBuilder {
        Self::OutputBuilder {
            resource_type: self.resource_type,
            metadata: self.metadata,
            auth: AccessRuleAuth(btreemap! { method => (method_auth, mutability) }),
        }
    }
}

impl<T: AnyResourceType> private::CanAddAuth for InProgressResourceBuilder<T, AccessRuleAuth> {
    type OutputBuilder = Self;

    fn add_auth(
        mut self,
        method: ResourceMethodAuthKey,
        method_auth: AccessRule,
        mutability: AccessRule,
    ) -> Self::OutputBuilder {
        self.auth.0.insert(method, (method_auth, mutability));
        self
    }
}

impl<T: AnyResourceType> private::CanAddOwner for InProgressResourceBuilder<T, NoAuth> {
    type OutputBuilder = InProgressResourceBuilder<T, OwnerBadgeAuth>;

    fn set_owner(self, owner_badge: NonFungibleGlobalId) -> Self::OutputBuilder {
        Self::OutputBuilder {
            resource_type: self.resource_type,
            metadata: self.metadata,
            auth: OwnerBadgeAuth(owner_badge),
        }
    }
}

impl<A: ConfiguredAuth> private::CanCreateWithNoSupply
    for InProgressResourceBuilder<FungibleResourceType, A>
{
    fn into_create_with_no_supply_invocation(self) -> private::CreateWithNoSupply {
        private::CreateWithNoSupply::Fungible {
            divisibility: self.resource_type.divisibility,
            metadata: self.metadata,
            access_rules: self.auth.into_access_rules(),
        }
    }
}

impl<A: ConfiguredAuth, Y: IsNonFungibleLocalId, D: NonFungibleData> private::CanCreateWithNoSupply
    for InProgressResourceBuilder<NonFungibleResourceType<Y, D>, A>
{
    fn into_create_with_no_supply_invocation(self) -> private::CreateWithNoSupply {
        private::CreateWithNoSupply::NonFungible {
            id_type: Y::id_type(),
            non_fungible_schema: NonFungibleDataSchema::new_schema::<D>(),
            metadata: self.metadata,
            access_rules: self.auth.into_access_rules(),
        }
    }
}

/// This file was experiencing combinatorial explosion - as part of the clean-up, we've used private traits to keep things simple.
///
/// Each public method has essentially one implementation, and one Rust doc (where there weren't clashes due to Rust trait issues -
/// eg with the `mint_initial_supply` methods).
///
/// Internally, the various builders implement these private traits, and then automatically implement the "nice" public traits.
/// The methods defined in the private traits are less nice, and so are hidden in order to not pollute the user facing API.
///
/// As users will nearly always use `scrypto::prelude::*`, as long as we make sure that the public traits are exported, this will
/// be seamless for the user.
///
/// See https://stackoverflow.com/a/53207767 for more information on this.
mod private {
    use super::*;
    use radix_engine_interface::blueprints::resource::{
        AccessRule, NonFungibleGlobalId, ResourceMethodAuthKey,
    };

    pub trait CanAddMetadata: Sized {
        type OutputBuilder;

        fn add_metadata(self, key: String, value: String) -> Self::OutputBuilder;
    }

    pub trait CanAddAuth: Sized {
        type OutputBuilder;

        fn add_auth(
            self,
            method: ResourceMethodAuthKey,
            auth: AccessRule,
            mutability: AccessRule,
        ) -> Self::OutputBuilder;
    }

    pub trait CanAddOwner: Sized {
        type OutputBuilder;

        fn set_owner(self, owner_badge: NonFungibleGlobalId) -> Self::OutputBuilder;
    }

    pub trait CanCreateWithNoSupply: Sized {
        fn into_create_with_no_supply_invocation(self) -> CreateWithNoSupply;
    }

    pub enum CreateWithNoSupply {
        Fungible {
            divisibility: u8,
            metadata: BTreeMap<String, String>,
            access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        },
        NonFungible {
            id_type: NonFungibleIdType,
            non_fungible_schema: NonFungibleDataSchema,
            metadata: BTreeMap<String, String>,
            access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        },
    }
}
