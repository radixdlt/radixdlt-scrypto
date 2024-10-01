use crate::modules::HasRoleAssignment;
use crate::prelude::{Global, ObjectStub, ObjectStubHandle, ScryptoEncode};
use crate::*;
use core::ops::Deref;
use radix_common::data::scrypto::model::*;
use radix_common::data::scrypto::well_known_scrypto_custom_types::resource_address_type_data;
use radix_common::data::scrypto::well_known_scrypto_custom_types::RESOURCE_ADDRESS_TYPE;
use radix_common::data::scrypto::*;
use radix_common::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_common::math::Decimal;
use radix_common::prelude::*;
use radix_common::traits::NonFungibleData;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::object_modules::metadata::{
    METADATA_SETTER_ROLE, METADATA_SETTER_UPDATER_ROLE,
};
use scrypto::component::HasStub;

//=============
// Traits
//=============

pub trait ScryptoResourceManager {
    fn set_mintable(&self, access_rule: AccessRule);

    fn set_burnable(&self, access_rule: AccessRule);

    fn set_withdrawable(&self, access_rule: AccessRule);

    fn set_depositable(&self, access_rule: AccessRule);

    fn set_recallable(&self, access_rule: AccessRule);

    fn set_freezeable(&self, access_rule: AccessRule);

    fn lock_mintable(&self);

    fn lock_burnable(&self);

    fn lock_withdrawable(&self);

    fn lock_depositable(&self);

    fn lock_recallable(&self);

    fn lock_freezeable(&self);

    fn set_updatable_metadata(&self, access_rule: AccessRule);

    fn lock_updatable_metadata(&self);
}

pub trait ScryptoResourceManagerStub {
    type VaultType;
    type BucketType;

    fn create_empty_vault(&self) -> Self::VaultType;

    fn create_empty_bucket(&self) -> Self::BucketType;

    fn resource_type(&self) -> ResourceType;

    fn total_supply(&self) -> Option<Decimal>;

    fn burn<B: Into<Bucket>>(&self, bucket: B);

    fn amount_for_withdrawal(
        &self,
        request_amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
    ) -> Decimal;
}

//=================
// ResourceManager
//=================

#[derive(Debug, Clone, Copy, Eq, PartialEq, ScryptoEncode, ScryptoCategorize, Hash)]
#[sbor(transparent)]
pub struct ResourceManager(Global<ResourceManagerStub>);

impl Describe<ScryptoCustomTypeKind> for ResourceManager {
    const TYPE_ID: RustTypeId = RustTypeId::WellKnown(RESOURCE_ADDRESS_TYPE);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
        resource_address_type_data()
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for ResourceManager {
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        let value =
            <Global<ResourceManagerStub>>::decode_body_with_value_kind(decoder, value_kind)?;
        if value.handle().as_node_id().is_global_resource_manager() {
            Ok(Self(value))
        } else {
            Err(DecodeError::InvalidCustomValue)
        }
    }
}

impl From<ResourceAddress> for ResourceManager {
    fn from(value: ResourceAddress) -> Self {
        let stub = ResourceManagerStub::new(ObjectStubHandle::Global(value.into()));
        Self(Global(stub))
    }
}

impl Into<GlobalAddress> for ResourceManager {
    fn into(self) -> GlobalAddress {
        GlobalAddress::new_or_panic(self.0 .0 .0.as_node_id().0)
    }
}

impl Deref for ResourceManager {
    type Target = Global<ResourceManagerStub>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ResourceManager {
    pub const fn from_address(address: ResourceAddress) -> Self {
        let stub = ResourceManagerStub(ObjectStubHandle::Global(GlobalAddress::new_or_panic(
            address.into_node_id().0,
        )));
        Self(Global(stub))
    }
}

impl ScryptoResourceManager for ResourceManager {
    fn set_mintable(&self, access_rule: AccessRule) {
        self.0.set_role(MINTER_ROLE, access_rule);
    }

    fn set_burnable(&self, access_rule: AccessRule) {
        self.0.set_role(BURNER_ROLE, access_rule);
    }

    fn set_withdrawable(&self, access_rule: AccessRule) {
        self.0.set_role(WITHDRAWER_ROLE, access_rule);
    }

    fn set_depositable(&self, access_rule: AccessRule) {
        self.0.set_role(DEPOSITOR_ROLE, access_rule);
    }

