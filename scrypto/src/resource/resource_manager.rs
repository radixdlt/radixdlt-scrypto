use sbor::rust::collections::HashMap;
use sbor::rust::fmt;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::abi::*;
use crate::address::*;
use crate::engine::{api::*, utils::*};
use crate::math::*;
use crate::misc::*;
use crate::native_fn;
use crate::resource::*;

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

impl ScryptoNativeInvocation for ResourceManagerCreateInvocation {
    type Output = (ResourceAddress, Option<Bucket>);
}

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

impl ScryptoNativeInvocation for ResourceManagerBucketBurnInvocation {
    type Output = ();
}

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

impl ScryptoNativeInvocation for ResourceManagerBurnInvocation {
    type Output = ();
}

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

impl ScryptoNativeInvocation for ResourceManagerUpdateAuthInvocation {
    type Output = ();
}

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

impl ScryptoNativeInvocation for ResourceManagerLockAuthInvocation {
    type Output = ();
}

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

impl ScryptoNativeInvocation for ResourceManagerCreateVaultInvocation {
    type Output = Vault;
}

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

impl ScryptoNativeInvocation for ResourceManagerCreateBucketInvocation {
    type Output = Bucket;
}

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

impl ScryptoNativeInvocation for ResourceManagerMintInvocation {
    type Output = Bucket;
}

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

impl ScryptoNativeInvocation for ResourceManagerGetMetadataInvocation {
    type Output = HashMap<String, String>;
}

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

impl ScryptoNativeInvocation for ResourceManagerGetResourceTypeInvocation {
    type Output = ResourceType;
}

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

impl ScryptoNativeInvocation for ResourceManagerGetTotalSupplyInvocation {
    type Output = Decimal;
}

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

impl ScryptoNativeInvocation for ResourceManagerUpdateMetadataInvocation {
    type Output = ();
}

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

impl ScryptoNativeInvocation for ResourceManagerUpdateNonFungibleDataInvocation {
    type Output = ();
}

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

impl ScryptoNativeInvocation for ResourceManagerNonFungibleExistsInvocation {
    type Output = bool;
}

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

impl ScryptoNativeInvocation for ResourceManagerGetNonFungibleInvocation {
    type Output = [Vec<u8>; 2];
}

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

/// Represents a resource address.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ResourceAddress {
    Normal([u8; 26]),
}

/// Represents a resource manager.
#[derive(Debug)]
pub struct ResourceManager(pub(crate) ResourceAddress);

