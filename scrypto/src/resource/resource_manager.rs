use crate::component::*;
use crate::modules::HasRoleAssignment;
use crate::prelude::{ObjectStub, ObjectStubHandle, ScryptoEncode};
use crate::*;
use core::ops::Deref;
use module_blueprints_interface::metadata::{METADATA_SETTER_ROLE, METADATA_SETTER_UPDATER_ROLE};
use native_blueprints_interface::resource::*;
use radix_engine_common::data::scrypto::well_known_scrypto_custom_types::resource_address_type_data;
use radix_engine_common::data::scrypto::well_known_scrypto_custom_types::RESOURCE_ADDRESS_TYPE;
use radix_engine_common::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_common::math::Decimal;
use radix_engine_common::prelude::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq, ScryptoEncode, ScryptoDecode, ScryptoCategorize)]
#[sbor(transparent)]
pub struct ResourceManager(Global<ResourceManagerStub>);

impl Describe<ScryptoCustomTypeKind> for ResourceManager {
    const TYPE_ID: RustTypeId = RustTypeId::WellKnown(RESOURCE_ADDRESS_TYPE);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
        resource_address_type_data()
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

    pub fn set_mintable(&self, access_rule: AccessRule) {
        self.0.set_role(MINTER_ROLE, access_rule);
    }

    pub fn set_burnable(&self, access_rule: AccessRule) {
        self.0.set_role(RESOURCE_MANAGER_BURN_IDENT, access_rule);
    }

    pub fn set_withdrawable(&self, access_rule: AccessRule) {
        self.0.set_role(WITHDRAWER_ROLE, access_rule);
    }

    pub fn set_depositable(&self, access_rule: AccessRule) {
        self.0.set_role(DEPOSITOR_ROLE, access_rule);
    }

    pub fn set_recallable(&self, access_rule: AccessRule) {
        self.0.set_role(RECALLER_ROLE, access_rule);
    }

    pub fn set_freezeable(&self, access_rule: AccessRule) {
        self.0.set_role(FREEZER_ROLE, access_rule);
    }

    pub fn set_updatable_non_fungible_data(&self, access_rule: AccessRule) {
        self.0.set_role(NON_FUNGIBLE_DATA_UPDATER_ROLE, access_rule);
    }

    pub fn lock_mintable(&self) {
        self.0.set_role(MINTER_UPDATER_ROLE, AccessRule::DenyAll);
    }

    pub fn lock_burnable(&self) {
        self.0.set_role(BURNER_UPDATER_ROLE, AccessRule::DenyAll);
    }

    pub fn lock_updatable_non_fungible_data(&self) {
        self.0
            .set_role(NON_FUNGIBLE_DATA_UPDATER_UPDATER_ROLE, AccessRule::DenyAll);
    }

    pub fn lock_withdrawable(&self) {
        self.0
            .set_role(WITHDRAWER_UPDATER_ROLE, AccessRule::DenyAll);
    }

    pub fn lock_depositable(&self) {
        self.0.set_role(DEPOSITOR_UPDATER_ROLE, AccessRule::DenyAll);
    }

    pub fn lock_recallable(&self) {
        self.0.set_role(RECALLER_UPDATER_ROLE, AccessRule::DenyAll);
    }

    pub fn lock_freezeable(&self) {
        self.0.set_role(FREEZER_UPDATER_ROLE, AccessRule::DenyAll);
    }

    pub fn set_updatable_metadata(&self, access_rule: AccessRule) {
        self.0.set_metadata_role(METADATA_SETTER_ROLE, access_rule);
    }

    pub fn lock_updatable_metadata(&self) {
        self.0
            .set_metadata_role(METADATA_SETTER_UPDATER_ROLE, AccessRule::DenyAll);
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

impl ResourceManagerStub {
    pub fn create_empty_vault(&self) -> Vault {
        self.call(
            RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT,
            &ResourceManagerCreateEmptyVaultInput {},
        )
    }

    pub fn create_empty_bucket(&self) -> Bucket {
        self.call(
            RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT,
            &ResourceManagerCreateEmptyBucketInput {},
        )
    }

    pub fn resource_type(&self) -> ResourceType {
        self.call(
            RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT,
            &ResourceManagerGetResourceTypeInput {},
        )
    }

    pub fn total_supply(&self) -> Option<Decimal> {
        self.call(
            RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT,
            &ResourceManagerGetTotalSupplyInput {},
        )
    }

    pub fn non_fungible_exists(&self, id: &NonFungibleLocalId) -> bool {
        self.call(
            NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT,
            &NonFungibleResourceManagerExistsInput { id: id.clone() },
        )
    }

    pub fn burn<B: Into<Bucket>>(&self, bucket: B) {
        self.call(
            RESOURCE_MANAGER_BURN_IDENT,
            &ResourceManagerBurnInput {
                bucket: bucket.into(),
            },
        )
    }

    /// Mints fungible resources
    pub fn mint<T: Into<Decimal>>(&self, amount: T) -> Bucket {
        self.call(
            FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
            &FungibleResourceManagerMintInput {
                amount: amount.into(),
            },
        )
    }

    /// Mints non-fungible resources
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

    pub fn amount_for_withdrawal(
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