    fn set_recallable(&self, access_rule: AccessRule) {
        self.0.set_role(RECALLER_ROLE, access_rule);
    }

    fn set_freezeable(&self, access_rule: AccessRule) {
        self.0.set_role(FREEZER_ROLE, access_rule);
    }

    fn lock_mintable(&self) {
        self.0.set_role(MINTER_UPDATER_ROLE, AccessRule::DenyAll);
    }

    fn lock_burnable(&self) {
        self.0.set_role(BURNER_UPDATER_ROLE, AccessRule::DenyAll);
    }

    fn lock_withdrawable(&self) {
        self.0
            .set_role(WITHDRAWER_UPDATER_ROLE, AccessRule::DenyAll);
    }

    fn lock_depositable(&self) {
        self.0.set_role(DEPOSITOR_UPDATER_ROLE, AccessRule::DenyAll);
    }

    fn lock_recallable(&self) {
        self.0.set_role(RECALLER_UPDATER_ROLE, AccessRule::DenyAll);
    }

    fn lock_freezeable(&self) {
        self.0.set_role(FREEZER_UPDATER_ROLE, AccessRule::DenyAll);
    }

    fn set_updatable_metadata(&self, access_rule: AccessRule) {
        self.0.set_metadata_role(METADATA_SETTER_ROLE, access_rule);
    }

    fn lock_updatable_metadata(&self) {
        self.0
            .set_metadata_role(METADATA_SETTER_UPDATER_ROLE, AccessRule::DenyAll);
    }
}

impl ResourceManager {
    #[deprecated = "Use NonFungibleResourceManager::set_updatable_non_fungible_data instead"]
    pub fn set_updatable_non_fungible_data(&self, access_rule: AccessRule) {
        self.0.set_role(NON_FUNGIBLE_DATA_UPDATER_ROLE, access_rule);
    }

    #[deprecated = "Use NonFungibleResourceManager::lock_updatable_non_fungible_data instead"]
    pub fn lock_updatable_non_fungible_data(&self) {
        self.0
            .set_role(NON_FUNGIBLE_DATA_UPDATER_UPDATER_ROLE, AccessRule::DenyAll);
    }
}

impl HasStub for ResourceManagerStub {
    type Stub = Self;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ResourceManagerStub(pub ObjectStubHandle);

impl ObjectStub for ResourceManagerStub {
    type AddressType = ResourceAddress;

    fn new(handle: ObjectStubHandle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &ObjectStubHandle {
        &self.0
    }
}

impl ScryptoResourceManagerStub for ResourceManagerStub {
    type VaultType = Vault;
    type BucketType = Bucket;

    fn create_empty_vault(&self) -> Self::VaultType {
        self.call(
            RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT,
            &ResourceManagerCreateEmptyVaultInput {},
        )
    }

    fn create_empty_bucket(&self) -> Self::BucketType {
        self.call(
            RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT,
            &ResourceManagerCreateEmptyBucketInput {},
        )
    }

    fn resource_type(&self) -> ResourceType {
        self.call(
            RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT,
            &ResourceManagerGetResourceTypeInput {},
        )
    }

    fn total_supply(&self) -> Option<Decimal> {
        self.call(
            RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT,
            &ResourceManagerGetTotalSupplyInput {},
        )
    }

    fn burn<B: Into<Bucket>>(&self, bucket: B) {
        self.call(
            RESOURCE_MANAGER_BURN_IDENT,
            &ResourceManagerBurnInput {
                bucket: bucket.into(),
            },
        )
    }

    fn amount_for_withdrawal(
        &self,
        request_amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
    ) -> Decimal {
        self.call(
            RESOURCE_MANAGER_GET_AMOUNT_FOR_WITHDRAWAL_IDENT,
            &ResourceManagerGetAmountForWithdrawalInput {
                request_amount,
                withdraw_strategy,
            },
        )
    }
}

impl ResourceManagerStub {
    /// Mints fungible resources
    #[deprecated = "Use FungibleResourceManagerStub::mint instead"]
    pub fn mint<T: Into<Decimal>>(&self, amount: T) -> Bucket {
        self.call(
            FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
            &FungibleResourceManagerMintInput {
                amount: amount.into(),
            },
        )
    }

    #[deprecated = "Use NonFungibleResourceManagerStub::non_fungible_exists instead"]
    pub fn non_fungible_exists(&self, id: &NonFungibleLocalId) -> bool {
        self.call(
            NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT,
            &NonFungibleResourceManagerExistsInput { id: id.clone() },
        )
    }

