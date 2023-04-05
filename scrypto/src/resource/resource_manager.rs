use crate::*;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::node_modules::auth::{
    AccessRulesSetGroupAccessRuleInput, AccessRulesSetMethodAccessRuleInput,
};
use radix_engine_interface::api::node_modules::metadata::METADATA_SET_IDENT;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode, ScryptoValue};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use scrypto::engine::scrypto_env::ScryptoEnv;

use crate::modules::AttachedMetadata;
use crate::prelude::ScryptoEncode;

/// Represents a resource manager.
#[derive(Debug)]
pub struct ResourceManager(pub ResourceAddress);

impl ResourceManager {
    pub fn metadata(&self) -> AttachedMetadata {
        AttachedMetadata(self.0.into())
    }

    pub fn set_mintable(&self, access_rule: AccessRule) {
        ScryptoEnv
            .call_module_method(
                self.0.as_node_id(),
                TypedModuleId::AccessRules,
                ACCESS_RULES_SET_GROUP_ACCESS_RULE_IDENT,
                scrypto_encode(&AccessRulesSetGroupAccessRuleInput {
                    object_key: ObjectKey::SELF,
                    name: "mint".to_string(),
                    rule: access_rule,
                })
                .unwrap(),
            )
            .unwrap();
    }

    pub fn set_burnable(&self, access_rule: AccessRule) -> () {
        ScryptoEnv
            .call_module_method(
                self.0.as_node_id(),
                TypedModuleId::AccessRules,
                ACCESS_RULES_SET_METHOD_ACCESS_RULE_IDENT,
                scrypto_encode(&AccessRulesSetMethodAccessRuleInput {
                    object_key: ObjectKey::SELF,
                    method_key: MethodKey::new(
                        TypedModuleId::ObjectState,
                        RESOURCE_MANAGER_BURN_IDENT,
                    ),
                    rule: AccessRuleEntry::AccessRule(access_rule),
                })
                .unwrap(),
            )
            .unwrap();
    }

    fn vault_blueprint_name(&self) -> &str {
        if self.0.as_node_id().is_global_fungible_resource() {
            NON_FUNGIBLE_VAULT_BLUEPRINT
        } else {
            FUNGIBLE_VAULT_BLUEPRINT
        }
    }

    pub fn set_withdrawable(&self, access_rule: AccessRule) {
        let _rtn = ScryptoEnv
            .call_module_method(
                self.0.as_node_id(),
                TypedModuleId::AccessRules,
                ACCESS_RULES_SET_GROUP_ACCESS_RULE_IDENT,
                scrypto_encode(&AccessRulesSetGroupAccessRuleInput {
                    object_key: ObjectKey::child_blueprint(self.vault_blueprint_name()),
                    name: "withdraw".to_string(),
                    rule: access_rule,
                })
                .unwrap(),
            )
            .unwrap();
    }

    pub fn set_depositable(&self, access_rule: AccessRule) {
        let _rtn = ScryptoEnv
            .call_module_method(
                self.0.as_node_id(),
                TypedModuleId::AccessRules,
                ACCESS_RULES_SET_METHOD_ACCESS_RULE_IDENT,
                scrypto_encode(&AccessRulesSetMethodAccessRuleInput {
                    object_key: ObjectKey::child_blueprint(self.vault_blueprint_name()),
                    method_key: MethodKey::new(TypedModuleId::ObjectState, VAULT_PUT_IDENT),
                    rule: AccessRuleEntry::AccessRule(access_rule),
                })
                .unwrap(),
            )
            .unwrap();
    }

    pub fn set_recallable(&self, access_rule: AccessRule) {
        let _rtn = ScryptoEnv
            .call_module_method(
                self.0.as_node_id(),
                TypedModuleId::AccessRules,
                ACCESS_RULES_SET_GROUP_ACCESS_RULE_IDENT,
                scrypto_encode(&AccessRulesSetGroupAccessRuleInput {
                    object_key: ObjectKey::child_blueprint(self.vault_blueprint_name()),
                    name: "recall".to_string(),
                    rule: access_rule,
                })
                .unwrap(),
            )
            .unwrap();
    }

