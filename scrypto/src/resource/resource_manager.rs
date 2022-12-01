use radix_engine_interface::api::api::SysNativeInvokable;
use radix_engine_interface::api::types::{
    GlobalAddress, MetadataMethod, NativeFn, NativeMethod, RENodeId, ResourceManagerMethod,
};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::VaultMethodAuthKey::{Deposit, Recall, Withdraw};
use radix_engine_interface::model::*;

use sbor::rust::collections::HashMap;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto::engine::scrypto_env::ScryptoEnv;
use scrypto::scrypto_env_native_fn;

use crate::scrypto;

/// Represents a resource manager.
#[derive(Debug)]
pub struct ResourceManager(pub(crate) ResourceAddress);

impl ResourceManager {
    pub fn set_metadata(&mut self, key: String, value: String) {
        let mut env = ScryptoEnv;
        env.sys_invoke(MetadataSetInvocation {
            receiver: RENodeId::Global(GlobalAddress::Resource(self.0)),
            key,
            value,
        })
        .unwrap()
    }

    pub fn get_metadata(&mut self, key: String) -> Option<String> {
        let mut env = ScryptoEnv;
        env.sys_invoke(MetadataGetInvocation {
            receiver: RENodeId::Global(GlobalAddress::Resource(self.0)),
            key,
        })
        .unwrap()
    }

    pub fn set_mintable(&mut self, access_rule: AccessRule) {
        let mut env = ScryptoEnv;
        env.sys_invoke(AccessRulesSetMethodAccessRuleInvocation {
            receiver: RENodeId::Global(GlobalAddress::Resource(self.0)),
            index: 0,
            key: AccessRuleKey::Native(NativeFn::Method(NativeMethod::ResourceManager(
                ResourceManagerMethod::Mint,
            ))),
            rule: access_rule,
        })
        .unwrap();
    }

    pub fn set_burnable(&mut self, access_rule: AccessRule) -> () {
        let mut env = ScryptoEnv;
        env.sys_invoke(AccessRulesSetMethodAccessRuleInvocation {
            receiver: RENodeId::Global(GlobalAddress::Resource(self.0)),
            index: 0,
            key: AccessRuleKey::Native(NativeFn::Method(NativeMethod::ResourceManager(
                ResourceManagerMethod::Burn,
            ))),
            rule: access_rule,
        })
        .unwrap();
    }

    pub fn set_withdrawable(&mut self, access_rule: AccessRule) -> () {
        let mut env = ScryptoEnv;
        env.sys_invoke(ResourceManagerUpdateVaultAuthInvocation {
            receiver: self.0,
            method: Withdraw,
            access_rule,
        })
        .unwrap()
    }

    pub fn set_depositable(&mut self, access_rule: AccessRule) {
        let mut env = ScryptoEnv;
        env.sys_invoke(ResourceManagerUpdateVaultAuthInvocation {
            receiver: self.0,
            method: Deposit,
            access_rule,
        })
        .unwrap()
    }

    pub fn set_recallable(&mut self, access_rule: AccessRule) {
        let mut env = ScryptoEnv;
        env.sys_invoke(ResourceManagerUpdateVaultAuthInvocation {
            receiver: self.0,
            method: Recall,
            access_rule,
        })
        .unwrap()
    }

    pub fn set_updateable_metadata(&self, access_rule: AccessRule) {
        let mut env = ScryptoEnv;
        env.sys_invoke(AccessRulesSetMethodAccessRuleInvocation {
            receiver: RENodeId::Global(GlobalAddress::Resource(self.0)),
            index: 0,
            key: AccessRuleKey::Native(NativeFn::Method(NativeMethod::Metadata(
                MetadataMethod::Set,
            ))),
            rule: access_rule,
        })
        .unwrap();
    }

    pub fn set_updateable_non_fungible_data(&self, access_rule: AccessRule) {
        let mut env = ScryptoEnv;
        env.sys_invoke(AccessRulesSetMethodAccessRuleInvocation {
            receiver: RENodeId::Global(GlobalAddress::Resource(self.0)),
            index: 0,
            key: AccessRuleKey::Native(NativeFn::Method(NativeMethod::ResourceManager(
                ResourceManagerMethod::UpdateNonFungibleData,
            ))),
            rule: access_rule,
        })
        .unwrap();
    }

    pub fn lock_mintable(&mut self) {
        let mut env = ScryptoEnv;
        env.sys_invoke(AccessRulesSetMethodMutabilityInvocation {
            receiver: RENodeId::Global(GlobalAddress::Resource(self.0)),
            index: 0,
            key: AccessRuleKey::Native(NativeFn::Method(NativeMethod::ResourceManager(
                ResourceManagerMethod::Mint,
            ))),
            mutability: AccessRule::DenyAll,
        })
        .unwrap()
    }

    pub fn lock_burnable(&mut self) {
        let mut env = ScryptoEnv;
        env.sys_invoke(AccessRulesSetMethodMutabilityInvocation {
            receiver: RENodeId::Global(GlobalAddress::Resource(self.0)),
            index: 0,
            key: AccessRuleKey::Native(NativeFn::Method(NativeMethod::ResourceManager(
                ResourceManagerMethod::Burn,
            ))),
            mutability: AccessRule::DenyAll,
        })
        .unwrap()
    }