    /// Mints non-fungible resources
    #[deprecated = "Use NonFungibleResourceManagerStub::mint_non_fungible instead"]
    pub fn mint_non_fungible<T: NonFungibleData>(
        &self,
        id: &NonFungibleLocalId,
        data: T,
    ) -> Bucket {
        let mut entries = index_map_new();
        entries.insert(id.clone(), (data,));
        self.call(
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
            &NonFungibleResourceManagerMintGenericInput { entries },
        )
    }

    /// Mints ruid non-fungible resources
    #[deprecated = "Use NonFungibleResourceManagerStub::mint_ruid_non_fungible instead"]
    pub fn mint_ruid_non_fungible<T: NonFungibleData>(&self, data: T) -> Bucket {
        let mut entries = Vec::new();
        entries.push((data,));

        self.call(
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_RUID_IDENT,
            &NonFungibleResourceManagerMintRuidGenericInput { entries },
        )
    }

    /// Returns the data of a non-fungible unit, both the immutable and mutable parts.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible resource or the specified non-fungible is not found.
    #[deprecated = "Use NonFungibleResourceManagerStub::get_non_fungible_data instead"]
    pub fn get_non_fungible_data<T: NonFungibleData>(&self, id: &NonFungibleLocalId) -> T {
        self.call(
            NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT,
            &NonFungibleResourceManagerGetNonFungibleInput { id: id.clone() },
        )
    }

    /// Updates the mutable part of a non-fungible unit.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible resource or the specified non-fungible is not found.
    #[deprecated = "Use NonFungibleResourceManagerStub::update_non_fungible_data instead"]
    pub fn update_non_fungible_data<D: ScryptoEncode>(
        &self,
        id: &NonFungibleLocalId,
        field_name: &str,
        new_data: D,
    ) {
        self.call(
            NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT,
            &NonFungibleResourceManagerUpdateDataInput {
                id: id.clone(),
                field_name: field_name.to_string(),
                data: scrypto_decode(&scrypto_encode(&new_data).unwrap()).unwrap(),
            },
        )
    }
}

//=========================
// FungibleResourceManager
//=========================

#[derive(Debug, Clone, Copy, Eq, PartialEq, ScryptoEncode, ScryptoCategorize, Hash)]
#[sbor(transparent)]
pub struct FungibleResourceManager(Global<FungibleResourceManagerStub>);

impl Describe<ScryptoCustomTypeKind> for FungibleResourceManager {
    const TYPE_ID: RustTypeId = RustTypeId::WellKnown(RESOURCE_ADDRESS_TYPE);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
        resource_address_type_data()
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D>
    for FungibleResourceManager
{
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        let value = <Global<FungibleResourceManagerStub>>::decode_body_with_value_kind(
            decoder, value_kind,
        )?;
        if value
            .handle()
            .as_node_id()
            .is_global_fungible_resource_manager()
        {
            Ok(Self(value))
        } else {
            Err(DecodeError::InvalidCustomValue)
        }
    }
}

impl From<FungibleResourceManager> for ResourceManager {
    fn from(value: FungibleResourceManager) -> Self {
        let rm: ResourceManagerStub = value.0 .0.into();
        ResourceManager(Global(rm))
    }
}

impl From<ResourceAddress> for FungibleResourceManager {
    fn from(value: ResourceAddress) -> Self {
        let stub = FungibleResourceManagerStub::new(ObjectStubHandle::Global(value.into()));
        Self(Global(stub))
    }
}

impl Into<GlobalAddress> for FungibleResourceManager {
    fn into(self) -> GlobalAddress {
        GlobalAddress::new_or_panic(self.0 .0 .0 .0.as_node_id().0)
    }
}

impl Deref for FungibleResourceManager {
    type Target = Global<FungibleResourceManagerStub>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ScryptoResourceManager for FungibleResourceManager {
    fn set_mintable(&self, access_rule: AccessRule) {
        ResourceManager::from(*self).set_mintable(access_rule)
    }

    fn set_burnable(&self, access_rule: AccessRule) {
        ResourceManager::from(*self).set_burnable(access_rule)
    }

    fn set_withdrawable(&self, access_rule: AccessRule) {
        ResourceManager::from(*self).set_withdrawable(access_rule)
    }

    fn set_depositable(&self, access_rule: AccessRule) {
        ResourceManager::from(*self).set_depositable(access_rule)
    }

    fn set_recallable(&self, access_rule: AccessRule) {
        ResourceManager::from(*self).set_recallable(access_rule)
    }

    fn set_freezeable(&self, access_rule: AccessRule) {
        ResourceManager::from(*self).set_freezeable(access_rule)
    }

    fn lock_mintable(&self) {
        ResourceManager::from(*self).lock_mintable()
    }

    fn lock_burnable(&self) {
        ResourceManager::from(*self).lock_burnable()
    }

    fn lock_withdrawable(&self) {
        ResourceManager::from(*self).lock_withdrawable()
    }

    fn lock_depositable(&self) {
        ResourceManager::from(*self).lock_depositable()
    }

    fn lock_recallable(&self) {
        ResourceManager::from(*self).lock_recallable()
    }

    fn lock_freezeable(&self) {
        ResourceManager::from(*self).lock_freezeable()
    }

    fn set_updatable_metadata(&self, access_rule: AccessRule) {
        ResourceManager::from(*self).set_updatable_metadata(access_rule)
    }

    fn lock_updatable_metadata(&self) {
        ResourceManager::from(*self).lock_updatable_metadata()
    }
}

impl HasStub for FungibleResourceManagerStub {
    type Stub = Self;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct FungibleResourceManagerStub(pub ResourceManagerStub);

impl From<FungibleResourceManagerStub> for ResourceManagerStub {
    fn from(value: FungibleResourceManagerStub) -> Self {
        value.0
    }
}

impl ObjectStub for FungibleResourceManagerStub {
    type AddressType = ResourceAddress;