    pub fn set_updateable_metadata(&self, access_rule: AccessRule) {
        ScryptoEnv
            .call_module_method(
                self.0.as_node_id(),
                TypedModuleId::AccessRules,
                ACCESS_RULES_SET_METHOD_ACCESS_RULE_IDENT,
                scrypto_encode(&AccessRulesSetMethodAccessRuleInput {
                    object_key: ObjectKey::SELF,
                    method_key: MethodKey::new(TypedModuleId::Metadata, METADATA_SET_IDENT),
                    rule: AccessRuleEntry::AccessRule(access_rule),
                })
                .unwrap(),
            )
            .unwrap();
    }

    pub fn set_updateable_non_fungible_data(&self, access_rule: AccessRule) {
        ScryptoEnv
            .call_module_method(
                self.0.as_node_id(),
                TypedModuleId::AccessRules,
                ACCESS_RULES_SET_METHOD_ACCESS_RULE_IDENT,
                scrypto_encode(&AccessRulesSetMethodAccessRuleInput {
                    object_key: ObjectKey::SELF,
                    method_key: MethodKey::new(
                        TypedModuleId::ObjectState,
                        NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT,
                    ),
                    rule: AccessRuleEntry::AccessRule(access_rule),
                })
                .unwrap(),
            )
            .unwrap();
    }

    pub fn lock_mintable(&self) {
        ScryptoEnv
            .call_module_method(
                self.0.as_node_id(),
                TypedModuleId::AccessRules,
                ACCESS_RULES_SET_GROUP_MUTABILITY_IDENT,
                scrypto_encode(&AccessRulesSetGroupMutabilityInput {
                    object_key: ObjectKey::SELF,
                    name: "mint".to_string(),
                    mutability: AccessRule::DenyAll,
                })
                .unwrap(),
            )
            .unwrap();
    }

    pub fn lock_burnable(&self) {
        ScryptoEnv
            .call_module_method(
                self.0.as_node_id(),
                TypedModuleId::AccessRules,
                ACCESS_RULES_SET_METHOD_MUTABILITY_IDENT,
                scrypto_encode(&AccessRulesSetMethodMutabilityInput {
                    object_key: ObjectKey::SELF,
                    method_key: MethodKey::new(
                        TypedModuleId::ObjectState,
                        RESOURCE_MANAGER_BURN_IDENT,
                    ),
                    mutability: AccessRuleEntry::AccessRule(AccessRule::DenyAll),
                })
                .unwrap(),
            )
            .unwrap();
    }

    pub fn lock_updateable_metadata(&self) {
        ScryptoEnv
            .call_module_method(
                self.0.as_node_id(),
                TypedModuleId::AccessRules,
                ACCESS_RULES_SET_METHOD_MUTABILITY_IDENT,
                scrypto_encode(&AccessRulesSetMethodMutabilityInput {
                    object_key: ObjectKey::SELF,
                    method_key: MethodKey::new(TypedModuleId::Metadata, METADATA_SET_IDENT),
                    mutability: AccessRuleEntry::AccessRule(AccessRule::DenyAll),
                })
                .unwrap(),
            )
            .unwrap();
    }

    pub fn lock_updateable_non_fungible_data(&self) {
        ScryptoEnv
            .call_module_method(
                self.0.as_node_id(),
                TypedModuleId::AccessRules,
                ACCESS_RULES_SET_METHOD_MUTABILITY_IDENT,
                scrypto_encode(&AccessRulesSetMethodMutabilityInput {
                    object_key: ObjectKey::SELF,
                    method_key: MethodKey::new(
                        TypedModuleId::ObjectState,
                        NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT,
                    ),
                    mutability: AccessRuleEntry::AccessRule(AccessRule::DenyAll),
                })
                .unwrap(),
            )
            .unwrap();
    }

    pub fn lock_withdrawable(&self) {
        let _rtn = ScryptoEnv.call_module_method(
            self.0.as_node_id(),
            TypedModuleId::AccessRules,
            ACCESS_RULES_SET_GROUP_MUTABILITY_IDENT,
            scrypto_encode(&AccessRulesSetGroupMutabilityInput {
                object_key: ObjectKey::child_blueprint(self.vault_blueprint_name()),
                name: "withdraw".to_string(),
                mutability: AccessRule::DenyAll,
            })
            .unwrap(),
        );
    }