    pub fn lock_updateable_metadata(&mut self) {
        let mut env = ScryptoEnv;
        env.sys_invoke(AccessRulesSetMethodMutabilityInvocation {
            receiver: RENodeId::Global(GlobalAddress::Resource(self.0)),
            index: 0,
            key: AccessRuleKey::Native(NativeFn::Method(NativeMethod::Metadata(
                MetadataMethod::Set,
            ))),
            mutability: AccessRule::DenyAll,
        })
        .unwrap()
    }

    pub fn lock_updateable_non_fungible_data(&mut self) {
        let mut env = ScryptoEnv;
        env.sys_invoke(AccessRulesSetMethodMutabilityInvocation {
            receiver: RENodeId::Global(GlobalAddress::Resource(self.0)),
            index: 0,
            key: AccessRuleKey::Native(NativeFn::Method(NativeMethod::ResourceManager(
                ResourceManagerMethod::UpdateNonFungibleData,
            ))),
            mutability: AccessRule::DenyAll,
        })
        .unwrap()
    }

    pub fn lock_withdrawable(&mut self) {
        let mut env = ScryptoEnv;
        env.sys_invoke(ResourceManagerSetVaultAuthMutabilityInvocation {
            receiver: self.0,
            method: Withdraw,
            mutability: AccessRule::DenyAll,
        })
        .unwrap()
    }

    pub fn lock_depositable(&mut self) {
        let mut env = ScryptoEnv;
        env.sys_invoke(ResourceManagerSetVaultAuthMutabilityInvocation {
            receiver: self.0,
            method: Deposit,
            mutability: AccessRule::DenyAll,
        })
        .unwrap()
    }

    pub fn lock_recallable(&mut self) {
        let mut env = ScryptoEnv;
        env.sys_invoke(ResourceManagerSetVaultAuthMutabilityInvocation {
            receiver: self.0,
            method: Recall,
            mutability: AccessRule::DenyAll,
        })
        .unwrap()
    }

    fn mint_internal(&mut self, mint_params: MintParams) -> Bucket {
        let mut env = ScryptoEnv;
        env.sys_invoke(ResourceManagerMintInvocation {
            mint_params,
            receiver: self.0,
        })
        .unwrap()
    }

    fn update_non_fungible_data_internal(&mut self, id: NonFungibleId, data: Vec<u8>) {
        let mut env = ScryptoEnv;
        env.sys_invoke(ResourceManagerUpdateNonFungibleDataInvocation {
            id,
            data,
            receiver: self.0,
        })
        .unwrap()
    }

    fn get_non_fungible_data_internal(&self, id: NonFungibleId) -> [Vec<u8>; 2] {
        let mut env = ScryptoEnv;
        env.sys_invoke(ResourceManagerGetNonFungibleInvocation {
            id,
            receiver: self.0,
        })
        .unwrap()
    }

    scrypto_env_native_fn! {
        pub fn resource_type(&self) -> ResourceType {
            ResourceManagerGetResourceTypeInvocation {
                receiver: self.0,
            }
        }
        pub fn total_supply(&self) -> Decimal {
            ResourceManagerGetTotalSupplyInvocation {
                receiver: self.0,
            }
        }
        pub fn non_fungible_exists(&self, id: &NonFungibleId) -> bool {
            ResourceManagerNonFungibleExistsInvocation {
                receiver: self.0,
                id: id.clone()
            }
        }
        pub fn burn(&mut self, bucket: Bucket) -> () {
            ResourceManagerBurnInvocation {
                receiver: self.0,
                bucket: Bucket(bucket.0),
            }
        }
    }

    /// Mints fungible resources
    pub fn mint<T: Into<Decimal>>(&mut self, amount: T) -> Bucket {
        self.mint_internal(MintParams::Fungible {
            amount: amount.into(),
        })
    }

    /// Mints non-fungible resources
    pub fn mint_non_fungible<T: NonFungibleData>(&mut self, id: &NonFungibleId, data: T) -> Bucket {
        let mut entries = HashMap::new();
        entries.insert(
            id.clone(),
            (data.immutable_data().unwrap(), data.mutable_data().unwrap()),
        );
        self.mint_internal(MintParams::NonFungible { entries })
    }

    /// Returns the data of a non-fungible unit, both the immutable and mutable parts.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible resource or the specified non-fungible is not found.
    pub fn get_non_fungible_data<T: NonFungibleData>(&self, id: &NonFungibleId) -> T {
        let non_fungible = self.get_non_fungible_data_internal(id.clone());
        T::decode(&non_fungible[0], &non_fungible[1]).unwrap()
    }

    /// Updates the mutable part of a non-fungible unit.
    ///
    /// # Panics
    /// Panics if this is not a non-fungible resource or the specified non-fungible is not found.
    pub fn update_non_fungible_data<T: NonFungibleData>(
        &mut self,
        id: &NonFungibleId,
        new_data: T,
    ) {
        self.update_non_fungible_data_internal(id.clone(), new_data.mutable_data().unwrap())
    }
}
