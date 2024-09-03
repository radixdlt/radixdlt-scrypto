use crate::engine::scrypto_env::ScryptoVmV1Api;
use crate::runtime::Runtime;
use radix_common::constants::RESOURCE_PACKAGE;
use radix_common::data::scrypto::model::*;
use radix_common::prelude::*;
use radix_common::traits::NonFungibleData;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::object_modules::metadata::MetadataInit;
use radix_engine_interface::object_modules::role_assignment::RoleDefinition;
use radix_engine_interface::object_modules::ModuleConfig;
use scrypto::resource::{FungibleResourceManager, NonFungibleResourceManager};

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
/// let bucket = ResourceBuilder::new_fungible(OwnerRole::None)
///     .mint_initial_supply(5);
/// ```
pub struct ResourceBuilder;

impl ResourceBuilder {
    /// Starts a new builder to create a fungible resource.
    pub fn new_fungible(owner_role: OwnerRole) -> InProgressResourceBuilder<FungibleResourceType> {
        InProgressResourceBuilder::new(owner_role, FungibleResourceType::default())
    }

    /// Starts a new builder to create a non-fungible resource with a `NonFungibleIdType::String`
    pub fn new_string_non_fungible<D: NonFungibleData>(
        owner_role: OwnerRole,
    ) -> InProgressResourceBuilder<
        NonFungibleResourceType<
            StringNonFungibleLocalId,
            D,
            SborFixedEnumVariant<
                NON_FUNGIBLE_DATA_SCHEMA_VARIANT_LOCAL,
                LocalNonFungibleDataSchema,
            >,
        >,
    > {
        InProgressResourceBuilder::new(
            owner_role,
            NonFungibleResourceType::new(SborFixedEnumVariant {
                fields: LocalNonFungibleDataSchema::new_with_self_package_replacement::<D>(
                    Runtime::package_address(),
                ),
            }),
        )
    }

    /// Starts a new builder to create a non-fungible resource with a `NonFungibleIdType::Integer`
    pub fn new_integer_non_fungible<D: NonFungibleData>(
        owner_role: OwnerRole,
    ) -> InProgressResourceBuilder<
        NonFungibleResourceType<
            IntegerNonFungibleLocalId,
            D,
            SborFixedEnumVariant<
                NON_FUNGIBLE_DATA_SCHEMA_VARIANT_LOCAL,
                LocalNonFungibleDataSchema,
            >,
        >,
    > {
        InProgressResourceBuilder::new(
            owner_role,
            NonFungibleResourceType::new(SborFixedEnumVariant {
                fields: LocalNonFungibleDataSchema::new_with_self_package_replacement::<D>(
                    Runtime::package_address(),
                ),
            }),
        )
    }

    /// Starts a new builder to create a non-fungible resource with a `NonFungibleIdType::Bytes`
    pub fn new_bytes_non_fungible<D: NonFungibleData>(
        owner_role: OwnerRole,
    ) -> InProgressResourceBuilder<
        NonFungibleResourceType<
            BytesNonFungibleLocalId,
            D,
            SborFixedEnumVariant<
                NON_FUNGIBLE_DATA_SCHEMA_VARIANT_LOCAL,
                LocalNonFungibleDataSchema,
            >,
        >,
    > {
        InProgressResourceBuilder::new(
            owner_role,
            NonFungibleResourceType::new(SborFixedEnumVariant {
                fields: LocalNonFungibleDataSchema::new_with_self_package_replacement::<D>(
                    Runtime::package_address(),
                ),
            }),
        )
    }

