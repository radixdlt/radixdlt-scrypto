use crate::prelude::{Global, ObjectStub, ObjectStubHandle, ScryptoEncode};
use crate::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode, ScryptoValue};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use scrypto::component::HasStub;
use std::ops::Deref;

#[derive(Debug, Clone, ScryptoSbor)]
#[sbor(transparent)]
pub struct ResourceManager(Global<ResourceManagerStub>);

impl From<ResourceAddress> for ResourceManager {
    fn from(value: ResourceAddress) -> Self {
        let stub = ResourceManagerStub::new(ObjectStubHandle::Global(value.into()));
        Self(Global(stub))
    }
}

impl Deref for ResourceManager {
    type Target = Global<ResourceManagerStub>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ResourceManager {
    pub fn resource_address(&self) -> ResourceAddress {
        ResourceAddress::new_or_panic(self.0 .0 .0.as_node_id().0.clone())
    }

    pub fn set_mintable(&self, access_rule: AccessRule) {
        let access_rules = self.0.access_rules();
        access_rules.set_authority_rule(MINT_AUTHORITY, access_rule);
    }

    pub fn set_burnable(&self, access_rule: AccessRule) {
        let access_rules = self.0.access_rules();
        access_rules.set_authority_rule(BURN_AUTHORITY, access_rule);
    }

    fn vault_blueprint_name(&self) -> &str {
        if self.0 .0 .0.as_node_id().is_global_fungible_resource() {
            FUNGIBLE_VAULT_BLUEPRINT
        } else {
            NON_FUNGIBLE_VAULT_BLUEPRINT
        }
    }

    pub fn set_withdrawable(&self, access_rule: AccessRule) {
        let blueprint_name = self.vault_blueprint_name();
        let access_rules = self.0.access_rules();
        access_rules.set_authority_rule_on_inner_blueprint(
            blueprint_name,
            WITHDRAW_AUTHORITY,
            access_rule,
        );
    }

    pub fn set_depositable(&self, access_rule: AccessRule) {
        let blueprint_name = self.vault_blueprint_name();
        let access_rules = self.0.access_rules();
        access_rules.set_authority_rule_on_inner_blueprint(
            blueprint_name,
            DEPOSIT_AUTHORITY,
            access_rule,
        );
    }

    pub fn set_recallable(&self, access_rule: AccessRule) {
        let blueprint_name = self.vault_blueprint_name();
        let access_rules = self.0.access_rules();
        access_rules.set_authority_rule_on_inner_blueprint(
            blueprint_name,
            RECALL_AUTHORITY,
            access_rule,
        );
    }

    pub fn set_updateable_metadata(&self, access_rule: AccessRule) {
        let access_rules = self.0.access_rules();
        access_rules.set_authority_rule("metadata", access_rule);
    }

    pub fn set_updateable_non_fungible_data(&self, access_rule: AccessRule) {
        let access_rules = self.0.access_rules();
        access_rules.set_authority_rule(UPDATE_NON_FUNGIBLE_DATA_AUTHORITY, access_rule);
    }

    pub fn lock_mintable(&self) {
        let access_rules = self.0.access_rules();
        access_rules.set_authority_mutability(MINT_AUTHORITY, AccessRule::DenyAll);
    }

    pub fn lock_burnable(&self) {
        let access_rules = self.0.access_rules();
        access_rules.set_authority_mutability(BURN_AUTHORITY, AccessRule::DenyAll);
    }

    pub fn lock_updateable_metadata(&self) {
        let access_rules = self.0.access_rules();
        access_rules.set_authority_mutability("metadata", AccessRule::DenyAll);
    }

    pub fn lock_updateable_non_fungible_data(&self) {
        let access_rules = self.0.access_rules();
        access_rules
            .set_authority_mutability(UPDATE_NON_FUNGIBLE_DATA_AUTHORITY, AccessRule::DenyAll);
    }

    pub fn lock_withdrawable(&self) {
        let access_rules = self.0.access_rules();
        access_rules.set_authority_mutability(WITHDRAW_AUTHORITY, AccessRule::DenyAll);
    }

    pub fn lock_depositable(&self) {
        let access_rules = self.0.access_rules();
        access_rules.set_authority_mutability(DEPOSIT_AUTHORITY, AccessRule::DenyAll);
    }

    pub fn lock_recallable(&self) {
        let access_rules = self.0.access_rules();
        access_rules.set_authority_mutability(RECALL_AUTHORITY, AccessRule::DenyAll);
    }
}

impl HasStub for ResourceManagerStub {
    type Stub = Self;
}

#[derive(Debug)]
pub struct ResourceManagerStub(pub ObjectStubHandle);

impl ObjectStub for ResourceManagerStub {
    fn new(handle: ObjectStubHandle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &ObjectStubHandle {
        &self.0
    }
}

impl ResourceManagerStub {
    pub fn resource_type(&self) -> ResourceType {
        self.call(
            RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT,
            &ResourceManagerGetResourceTypeInput {},
        )
    }

    pub fn total_supply(&self) -> Decimal {
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

    /// Mints uuid non-fungible resources
    pub fn mint_uuid_non_fungible<T: NonFungibleData>(&self, data: T) -> Bucket {
        let mut entries = Vec::new();
        let value: ScryptoValue = scrypto_decode(&scrypto_encode(&data).unwrap()).unwrap();
        entries.push((value,));

        self.call(
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_IDENT,
            &NonFungibleResourceManagerMintUuidInput { entries },
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
