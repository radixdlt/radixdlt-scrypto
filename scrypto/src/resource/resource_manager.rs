use crate::modules::HasAccessRules;
use crate::prelude::{Global, ObjectStub, ObjectStubHandle, ScryptoEncode};
use crate::*;
use radix_engine_interface::api::node_modules::metadata::METADATA_SETTER_ROLE;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::well_known_scrypto_custom_types::resource_address_type_data;
use radix_engine_interface::data::scrypto::well_known_scrypto_custom_types::RESOURCE_ADDRESS_ID;
use radix_engine_interface::data::scrypto::*;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode, ScryptoValue};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::ops::Deref;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::component::HasStub;

#[derive(Debug, Clone, Copy, ScryptoEncode, ScryptoDecode, ScryptoCategorize)]
#[sbor(transparent)]
pub struct ResourceManager(Global<ResourceManagerStub>);

impl Describe<ScryptoCustomTypeKind> for ResourceManager {
    const TYPE_ID: GlobalTypeId = GlobalTypeId::WellKnown([RESOURCE_ADDRESS_ID]);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, GlobalTypeId> {
        resource_address_type_data()
    }

    fn add_all_dependencies(_aggregator: &mut TypeAggregator<ScryptoCustomTypeKind>) {}
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
        self.0.lock_role(MINTER_ROLE);
    }

    pub fn lock_burnable(&self) {
        self.0.lock_role(BURNER_ROLE);
    }

    pub fn lock_updatable_non_fungible_data(&self) {
        self.0.lock_role(NON_FUNGIBLE_DATA_UPDATER_ROLE);
    }

    pub fn lock_withdrawable(&self) {
        self.0.lock_role(WITHDRAWER_ROLE);
    }

    pub fn lock_depositable(&self) {
        self.0.lock_role(DEPOSITOR_ROLE);
    }

    pub fn lock_recallable(&self) {
        self.0.lock_role(RECALLER_ROLE);
    }

    pub fn lock_freezeable(&self) {
        self.0.lock_role(FREEZER_ROLE);
    }

    pub fn set_updatable_metadata(&self, access_rule: AccessRule) {
        self.0.set_metadata_role(METADATA_SETTER_ROLE, access_rule);
    }

    pub fn lock_updatable_metadata(&self) {
        self.0.lock_metadata_role(METADATA_SETTER_ROLE);
    }
}

impl HasStub for ResourceManagerStub {
    type Stub = Self;
}

#[derive(Debug, Clone, Copy)]
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

    pub fn burn(&self, bucket: Bucket) {
        self.call(
            RESOURCE_MANAGER_BURN_IDENT,
            &ResourceManagerBurnInput { bucket },
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
        let mut entries = BTreeMap::new();
        let value: ScryptoValue = scrypto_decode(&scrypto_encode(&data).unwrap()).unwrap();
        entries.insert(id.clone(), (value,));
        self.call(
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
            &NonFungibleResourceManagerMintInput { entries },
        )
    }

    /// Mints ruid non-fungible resources
    pub fn mint_ruid_non_fungible<T: NonFungibleData>(&self, data: T) -> Bucket {
        let mut entries = Vec::new();
        let value: ScryptoValue = scrypto_decode(&scrypto_encode(&data).unwrap()).unwrap();
        entries.push((value,));

        self.call(
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_RUID_IDENT,
            &NonFungibleResourceManagerMintRuidInput { entries },
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