    /// Starts a new builder to create a non-fungible resource with a `NonFungibleIdType::RUID`
    pub fn new_ruid_non_fungible<D: NonFungibleData>(
        owner_role: OwnerRole,
    ) -> InProgressResourceBuilder<
        NonFungibleResourceType<
            RUIDNonFungibleLocalId,
            D,
            SborFixedEnumVariant<
                NON_FUNGIBLE_DATA_SCHEMA_VARIANT_LOCAL,
                LocalNonFungibleDataSchema,
            >,
        >,
    > {
        InProgressResourceBuilder::new(
            owner_role,
            NonFungibleResourceType::new(SborFixedEnumVariant {
                fields: LocalNonFungibleDataSchema::new_with_self_package_replacement::<D>(
                    Runtime::package_address(),
                ),
            }),
        )
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
/// let bucket = ResourceBuilder::new_fungible(OwnerRole::None)
///     .mint_initial_supply(5);
/// ```
#[must_use]
pub struct InProgressResourceBuilder<T: AnyResourceType> {
    owner_role: OwnerRole,
    resource_type: T,
    resource_roles: T::ResourceRoles,
    metadata_config: Option<ModuleConfig<MetadataInit>>,
    address_reservation: Option<GlobalAddressReservation>,
}

impl<T: AnyResourceType> InProgressResourceBuilder<T> {
    pub fn new(owner_role: OwnerRole, resource_type: T) -> Self {
        Self {
            owner_role,
            resource_type,
            metadata_config: None,
            address_reservation: None,
            resource_roles: T::ResourceRoles::default(),
        }
    }
}

// Various types for ResourceType
pub trait AnyResourceType {
    type ResourceRoles: Default;
}

pub struct FungibleResourceType {
    divisibility: u8,
}
impl AnyResourceType for FungibleResourceType {
    type ResourceRoles = FungibleResourceRoles;
}
impl Default for FungibleResourceType {
    fn default() -> Self {
        Self {
            divisibility: DIVISIBILITY_MAXIMUM,
        }
    }
}

pub struct NonFungibleResourceType<
    T: IsNonFungibleLocalId,
    D: NonFungibleData,
    S: ScryptoCategorize + ScryptoEncode + ScryptoDecode,
>(S, PhantomData<T>, PhantomData<D>);
impl<
        T: IsNonFungibleLocalId,
        D: NonFungibleData,
        S: ScryptoCategorize + ScryptoEncode + ScryptoDecode,
    > AnyResourceType for NonFungibleResourceType<T, D, S>
{
    type ResourceRoles = NonFungibleResourceRoles;
}
impl<
        T: IsNonFungibleLocalId,
        D: NonFungibleData,
        S: ScryptoCategorize + ScryptoEncode + ScryptoDecode,
    > NonFungibleResourceType<T, D, S>
{
    pub fn new(schema: S) -> Self {
        Self(schema, PhantomData, PhantomData)
    }
}

////////////////////////////////////////////////////////////
/// PUBLIC TRAITS AND METHODS
/// All public methods first - these all need good rust docs
////////////////////////////////////////////////////////////

pub trait UpdateMetadataBuilder: private::CanSetMetadata {
    fn metadata(self, metadata: ModuleConfig<MetadataInit>) -> Self::OutputBuilder {
        self.set_metadata(metadata)
    }
}
impl<B: private::CanSetMetadata> UpdateMetadataBuilder for B {}

pub trait SetAddressReservationBuilder: private::CanSetAddressReservation {
    /// Sets the address reservation
    fn with_address(self, reservation: GlobalAddressReservation) -> Self::OutputBuilder {
        self.set_address(reservation)
    }
}
impl<B: private::CanSetAddressReservation> SetAddressReservationBuilder for B {}

pub trait UpdateAuthBuilder {
    /// Sets the resource to be mintable
    ///
    /// * The first parameter is the access rule which allows minting of the resource.
    /// * The second parameter is the mutability / access rule which controls if and how the access rule can be updated.
    ///
    /// ### Examples
    ///
    /// ```no_run
    /// use radix_engine_interface::mint_roles;
    /// use scrypto::prelude::*;
    ///
    /// # let resource_address = XRD;
    /// // Sets the resource to be mintable with a proof of a specific resource, and this is locked forever.
    /// ResourceBuilder::new_fungible(OwnerRole::None)
    ///    .mint_roles(mint_roles! {
    ///         minter => rule!(require(resource_address));
    ///         minter_updater => rule!(deny_all);
    ///     });
    ///
    /// # let resource_address = XRD;
    /// // Sets the resource to not be mintable, but this is can be changed in future by the second rule
    /// ResourceBuilder::new_fungible(OwnerRole::None)
    ///    .mint_roles(mint_roles! {
    ///         minter => rule!(deny_all);
    ///         minter_updater => rule!(require(resource_address));
    ///    });
    /// ```
    fn mint_roles(self, mint_roles: Option<MintRoles<RoleDefinition>>) -> Self;

    /// Sets the resource to be burnable.
    ///
    /// * The first parameter is the access rule which allows minting of the resource.
    /// * The second parameter is the mutability / access rule which controls if and how the access rule can be updated.
    ///
    /// ### Examples
    ///
    /// ```no_run
    /// use radix_engine_interface::burn_roles;
    /// use scrypto::prelude::*;
    ///
    /// # let resource_address = XRD;
    /// // Sets the resource to be burnable with a proof of a specific resource, and this is locked forever.
    /// ResourceBuilder::new_fungible(OwnerRole::None)
    ///    .burn_roles(burn_roles! {
    ///        burner => rule!(require(resource_address));
    ///        burner_updater => rule!(deny_all);
    ///    });
    ///
    /// # let resource_address = XRD;
    /// // Sets the resource to be freely burnable, but this is can be changed in future by the second rule.
    /// ResourceBuilder::new_fungible(OwnerRole::None)
    ///    .burn_roles(burn_roles! {
    ///        burner => rule!(allow_all);
    ///        burner_updater => rule!(require(resource_address));
    ///    });
    /// ```
    fn burn_roles(self, burn_roles: Option<BurnRoles<RoleDefinition>>) -> Self;

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
    /// # let resource_address = XRD;
    /// // Sets the resource to be recallable with a proof of a specific resource, and this is locked forever.
    /// ResourceBuilder::new_fungible(OwnerRole::None)
    ///    .recall_roles(recall_roles! {
    ///        recaller => rule!(require(resource_address));
    ///        recaller_updater => rule!(deny_all);
    ///    });
    ///
    /// # let resource_address = XRD;
    /// // Sets the resource to not be recallable, but this is can be changed in future by the second rule
    /// ResourceBuilder::new_fungible(OwnerRole::None)
    ///    .recall_roles(recall_roles! {
    ///        recaller => rule!(deny_all);
    ///        recaller_updater => rule!(require(resource_address));
    ///    });
    /// ```
    fn recall_roles(self, recall_roles: Option<RecallRoles<RoleDefinition>>) -> Self;

    /// Sets the resource to have vaults be freezable.
    ///
    /// * The first parameter is the access rule which allows freezing of the vault.
    /// * The second parameter is the mutability / access rule which controls if and how the access rule can be updated.
    ///
    /// ### Examples
    ///
    /// ```no_run
    /// use radix_engine_interface::freeze_roles;
    /// use scrypto::prelude::*;
    ///
    /// # let resource_address = XRD;
    /// // Sets the resource to be freezeable with a proof of a specific resource, and this is locked forever.
    /// ResourceBuilder::new_fungible(OwnerRole::None)
    ///    .freeze_roles(freeze_roles! {
    ///        freezer => rule!(require(resource_address));
    ///        freezer_updater => rule!(deny_all);
    ///    });
    ///
    /// # let resource_address = XRD;
    /// // Sets the resource to not be freezeable, but this is can be changed in future by the second rule
    /// ResourceBuilder::new_fungible(OwnerRole::None)
    ///    .freeze_roles(freeze_roles! {
    ///        freezer => rule!(deny_all);
    ///        freezer_updater => rule!(require(resource_address));
    ///    });
    /// ```
    fn freeze_roles(self, freeze_roles: Option<FreezeRoles<RoleDefinition>>) -> Self;

    /// Sets the role rules of withdrawing from a vault of this resource.
    ///
    /// * The first parameter is the access rule which allows withdrawing from a vault.
    /// * The second parameter is the mutability / access rule which controls if and how the access rule can be updated.
    ///
    /// ### Examples
    ///
    /// ```no_run
    /// use radix_engine_interface::withdraw_roles;
    /// use scrypto::prelude::*;
    ///
    /// # let resource_address = XRD;
    /// // Sets the resource to be withdrawable with a proof of a specific resource, and this is locked forever.
    /// ResourceBuilder::new_fungible(OwnerRole::None)
    ///    .withdraw_roles(withdraw_roles! {
    ///        withdrawer => rule!(require(resource_address));
    ///        withdrawer_updater => rule!(deny_all);
    ///    });
    ///
    /// # let resource_address = XRD;
    /// // Sets the resource to not be withdrawable, but this is can be changed in future by the second rule
    /// ResourceBuilder::new_fungible(OwnerRole::None)
    ///    .withdraw_roles(withdraw_roles! {
    ///        withdrawer => rule!(deny_all);
    ///        withdrawer_updater => rule!(require(resource_address));
    ///    });
    /// ```
    fn withdraw_roles(self, withdraw_roles: Option<WithdrawRoles<RoleDefinition>>) -> Self;

    /// Sets the roles rules of depositing this resource into a vault.
    ///
    /// * The first parameter is the access rule which allows depositing into a vault.
    /// * The second parameter is the mutability / access rule which controls if and how the access rule can be updated.
    ///
    /// ### Examples
    ///
    /// ```no_run
    /// use scrypto::prelude::*;
    ///
    /// # let resource_address = XRD;
    /// // Sets the resource to be depositable with a proof of a specific resource, and this is locked forever.
    /// ResourceBuilder::new_fungible(OwnerRole::None)
    ///    .deposit_roles(deposit_roles! {
    ///        depositor => rule!(require(resource_address));
    ///        depositor_updater => rule!(deny_all);
    ///    });
    ///
    /// # let resource_address = XRD;
    /// // Sets the resource to not be depositable, but this is can be changed in future by the second rule
    /// ResourceBuilder::new_fungible(OwnerRole::None)
    ///    .deposit_roles(deposit_roles! {
    ///        depositor => rule!(deny_all);
    ///        depositor_updater => rule!(require(resource_address));
    ///    });
    /// ```
    fn deposit_roles(self, deposit_roles: Option<DepositRoles<RoleDefinition>>) -> Self;
}

impl UpdateAuthBuilder for InProgressResourceBuilder<FungibleResourceType> {
    fn mint_roles(mut self, mint_roles: Option<MintRoles<RoleDefinition>>) -> Self {
        self.resource_roles.mint_roles = mint_roles;
        self
    }

    fn burn_roles(mut self, burn_roles: Option<BurnRoles<RoleDefinition>>) -> Self {
        self.resource_roles.burn_roles = burn_roles;
        self
    }

    fn recall_roles(mut self, recall_roles: Option<RecallRoles<RoleDefinition>>) -> Self {
        self.resource_roles.recall_roles = recall_roles;
        self
    }

    fn freeze_roles(mut self, freeze_roles: Option<FreezeRoles<RoleDefinition>>) -> Self {
        self.resource_roles.freeze_roles = freeze_roles;
        self
    }

    fn withdraw_roles(mut self, withdraw_roles: Option<WithdrawRoles<RoleDefinition>>) -> Self {
        self.resource_roles.withdraw_roles = withdraw_roles;
        self
    }

    fn deposit_roles(mut self, deposit_roles: Option<DepositRoles<RoleDefinition>>) -> Self {
        self.resource_roles.deposit_roles = deposit_roles;
        self
    }
}

impl<
        T: IsNonFungibleLocalId,
        D: NonFungibleData,
        S: ScryptoCategorize + ScryptoEncode + ScryptoDecode,
    > UpdateAuthBuilder for InProgressResourceBuilder<NonFungibleResourceType<T, D, S>>
{
    fn mint_roles(mut self, mint_roles: Option<MintRoles<RoleDefinition>>) -> Self {
        self.resource_roles.mint_roles = mint_roles;
        self
    }

    fn burn_roles(mut self, burn_roles: Option<BurnRoles<RoleDefinition>>) -> Self {
        self.resource_roles.burn_roles = burn_roles;
        self
    }

    fn recall_roles(mut self, recall_roles: Option<RecallRoles<RoleDefinition>>) -> Self {
        self.resource_roles.recall_roles = recall_roles;
        self
    }

    fn freeze_roles(mut self, freeze_roles: Option<FreezeRoles<RoleDefinition>>) -> Self {
        self.resource_roles.freeze_roles = freeze_roles;
        self
    }

    fn withdraw_roles(mut self, withdraw_roles: Option<WithdrawRoles<RoleDefinition>>) -> Self {
        self.resource_roles.withdraw_roles = withdraw_roles;
        self
    }

    fn deposit_roles(mut self, deposit_roles: Option<DepositRoles<RoleDefinition>>) -> Self {
        self.resource_roles.deposit_roles = deposit_roles;
        self
    }
}

impl<
        T: IsNonFungibleLocalId,
        D: NonFungibleData,
        S: ScryptoCategorize + ScryptoEncode + ScryptoDecode,
    > InProgressResourceBuilder<NonFungibleResourceType<T, D, S>>
{
    /// Sets how each non-fungible's mutable data can be updated.
    ///
    /// * The first parameter is the access rule which allows updating the mutable data of each non-fungible.
    /// * The second parameter is the mutability / access rule which controls if and how the access rule can be updated.
    ///
    /// ### Examples
    ///
    /// ```no_run
    /// use radix_engine_interface::non_fungible_data_update_roles;
    /// use scrypto::prelude::*;
    ///
    /// # let resource_address = XRD;
    ///
    /// #[derive(ScryptoSbor, NonFungibleData)]
    /// struct NFData {
    ///     pub name: String,
    ///     #[mutable]
    ///     pub flag: bool,
    /// }
    /// // Permits the updating of non-fungible mutable data with a proof of a specific resource, and this is locked forever.
    /// ResourceBuilder::new_ruid_non_fungible::<NFData>(OwnerRole::None)
    ///    .non_fungible_data_update_roles(non_fungible_data_update_roles! {
    ///        non_fungible_data_updater => rule!(require(resource_address));
    ///        non_fungible_data_updater_updater => rule!(deny_all);
    ///    });
    ///
    /// # let resource_address = XRD;
    /// // Does not currently permit the updating of non-fungible mutable data, but this is can be changed in future by the second rule.
    /// ResourceBuilder::new_ruid_non_fungible::<NFData>(OwnerRole::None)
    ///    .non_fungible_data_update_roles(non_fungible_data_update_roles! {
    ///        non_fungible_data_updater => rule!(deny_all);
    ///        non_fungible_data_updater_updater => rule!(require(resource_address));
    ///    });
    /// ```
    pub fn non_fungible_data_update_roles(
        mut self,
        non_fungible_data_update_roles: Option<NonFungibleDataUpdateRoles<RoleDefinition>>,
    ) -> Self {
        self.resource_roles.non_fungible_data_update_roles = non_fungible_data_update_roles;
        self
    }
}

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

impl InProgressResourceBuilder<FungibleResourceType> {
    /// Creates the fungible resource with no initial supply.
    ///
    /// The fungible resource's manager is returned.
    /// It is easily convertible to generic resource manager.
    /// One just can use `.into()` method.
    ///
    /// ### Example
    ///
    /// ```no_run
    /// # // Can't run because it tries to call into the radix engine
    /// # use scrypto::prelude::*;
    /// # let address_reservation: GlobalAddressReservation = todo!();
    /// let resource_manager = ResourceBuilder::new_fungible(OwnerRole::None)
    ///     .mint_roles(mint_roles! {
    ///         minter => rule!(deny_all);
    ///         minter_updater => rule!(deny_all);
    ///     })
    ///     .metadata(metadata! {
    ///         init {
    ///             "name" => "Super Admin Badge", locked;
    ///         }
    ///     })
    ///     .with_address(address_reservation)
    ///     .create_with_no_initial_supply();
    /// ```
    pub fn create_with_no_initial_supply(self) -> FungibleResourceManager {
        let metadata = self.metadata_config.unwrap_or_else(|| Default::default());

        let bytes = ScryptoVmV1Api::blueprint_call(
            RESOURCE_PACKAGE,
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
            scrypto_encode(&FungibleResourceManagerCreateInput {
                owner_role: self.owner_role,
                track_total_supply: true,
                divisibility: self.resource_type.divisibility,
                resource_roles: self.resource_roles,
                metadata,
                address_reservation: self.address_reservation,
            })
            .unwrap(),
        );
        scrypto_decode(&bytes).unwrap()
    }
}

impl<
        Y: IsNonFungibleLocalId,
        D: NonFungibleData,
        S: ScryptoCategorize + ScryptoEncode + ScryptoDecode,
    > InProgressResourceBuilder<NonFungibleResourceType<Y, D, S>>
{
    /// Creates the non-fungible resource with no initial supply.
    ///
    /// The non-fungible resource's manager is returned.
    /// It is easily convertible to the generic resource manager.
    /// One just can use `.into()` method.
    ///
    /// ### Example
    ///
    /// ```no_run
    /// # use scrypto::prelude::*;
    /// #[derive(ScryptoSbor, NonFungibleData)]
    /// struct Sandwich {
    ///     name: String,
    ///     with_ham: bool,
    /// }
    ///
    /// let resource_manager: ResourceManager =
    ///     ResourceBuilder::new_ruid_non_fungible::<Sandwich>(OwnerRole::None)
    ///         .mint_roles(mint_roles! {
    ///             minter => rule!(allow_all);
    ///             minter_updater => rule!(deny_all);
    ///         })
    ///         .create_with_no_initial_supply()
    ///         .into();
    /// ```
    pub fn create_with_no_initial_supply(self) -> NonFungibleResourceManager {
        let metadata = self.metadata_config.unwrap_or_else(|| Default::default());

        let bytes = ScryptoVmV1Api::blueprint_call(
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
            scrypto_encode(&NonFungibleResourceManagerCreateGenericInput {
                owner_role: self.owner_role,
                id_type: Y::id_type(),
                track_total_supply: true,
                non_fungible_schema: self.resource_type.0,
                resource_roles: self.resource_roles,
                metadata,
                address_reservation: self.address_reservation,
            })
            .unwrap(),
        );
        scrypto_decode(&bytes).unwrap()
    }
}

impl InProgressResourceBuilder<FungibleResourceType> {
    /// Set the resource's divisibility: the number of digits of precision after the decimal point in its balances.
    ///
    /// * `0` means the resource is not divisible (balances are always whole numbers)
    /// * `18` is the maximum divisibility, and the default.
    ///
    /// ### Examples
    ///
    /// ```no_run
    /// # use scrypto::prelude::*;
    /// #
    /// // Only permits whole-number balances.
    /// ResourceBuilder::new_fungible(OwnerRole::None)
    ///    .divisibility(0);
    ///
    /// // Only permits balances to 3 decimal places.
    /// ResourceBuilder::new_fungible(OwnerRole::None)
    ///    .divisibility(3);
    /// ```
    pub fn divisibility(mut self, divisibility: u8) -> Self {
        assert!(divisibility <= 18);
        self.resource_type = FungibleResourceType { divisibility };
        self
    }
}

impl InProgressResourceBuilder<FungibleResourceType> {
    /// Creates resource with the given initial supply.
    ///
    /// # Example
    /// ```no_run
    /// use scrypto::prelude::*;
    ///
    /// let bucket: FungibleBucket = ResourceBuilder::new_fungible(OwnerRole::None)
    ///     .mint_initial_supply(5);
    /// ```
    pub fn mint_initial_supply<T: Into<Decimal>>(mut self, amount: T) -> FungibleBucket {
        let metadata = self
            .metadata_config
            .take()
            .unwrap_or_else(|| Default::default());

        let bytes = ScryptoVmV1Api::blueprint_call(
            RESOURCE_PACKAGE,
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
            scrypto_encode(&FungibleResourceManagerCreateWithInitialSupplyInput {
                owner_role: self.owner_role,
                track_total_supply: true,
                divisibility: self.resource_type.divisibility,
                resource_roles: self.resource_roles,
                metadata,
                initial_supply: amount.into(),
                address_reservation: self.address_reservation,
            })
            .unwrap(),
        );

        scrypto_decode::<(ResourceAddress, FungibleBucket)>(&bytes)
            .unwrap()
            .1
    }
}

impl<D: NonFungibleData, S: ScryptoCategorize + ScryptoEncode + ScryptoDecode>
    InProgressResourceBuilder<NonFungibleResourceType<StringNonFungibleLocalId, D, S>>
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
    /// let bucket: NonFungibleBucket = ResourceBuilder::new_string_non_fungible::<NFData>(OwnerRole::None)
    ///     .mint_initial_supply([
    ///         ("One".try_into().unwrap(), NFData { name: "NF One".to_owned(), flag: true }),
    ///         ("Two".try_into().unwrap(), NFData { name: "NF Two".to_owned(), flag: true }),
    ///     ]);
    /// ```
    pub fn mint_initial_supply<T>(mut self, entries: T) -> NonFungibleBucket
    where
        T: IntoIterator<Item = (StringNonFungibleLocalId, D)>,
    {
        let metadata = self
            .metadata_config
            .take()
            .unwrap_or_else(|| Default::default());

        let bytes = ScryptoVmV1Api::blueprint_call(
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
            scrypto_encode(
                &NonFungibleResourceManagerCreateWithInitialSupplyGenericInput {
                    owner_role: self.owner_role,
                    track_total_supply: true,
                    id_type: StringNonFungibleLocalId::id_type(),
                    non_fungible_schema: self.resource_type.0,
                    resource_roles: self.resource_roles,
                    metadata,
                    entries: map_entries(entries),
                    address_reservation: self.address_reservation,
                },
            )
            .unwrap(),
        );
        scrypto_decode::<(ResourceAddress, NonFungibleBucket)>(&bytes)
            .unwrap()
            .1
    }
}

impl<D: NonFungibleData, S: ScryptoCategorize + ScryptoEncode + ScryptoDecode>
    InProgressResourceBuilder<NonFungibleResourceType<IntegerNonFungibleLocalId, D, S>>
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
    /// let bucket: NonFungibleBucket = ResourceBuilder::new_integer_non_fungible(OwnerRole::None)
    ///     .mint_initial_supply([
    ///         (1u64.into(), NFData { name: "NF One".to_owned(), flag: true }),
    ///         (2u64.into(), NFData { name: "NF Two".to_owned(), flag: true }),
    ///     ]);
    /// ```
    pub fn mint_initial_supply<T>(mut self, entries: T) -> NonFungibleBucket
    where
        T: IntoIterator<Item = (IntegerNonFungibleLocalId, D)>,
    {
        let metadata = self
            .metadata_config
            .take()
            .unwrap_or_else(|| Default::default());

        let bytes = ScryptoVmV1Api::blueprint_call(
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
            scrypto_encode(
                &NonFungibleResourceManagerCreateWithInitialSupplyGenericInput {
                    owner_role: self.owner_role,
                    track_total_supply: true,
                    id_type: IntegerNonFungibleLocalId::id_type(),
                    non_fungible_schema: self.resource_type.0,
                    resource_roles: self.resource_roles,
                    metadata,
                    entries: map_entries(entries),
                    address_reservation: self.address_reservation,
                },
            )
            .unwrap(),
        );
        scrypto_decode::<(ResourceAddress, NonFungibleBucket)>(&bytes)
            .unwrap()
            .1
    }
}