impl ResourceManager {
    pub fn set_mintable(&mut self, access_rule: AccessRule) {
        let mut syscalls = Syscalls;
        syscalls
            .sys_invoke(ResourceManagerUpdateAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Mint,
                access_rule,
            })
            .unwrap();
    }

    pub fn set_burnable(&mut self, access_rule: AccessRule) -> () {
        let mut syscalls = Syscalls;
        syscalls
            .sys_invoke(ResourceManagerUpdateAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Burn,
                access_rule,
            })
            .unwrap()
    }

    pub fn set_withdrawable(&mut self, access_rule: AccessRule) -> () {
        let mut syscalls = Syscalls;
        syscalls
            .sys_invoke(ResourceManagerUpdateAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Withdraw,
                access_rule,
            })
            .unwrap()
    }

    pub fn set_depositable(&mut self, access_rule: AccessRule) {
        let mut syscalls = Syscalls;
        syscalls
            .sys_invoke(ResourceManagerUpdateAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Deposit,
                access_rule,
            })
            .unwrap()
    }

    pub fn set_updateable_metadata(&self, access_rule: AccessRule) {
        let mut syscalls = Syscalls;
        syscalls
            .sys_invoke(ResourceManagerUpdateAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::UpdateMetadata,
                access_rule,
            })
            .unwrap()
    }

    pub fn set_updateable_non_fungible_data(&self, access_rule: AccessRule) {
        let mut syscalls = Syscalls;
        syscalls
            .sys_invoke(ResourceManagerUpdateAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::UpdateNonFungibleData,
                access_rule,
            })
            .unwrap()
    }

    pub fn lock_mintable(&mut self) {
        let mut syscalls = Syscalls;
        syscalls
            .sys_invoke(ResourceManagerLockAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Mint,
            })
            .unwrap()
    }

    pub fn lock_burnable(&mut self) {
        let mut syscalls = Syscalls;
        syscalls
            .sys_invoke(ResourceManagerLockAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Burn,
            })
            .unwrap()
    }

    pub fn lock_withdrawable(&mut self) {
        let mut syscalls = Syscalls;
        syscalls
            .sys_invoke(ResourceManagerLockAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Withdraw,
            })
            .unwrap()
    }

    pub fn lock_depositable(&mut self) {
        let mut syscalls = Syscalls;
        syscalls
            .sys_invoke(ResourceManagerLockAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::Deposit,
            })
            .unwrap()
    }

    pub fn lock_updateable_metadata(&mut self) {
        let mut syscalls = Syscalls;
        syscalls
            .sys_invoke(ResourceManagerLockAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::UpdateMetadata,
            })
            .unwrap()
    }

    pub fn lock_updateable_non_fungible_data(&mut self) {
        let mut syscalls = Syscalls;
        syscalls
            .sys_invoke(ResourceManagerLockAuthInvocation {
                receiver: self.0,
                method: ResourceMethodAuthKey::UpdateNonFungibleData,
            })
            .unwrap()
    }

    fn mint_internal(&mut self, mint_params: MintParams) -> Bucket {
        let mut syscalls = Syscalls;
        syscalls
            .sys_invoke(ResourceManagerMintInvocation {
                mint_params,
                receiver: self.0,
            })
            .unwrap()
    }

    fn update_non_fungible_data_internal(&mut self, id: NonFungibleId, data: Vec<u8>) {
        let mut syscalls = Syscalls;
        syscalls
            .sys_invoke(ResourceManagerUpdateNonFungibleDataInvocation {
                id,
                data,
                receiver: self.0,
            })
            .unwrap()
    }

    fn get_non_fungible_data_internal(&self, id: NonFungibleId) -> [Vec<u8>; 2] {
        let mut syscalls = Syscalls;
        syscalls
            .sys_invoke(ResourceManagerGetNonFungibleInvocation {
                id,
                receiver: self.0,
            })
            .unwrap()
    }

    native_fn! {
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

//========
// binary
//========

impl TryFrom<&[u8]> for ResourceAddress {
    type Error = AddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            27 => match EntityType::try_from(slice[0])
                .map_err(|_| AddressError::InvalidEntityTypeId(slice[0]))?
            {
                EntityType::Resource => Ok(Self::Normal(copy_u8_array(&slice[1..]))),
                _ => Err(AddressError::InvalidEntityTypeId(slice[0])),
            },
            _ => Err(AddressError::InvalidLength(slice.len())),
        }
    }
}

impl ResourceAddress {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(EntityType::resource(self).id());
        match self {
            Self::Normal(v) => buf.extend(v),
        }
        buf
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.to_vec())
    }

    pub fn try_from_hex(hex_str: &str) -> Result<Self, AddressError> {
        let bytes = hex::decode(hex_str).map_err(|_| AddressError::HexDecodingError)?;

        Self::try_from(bytes.as_ref())
    }
}

scrypto_type!(ResourceAddress, ScryptoType::ResourceAddress, Vec::new());

//======
// text
//======

impl fmt::Debug for ResourceAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for ResourceAddress {
    type Error = AddressError;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        if let Some(encoder) = context.encoder {
            return encoder.encode_resource_address_to_fmt(f, self);
        }

        // This could be made more performant by streaming the hex into the formatter
        match self {
            ResourceAddress::Normal(_) => {
                write!(f, "NormalResource[{}]", self.to_hex())
            }
        }
        .map_err(|err| AddressError::FormatError(err))
    }
}