    fn new(handle: ObjectStubHandle) -> Self {
        assert!(
            handle.as_node_id().is_global_fungible_resource_manager(),
            "Expected a fungible resource"
        );

        Self(ResourceManagerStub::new(handle))
    }

    fn handle(&self) -> &ObjectStubHandle {
        &self.0 .0
    }
}

impl ScryptoResourceManagerStub for FungibleResourceManagerStub {
    type VaultType = FungibleVault;
    type BucketType = FungibleBucket;

    fn create_empty_vault(&self) -> Self::VaultType {
        FungibleVault(self.0.create_empty_vault())
    }

    fn create_empty_bucket(&self) -> Self::BucketType {
        FungibleBucket(self.0.create_empty_bucket())
    }

    fn resource_type(&self) -> ResourceType {
        self.0.resource_type()
    }

    fn total_supply(&self) -> Option<Decimal> {
        self.0.total_supply()
    }

    fn burn<B: Into<Bucket>>(&self, bucket: B) {
        self.0.burn(bucket)
    }

    fn amount_for_withdrawal(
        &self,
        request_amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
    ) -> Decimal {
        self.0
            .amount_for_withdrawal(request_amount, withdraw_strategy)
    }
}

impl FungibleResourceManagerStub {
    /// Mints fungible resources
    ///
    /// The fungible bucket is returned.
    /// One can easily convert it to the generic bucket using `.into()` method.
    ///
    /// ### Example
    /// ```no_run
    /// # use scrypto::prelude::*;
    /// let resource_manager = ResourceBuilder::new_fungible(OwnerRole::None)
    ///     .divisibility(DIVISIBILITY_MAXIMUM)
    ///     .create_with_no_initial_supply();
    /// let bucket = resource_manager.mint(1);
    /// ```
    pub fn mint<T: Into<Decimal>>(&self, amount: T) -> FungibleBucket {
        self.call(
            FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
            &FungibleResourceManagerMintInput {
                amount: amount.into(),
            },
        )
    }
}

//============================
// NonFungibleResourceManager
//============================

#[derive(Debug, Clone, Copy, Eq, PartialEq, ScryptoEncode, ScryptoCategorize, Hash)]
#[sbor(transparent)]
pub struct NonFungibleResourceManager(Global<NonFungibleResourceManagerStub>);

impl Describe<ScryptoCustomTypeKind> for NonFungibleResourceManager {
    const TYPE_ID: RustTypeId = RustTypeId::WellKnown(RESOURCE_ADDRESS_TYPE);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
        resource_address_type_data()
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D>
    for NonFungibleResourceManager
{
    #[inline]
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        let value = <Global<NonFungibleResourceManagerStub>>::decode_body_with_value_kind(
            decoder, value_kind,
        )?;
        if value
            .handle()
            .as_node_id()
            .is_global_non_fungible_resource_manager()
        {
            Ok(Self(value))
        } else {
            Err(DecodeError::InvalidCustomValue)
        }
    }
}

impl From<NonFungibleResourceManager> for ResourceManager {
    fn from(value: NonFungibleResourceManager) -> Self {
        let rm: ResourceManagerStub = value.0 .0.into();
        ResourceManager(Global(rm))
    }
}

impl From<ResourceAddress> for NonFungibleResourceManager {
    fn from(value: ResourceAddress) -> Self {
        let stub = NonFungibleResourceManagerStub::new(ObjectStubHandle::Global(value.into()));
        Self(Global(stub))
    }
}

impl Into<GlobalAddress> for NonFungibleResourceManager {
    fn into(self) -> GlobalAddress {
        GlobalAddress::new_or_panic(self.0 .0 .0 .0.as_node_id().0)
    }
}

impl Deref for NonFungibleResourceManager {
    type Target = Global<NonFungibleResourceManagerStub>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ScryptoResourceManager for NonFungibleResourceManager {
    fn set_mintable(&self, access_rule: AccessRule) {
        ResourceManager::from(*self).set_mintable(access_rule)
    }

    fn set_burnable(&self, access_rule: AccessRule) {
        ResourceManager::from(*self).set_burnable(access_rule)
    }

    fn set_withdrawable(&self, access_rule: AccessRule) {
        ResourceManager::from(*self).set_withdrawable(access_rule)
    }

    fn set_depositable(&self, access_rule: AccessRule) {
        ResourceManager::from(*self).set_depositable(access_rule)
    }

    fn set_recallable(&self, access_rule: AccessRule) {
        ResourceManager::from(*self).set_recallable(access_rule)
    }

    fn set_freezeable(&self, access_rule: AccessRule) {
        ResourceManager::from(*self).set_freezeable(access_rule)
    }

    fn lock_mintable(&self) {
        ResourceManager::from(*self).lock_mintable()
    }

    fn lock_burnable(&self) {
        ResourceManager::from(*self).lock_burnable()
    }

    fn lock_withdrawable(&self) {
        ResourceManager::from(*self).lock_withdrawable()
    }

    fn lock_depositable(&self) {
        ResourceManager::from(*self).lock_depositable()
    }

    fn lock_recallable(&self) {
        ResourceManager::from(*self).lock_recallable()
    }

    fn lock_freezeable(&self) {
        ResourceManager::from(*self).lock_freezeable()
    }

    fn set_updatable_metadata(&self, access_rule: AccessRule) {
        ResourceManager::from(*self).set_updatable_metadata(access_rule)
    }

    fn lock_updatable_metadata(&self) {
        ResourceManager::from(*self).lock_updatable_metadata()
    }
}

impl NonFungibleResourceManager {
    pub fn set_updatable_non_fungible_data(&self, access_rule: AccessRule) {
        self.0.set_role(NON_FUNGIBLE_DATA_UPDATER_ROLE, access_rule);
    }