impl<D: NonFungibleData, S: ScryptoCategorize + ScryptoEncode + ScryptoDecode>
    InProgressResourceBuilder<NonFungibleResourceType<BytesNonFungibleLocalId, D, S>>
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
    /// let bucket: NonFungibleBucket = ResourceBuilder::new_bytes_non_fungible::<NFData>(OwnerRole::None)
    ///     .mint_initial_supply([
    ///         (vec![1u8].try_into().unwrap(), NFData { name: "NF One".to_owned(), flag: true }),
    ///         (vec![2u8].try_into().unwrap(), NFData { name: "NF Two".to_owned(), flag: true }),
    ///     ]);
    /// ```
    pub fn mint_initial_supply<T>(mut self, entries: T) -> NonFungibleBucket
    where
        T: IntoIterator<Item = (BytesNonFungibleLocalId, D)>,
    {
        let metadata = self
            .metadata_config
            .take()
            .unwrap_or_else(|| Default::default());

        let bytes = ScryptoVmV1Api::blueprint_call(
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
            scrypto_encode(
                &NonFungibleResourceManagerCreateWithInitialSupplyGenericInput {
                    owner_role: self.owner_role,
                    id_type: BytesNonFungibleLocalId::id_type(),
                    track_total_supply: true,
                    non_fungible_schema: self.resource_type.0,
                    resource_roles: self.resource_roles,
                    metadata,
                    entries: map_entries(entries),
                    address_reservation: self.address_reservation,
                },
            )
            .unwrap(),
        );
        scrypto_decode::<(ResourceAddress, NonFungibleBucket)>(&bytes)
            .unwrap()
            .1
    }
}