    pub fn lock_depositable(&self) {
        let _rtn = ScryptoEnv.call_module_method(
            self.0.as_node_id(),
            TypedModuleId::AccessRules,
            ACCESS_RULES_SET_METHOD_MUTABILITY_IDENT,
            scrypto_encode(&AccessRulesSetMethodMutabilityInput {
                object_key: ObjectKey::child_blueprint(self.vault_blueprint_name()),
                method_key: MethodKey::new(TypedModuleId::ObjectState, VAULT_PUT_IDENT),
                mutability: AccessRuleEntry::AccessRule(AccessRule::DenyAll),
            })
            .unwrap(),
        );
    }

    pub fn lock_recallable(&self) {
        let _rtn = ScryptoEnv.call_module_method(
            self.0.as_node_id(),
            TypedModuleId::AccessRules,
            ACCESS_RULES_SET_GROUP_MUTABILITY_IDENT,
            scrypto_encode(&AccessRulesSetGroupMutabilityInput {
                object_key: ObjectKey::child_blueprint(self.vault_blueprint_name()),
                name: "recall".to_string(),
                mutability: AccessRule::DenyAll,
            })
            .unwrap(),
        );
    }

    pub fn resource_type(&self) -> ResourceType {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT,
                scrypto_encode(&ResourceManagerGetResourceTypeInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    pub fn total_supply(&self) -> Decimal {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT,
                scrypto_encode(&ResourceManagerGetTotalSupplyInput {}).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    pub fn non_fungible_exists(&self, id: &NonFungibleLocalId) -> bool {
        let mut env = ScryptoEnv;

        let rtn = env
            .call_method(
                self.0.as_node_id(),
                NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT,
                scrypto_encode(&NonFungibleResourceManagerExistsInput { id: id.clone() }).unwrap(),
            )
            .unwrap();

        scrypto_decode(&rtn).unwrap()
    }

    pub fn burn(&self, bucket: Bucket) {
        let mut env = ScryptoEnv;

        let _rtn = env
            .call_method(
                self.0.as_node_id(),
                RESOURCE_MANAGER_BURN_IDENT,
                scrypto_encode(&ResourceManagerBurnInput {
                    bucket: Bucket(bucket.0),
                })
                .unwrap(),
            )
            .unwrap();
    }

    /// Mints fungible resources
    pub fn mint<T: Into<Decimal>>(&self, amount: T) -> Bucket {
        let mut env = ScryptoEnv;

        let rtn = env
            .call_method(
                self.0.as_node_id(),
                FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
                scrypto_encode(&FungibleResourceManagerMintInput {
                    amount: amount.into(),
                })
                .unwrap(),
            )
            .unwrap();

        scrypto_decode(&rtn).unwrap()
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
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
                scrypto_encode(&NonFungibleResourceManagerMintInput { entries }).unwrap(),
            )
            .unwrap();

        scrypto_decode(&rtn).unwrap()
    }

    /// Mints uuid non-fungible resources
    pub fn mint_uuid_non_fungible<T: NonFungibleData>(&self, data: T) -> Bucket {
        let mut entries = Vec::new();
        let value: ScryptoValue = scrypto_decode(&scrypto_encode(&data).unwrap()).unwrap();
        entries.push((value,));
        let mut env = ScryptoEnv;

        let rtn = env
            .call_method(
                self.0.as_node_id(),
                NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_IDENT,
                scrypto_encode(&NonFungibleResourceManagerMintUuidInput { entries }).unwrap(),
            )
            .unwrap();

        scrypto_decode(&rtn).unwrap()
    }

    /// Returns the data of a non-fungible unit, both the immutable and mutable parts.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible resource or the specified non-fungible is not found.
    pub fn get_non_fungible_data<T: NonFungibleData>(&self, id: &NonFungibleLocalId) -> T {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                self.0.as_node_id(),
                NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT,
                scrypto_encode(&NonFungibleResourceManagerGetNonFungibleInput { id: id.clone() })
                    .unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    /// Updates the mutable part of a non-fungible unit.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible resource or the specified non-fungible is not found.
    pub fn update_non_fungible_data<D: ScryptoEncode>(
        &mut self,
        id: &NonFungibleLocalId,
        field_name: &str,
        new_data: D,
    ) {
        let mut env = ScryptoEnv;
        let _rtn = env
            .call_method(
                self.0.as_node_id(),
                NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT,
                scrypto_encode(&NonFungibleResourceManagerUpdateDataInput {
                    id: id.clone(),
                    field_name: field_name.to_string(),
                    data: scrypto_decode(&scrypto_encode(&new_data).unwrap()).unwrap(),
                })
                .unwrap(),
            )
            .unwrap();
    }
}