    pub fn lock_updatable_non_fungible_data(&self) {
        self.0
            .set_role(NON_FUNGIBLE_DATA_UPDATER_UPDATER_ROLE, AccessRule::DenyAll);
    }
}

impl HasStub for NonFungibleResourceManagerStub {
    type Stub = Self;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct NonFungibleResourceManagerStub(pub ResourceManagerStub);

impl From<NonFungibleResourceManagerStub> for ResourceManagerStub {
    fn from(value: NonFungibleResourceManagerStub) -> Self {
        value.0
    }
}

impl ObjectStub for NonFungibleResourceManagerStub {
    type AddressType = ResourceAddress;

    fn new(handle: ObjectStubHandle) -> Self {
        assert!(
            handle
                .as_node_id()
                .is_global_non_fungible_resource_manager(),
            "Expected a non-fungible resource"
        );

        Self(ResourceManagerStub::new(handle))
    }

    fn handle(&self) -> &ObjectStubHandle {
        &self.0 .0
    }
}

impl ScryptoResourceManagerStub for NonFungibleResourceManagerStub {
    type VaultType = NonFungibleVault;
    type BucketType = NonFungibleBucket;

    fn create_empty_vault(&self) -> Self::VaultType {
        NonFungibleVault(self.0.create_empty_vault())
    }