impl<D: NonFungibleData, S: ScryptoCategorize + ScryptoEncode + ScryptoDecode>
    InProgressResourceBuilder<NonFungibleResourceType<RUIDNonFungibleLocalId, D, S>>
{
    /// Creates the RUID non-fungible resource, and mints an individual non-fungible for each piece of data provided.
    ///
    /// The system automatically generates a new RUID `NonFungibleLocalId` for each non-fungible,
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
    /// let bucket: NonFungibleBucket = ResourceBuilder::new_ruid_non_fungible::<NFData>(OwnerRole::None)
    ///     .mint_initial_supply([
    ///         (NFData { name: "NF One".to_owned(), flag: true }),
    ///         (NFData { name: "NF Two".to_owned(), flag: true }),
    ///     ]);
    /// ```
    pub fn mint_initial_supply<T>(mut self, entries: T) -> NonFungibleBucket
    where
        T: IntoIterator<Item = D>,
        D: ScryptoEncode,
    {
        let metadata = self
            .metadata_config
            .take()
            .unwrap_or_else(|| Default::default());

        let bytes = ScryptoVmV1Api::blueprint_call(
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_RUID_WITH_INITIAL_SUPPLY_IDENT,
            scrypto_encode(
                &NonFungibleResourceManagerCreateRuidWithInitialSupplyGenericInput {
                    owner_role: self.owner_role,
                    non_fungible_schema: self.resource_type.0,
                    track_total_supply: true,
                    resource_roles: self.resource_roles,
                    metadata,
                    entries: entries.into_iter().map(|data| (data,)).collect(),
                    address_reservation: self.address_reservation,
                },
            )
            .unwrap(),
        );
        scrypto_decode::<(ResourceAddress, NonFungibleBucket)>(&bytes)
            .unwrap()
            .1
    }
}

