use radix_engine_lib::resource::{NonFungibleId, ResourceAddress};
use sbor::rust::collections::HashMap;
use sbor::rust::fmt;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::misc::copy_u8_array;
use utils::misc::ContextualDisplay;

use crate::abi::*;
use crate::engine::{api::*, scrypto_env::*};
use crate::math::*;
use crate::resource::*;
use crate::scrypto_env_native_fn;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum ResourceMethodAuthKey {
    Mint,
    Burn,
    Withdraw,
    Deposit,
    UpdateMetadata,
    UpdateNonFungibleData,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe)]
pub enum Mutability {
    LOCKED,
    MUTABLE(AccessRule),
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerCreateInvocation {
    pub resource_type: ResourceType,
    pub metadata: HashMap<String, String>,
    pub access_rules: HashMap<ResourceMethodAuthKey, (AccessRule, Mutability)>,
    pub mint_params: Option<MintParams>,
}

impl SysInvocation for ResourceManagerCreateInvocation {
    type Output = (ResourceAddress, Option<Bucket>);
}

impl ScryptoNativeInvocation for ResourceManagerCreateInvocation {}

impl Into<NativeFnInvocation> for ResourceManagerCreateInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Function(NativeFunctionInvocation::ResourceManager(
            ResourceManagerFunctionInvocation::Create(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerBucketBurnInvocation {
    pub bucket: Bucket,
}

impl SysInvocation for ResourceManagerBucketBurnInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for ResourceManagerBucketBurnInvocation {}

impl Into<NativeFnInvocation> for ResourceManagerBucketBurnInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Function(NativeFunctionInvocation::ResourceManager(
            ResourceManagerFunctionInvocation::BurnBucket(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerBurnInvocation {
    pub receiver: ResourceAddress,
    pub bucket: Bucket,
}

impl SysInvocation for ResourceManagerBurnInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for ResourceManagerBurnInvocation {}

impl Into<NativeFnInvocation> for ResourceManagerBurnInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::Burn(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerUpdateAuthInvocation {
    pub receiver: ResourceAddress,
    pub method: ResourceMethodAuthKey,
    pub access_rule: AccessRule,
}

impl SysInvocation for ResourceManagerUpdateAuthInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for ResourceManagerUpdateAuthInvocation {}

impl Into<NativeFnInvocation> for ResourceManagerUpdateAuthInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::UpdateAuth(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerLockAuthInvocation {
    pub receiver: ResourceAddress,
    pub method: ResourceMethodAuthKey,
}

impl SysInvocation for ResourceManagerLockAuthInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for ResourceManagerLockAuthInvocation {}

impl Into<NativeFnInvocation> for ResourceManagerLockAuthInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::LockAuth(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerCreateVaultInvocation {
    pub receiver: ResourceAddress,
}

impl SysInvocation for ResourceManagerCreateVaultInvocation {
    type Output = Vault;
}

impl ScryptoNativeInvocation for ResourceManagerCreateVaultInvocation {}

impl Into<NativeFnInvocation> for ResourceManagerCreateVaultInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::CreateVault(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerCreateBucketInvocation {
    pub receiver: ResourceAddress,
}

impl SysInvocation for ResourceManagerCreateBucketInvocation {
    type Output = Bucket;
}

impl ScryptoNativeInvocation for ResourceManagerCreateBucketInvocation {}

impl Into<NativeFnInvocation> for ResourceManagerCreateBucketInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::CreateBucket(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerMintInvocation {
    pub receiver: ResourceAddress,
    pub mint_params: MintParams,
}

impl SysInvocation for ResourceManagerMintInvocation {
    type Output = Bucket;
}

impl ScryptoNativeInvocation for ResourceManagerMintInvocation {}

impl Into<NativeFnInvocation> for ResourceManagerMintInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::Mint(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerGetMetadataInvocation {
    pub receiver: ResourceAddress,
}

impl SysInvocation for ResourceManagerGetMetadataInvocation {
    type Output = HashMap<String, String>;
}

impl ScryptoNativeInvocation for ResourceManagerGetMetadataInvocation {}

impl Into<NativeFnInvocation> for ResourceManagerGetMetadataInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::GetMetadata(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerGetResourceTypeInvocation {
    pub receiver: ResourceAddress,
}

impl SysInvocation for ResourceManagerGetResourceTypeInvocation {
    type Output = ResourceType;
}

impl ScryptoNativeInvocation for ResourceManagerGetResourceTypeInvocation {}

impl Into<NativeFnInvocation> for ResourceManagerGetResourceTypeInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::GetResourceType(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerGetTotalSupplyInvocation {
    pub receiver: ResourceAddress,
}

impl SysInvocation for ResourceManagerGetTotalSupplyInvocation {
    type Output = Decimal;
}

impl ScryptoNativeInvocation for ResourceManagerGetTotalSupplyInvocation {}

impl Into<NativeFnInvocation> for ResourceManagerGetTotalSupplyInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::GetTotalSupply(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerUpdateMetadataInvocation {
    pub receiver: ResourceAddress,
    pub metadata: HashMap<String, String>,
}

impl SysInvocation for ResourceManagerUpdateMetadataInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for ResourceManagerUpdateMetadataInvocation {}

impl Into<NativeFnInvocation> for ResourceManagerUpdateMetadataInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::UpdateMetadata(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerUpdateNonFungibleDataInvocation {
    pub receiver: ResourceAddress,
    pub id: NonFungibleId,
    pub data: Vec<u8>,
}

impl SysInvocation for ResourceManagerUpdateNonFungibleDataInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for ResourceManagerUpdateNonFungibleDataInvocation {}

impl Into<NativeFnInvocation> for ResourceManagerUpdateNonFungibleDataInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::UpdateNonFungibleData(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerNonFungibleExistsInvocation {
    pub receiver: ResourceAddress,
    pub id: NonFungibleId,
}

impl SysInvocation for ResourceManagerNonFungibleExistsInvocation {
    type Output = bool;
}

impl ScryptoNativeInvocation for ResourceManagerNonFungibleExistsInvocation {}

impl Into<NativeFnInvocation> for ResourceManagerNonFungibleExistsInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::NonFungibleExists(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerGetNonFungibleInvocation {
    pub receiver: ResourceAddress,
    pub id: NonFungibleId,
}

impl SysInvocation for ResourceManagerGetNonFungibleInvocation {
    type Output = [Vec<u8>; 2];
}

impl ScryptoNativeInvocation for ResourceManagerGetNonFungibleInvocation {}

impl Into<NativeFnInvocation> for ResourceManagerGetNonFungibleInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::ResourceManager(
            ResourceManagerMethodInvocation::GetNonFungible(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct ResourceManagerSetResourceAddressInvocation {
    pub receiver: ResourceAddress,
}

/// Represents a resource manager.
#[derive(Debug)]
pub struct ResourceManager(pub(crate) ResourceAddress);

impl ResourceManager {
    pub fn set_mintable(&mut self, access_rule: AccessRule) {
        let mut syscalls = ScryptoEnv;
        syscalls
            .sys_invoke(ResourceManagerUpdateAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Mint,
                access_rule,
            })
            .unwrap();
    }

    pub fn set_burnable(&mut self, access_rule: AccessRule) -> () {
        let mut syscalls = ScryptoEnv;
        syscalls
            .sys_invoke(ResourceManagerUpdateAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Burn,
                access_rule,
            })
            .unwrap()
    }

    pub fn set_withdrawable(&mut self, access_rule: AccessRule) -> () {
        let mut syscalls = ScryptoEnv;
        syscalls
            .sys_invoke(ResourceManagerUpdateAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Withdraw,
                access_rule,
            })
            .unwrap()
    }

    pub fn set_depositable(&mut self, access_rule: AccessRule) {
        let mut syscalls = ScryptoEnv;
        syscalls
            .sys_invoke(ResourceManagerUpdateAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Deposit,
                access_rule,
            })
            .unwrap()
    }

    pub fn set_updateable_metadata(&self, access_rule: AccessRule) {
        let mut syscalls = ScryptoEnv;
        syscalls
            .sys_invoke(ResourceManagerUpdateAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::UpdateMetadata,
                access_rule,
            })
            .unwrap()
    }

    pub fn set_updateable_non_fungible_data(&self, access_rule: AccessRule) {
        let mut syscalls = ScryptoEnv;
        syscalls
            .sys_invoke(ResourceManagerUpdateAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::UpdateNonFungibleData,
                access_rule,
            })
            .unwrap()
    }

    pub fn lock_mintable(&mut self) {
        let mut syscalls = ScryptoEnv;
        syscalls
            .sys_invoke(ResourceManagerLockAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Mint,
            })
            .unwrap()
    }

    pub fn lock_burnable(&mut self) {
        let mut syscalls = ScryptoEnv;
        syscalls
            .sys_invoke(ResourceManagerLockAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Burn,
            })
            .unwrap()
    }

    pub fn lock_withdrawable(&mut self) {
        let mut syscalls = ScryptoEnv;
        syscalls
            .sys_invoke(ResourceManagerLockAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Withdraw,
            })
            .unwrap()
    }

    pub fn lock_depositable(&mut self) {
        let mut syscalls = ScryptoEnv;
        syscalls
            .sys_invoke(ResourceManagerLockAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Deposit,
            })
            .unwrap()
    }

    pub fn lock_updateable_metadata(&mut self) {
        let mut syscalls = ScryptoEnv;
        syscalls
            .sys_invoke(ResourceManagerLockAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::UpdateMetadata,
            })
            .unwrap()
    }

    pub fn lock_updateable_non_fungible_data(&mut self) {
        let mut syscalls = ScryptoEnv;
        syscalls
            .sys_invoke(ResourceManagerLockAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::UpdateNonFungibleData,
            })
            .unwrap()
    }

    fn mint_internal(&mut self, mint_params: MintParams) -> Bucket {
        let mut syscalls = ScryptoEnv;
        syscalls
            .sys_invoke(ResourceManagerMintInvocation {
                mint_params,
                receiver: self.0,
            })
            .unwrap()
    }

    fn update_non_fungible_data_internal(&mut self, id: NonFungibleId, data: Vec<u8>) {
        let mut syscalls = ScryptoEnv;
        syscalls
            .sys_invoke(ResourceManagerUpdateNonFungibleDataInvocation {
                id,
                data,
                receiver: self.0,
            })
            .unwrap()
    }

    fn get_non_fungible_data_internal(&self, id: NonFungibleId) -> [Vec<u8>; 2] {
        let mut syscalls = ScryptoEnv;
        syscalls
            .sys_invoke(ResourceManagerGetNonFungibleInvocation {
                id,
                receiver: self.0,
            })
            .unwrap()
    }

    scrypto_env_native_fn! {
        pub fn metadata(&self) -> HashMap<String, String> {
            ResourceManagerGetMetadataInvocation {
                receiver: self.0,
            }
        }
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
        pub fn update_metadata(&mut self, metadata: HashMap<String, String>) -> () {
            ResourceManagerUpdateMetadataInvocation {
                receiver: self.0,
                metadata,
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
                bucket
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
        entries.insert(id.clone(), (data.immutable_data(), data.mutable_data()));
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
        self.update_non_fungible_data_internal(id.clone(), new_data.mutable_data())
    }
}