    fn create_empty_bucket(&self) -> Self::BucketType {
        NonFungibleBucket(self.0.create_empty_bucket())
    }

    fn resource_type(&self) -> ResourceType {
        self.0.resource_type()
    }

    fn total_supply(&self) -> Option<Decimal> {
        self.0.total_supply()
    }

    fn burn<B: Into<Bucket>>(&self, bucket: B) {
        self.0.burn(bucket)
    }

    fn amount_for_withdrawal(
        &self,
        request_amount: Decimal,
        withdraw_strategy: WithdrawStrategy,
    ) -> Decimal {
        self.0
            .amount_for_withdrawal(request_amount, withdraw_strategy)
    }
}

impl NonFungibleResourceManagerStub {
    pub fn non_fungible_exists(&self, id: &NonFungibleLocalId) -> bool {
        self.call(
            NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT,
            &NonFungibleResourceManagerExistsInput { id: id.clone() },
        )
    }

    /// Mints non-fungible resources
    ///
    /// The non-fungible bucket is returned.
    /// One can easily convert it to the generic bucket using `.into()` method.
    ///
    /// ### Example
    /// ```no_run
    /// # // Can't run because it tries to call into the radix engine
    /// # use scrypto::prelude::*;
    /// #[derive(ScryptoSbor, NonFungibleData)]
    /// struct Sandwich {
    ///     name: String,
    ///     with_ham: bool,
    /// }
    ///
    /// let resource_manager = ResourceBuilder::new_integer_non_fungible::<Sandwich>(
    ///     OwnerRole::None,
    /// )
    /// .create_with_no_initial_supply();
    ///
    /// let bucket = resource_manager.mint_non_fungible(
    ///     &NonFungibleLocalId::integer(0),
    ///     Sandwich {
    ///         name: "IntegerNftSandwich".to_owned(),
    ///         with_ham: false,
    ///     },
    /// );
    /// ```
    pub fn mint_non_fungible<T: NonFungibleData>(
        &self,
        id: &NonFungibleLocalId,
        data: T,
    ) -> NonFungibleBucket {
        let mut entries = index_map_new();
        entries.insert(id.clone(), (data,));
        self.call(
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
            &NonFungibleResourceManagerMintGenericInput { entries },
        )
    }

    /// Mints ruid non-fungible resources
    ///
    /// The non-fungible bucket is returned.
    /// One can easily convert it to the generic bucket using `.into()` method.
    ///
    /// ### Example
    /// ```no_run
    /// # // Can't run because it tries to call into the radix engine
    /// # use scrypto::prelude::*;
    /// #[derive(ScryptoSbor, NonFungibleData)]
    /// struct Sandwich {
    ///     name: String,
    ///     with_ham: bool,
    /// }
    ///
    /// let resource_manager = ResourceBuilder::new_ruid_non_fungible::<Sandwich>(
    ///     OwnerRole::None,
    /// )
    /// .create_with_no_initial_supply();
    ///
    /// let bucket = resource_manager.mint_ruid_non_fungible(
    ///     Sandwich {
    ///         name: "RuidNftSandwich".to_owned(),
    ///         with_ham: false,
    ///     },
    /// );
    /// ```
    pub fn mint_ruid_non_fungible<T: NonFungibleData>(&self, data: T) -> NonFungibleBucket {
        let mut entries = Vec::new();
        entries.push((data,));

        self.call(
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_RUID_IDENT,
            &NonFungibleResourceManagerMintRuidGenericInput { entries },
        )
    }

    /// Returns the data of a non-fungible unit, both the immutable and mutable parts.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible resource or the specified non-fungible is not found.
    pub fn get_non_fungible_data<T: NonFungibleData>(&self, id: &NonFungibleLocalId) -> T {
        self.call(
            NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT,
            &NonFungibleResourceManagerGetNonFungibleInput { id: id.clone() },
        )
    }

    /// Updates the mutable part of a non-fungible unit.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible resource or the specified non-fungible is not found.
    pub fn update_non_fungible_data<D: ScryptoEncode>(
        &self,
        id: &NonFungibleLocalId,
        field_name: &str,
        new_data: D,
    ) {
        self.call(
            NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT,
            &NonFungibleResourceManagerUpdateDataInput {
                id: id.clone(),
                field_name: field_name.to_string(),
                data: scrypto_decode(&scrypto_encode(&new_data).unwrap()).unwrap(),
            },
        )
    }
}