///////////////////////////////////
/// PRIVATE TRAIT IMPLEMENTATIONS
/// These don't need good rust docs
///////////////////////////////////

fn map_entries<T: IntoIterator<Item = (Y, V)>, V: NonFungibleData, Y: IsNonFungibleLocalId>(
    entries: T,
) -> IndexMap<NonFungibleLocalId, (V,)> {
    entries
        .into_iter()
        .map(|(id, data)| (id.into(), (data,)))
        .collect()
}

impl<T: AnyResourceType> private::CanSetMetadata for InProgressResourceBuilder<T> {
    type OutputBuilder = Self;

    fn set_metadata(mut self, metadata: ModuleConfig<MetadataInit>) -> Self::OutputBuilder {
        self.metadata_config = Some(metadata);
        self
    }
}

impl<T: AnyResourceType> private::CanSetAddressReservation for InProgressResourceBuilder<T> {
    type OutputBuilder = Self;

    fn set_address(mut self, address_reservation: GlobalAddressReservation) -> Self::OutputBuilder {
        self.address_reservation = Some(address_reservation);
        self
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
    use radix_common::types::NonFungibleGlobalId;

    pub trait CanSetMetadata: Sized {
        type OutputBuilder;

        fn set_metadata(self, metadata: ModuleConfig<MetadataInit>) -> Self::OutputBuilder;
    }

    pub trait CanSetAddressReservation: Sized {
        type OutputBuilder;

        fn set_address(self, address_reservation: GlobalAddressReservation) -> Self::OutputBuilder;
    }

    pub trait CanAddOwner: Sized {
        type OutputBuilder;

        fn set_owner(self, owner_badge: NonFungibleGlobalId) -> Self::OutputBuilder;
    }
}
